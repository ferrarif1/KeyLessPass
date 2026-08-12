use keylesspass_core::crypto::{encoder, kdf, recovery};
use keylesspass_core::domain::{
    apply_authentication_probe, compare_replicas, AuthenticationProbe, CredentialDescriptionRecord,
    EncodingDescriptor, ProbeVerdict, ProbedCredential, ReplicaRelation, RotationContract,
};
use keylesspass_core::service::{FreshnessAnchor, FreshnessService, SqliteFreshnessService};
use keylesspass_core::storage::CdrStore;
use serde::Serialize;
use std::collections::BTreeMap;
use std::path::PathBuf;
use std::time::Instant;
use uuid::Uuid;
use vsss_rs::Gf256;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Measurement {
    name: String,
    iterations: usize,
    total_ms: f64,
    mean_us: f64,
    median_us: Option<f64>,
    p95_us: Option<f64>,
    stddev_us: Option<f64>,
    detail: serde_json::Value,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let output = std::env::args()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("../experiments/results/latest"));
    std::fs::create_dir_all(&output)?;
    let full = std::env::var("KEYLESSPASS_FULL").as_deref() == Ok("1");
    let iterations = if full { 10_000 } else { 1_000 };
    let root = [0x42_u8; 32];
    let vault = Uuid::from_u128(0x102030405060708090a0b0c0d0e0f000);
    let set = recovery::create_share_set(&root, vault, 1, 1, 1, "computer", 1, "usb", 1)?;
    let recovery_share = recovery::decode_recovery_phrase(&set.recovery_phrase)?;
    let combinations = [
        ("recovery+computer", &recovery_share, &set.managed_computer),
        ("recovery+usb", &recovery_share, &set.usb),
        ("computer+usb", &set.managed_computer, &set.usb),
    ];
    let mut measurements = Vec::new();

    let started = Instant::now();
    let mut samples = Vec::with_capacity(iterations);
    for _ in 0..iterations {
        let iteration_started = Instant::now();
        let shares = Gf256::split_array(2, 3, root, rand::rngs::OsRng).map_err(|error| {
            std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Shamir split failed: {error}"),
            )
        })?;
        std::hint::black_box(shares);
        samples.push(iteration_started.elapsed().as_secs_f64() * 1_000_000.0);
    }
    measurements.push(timing(
        "shamir_split_2_of_3",
        iterations,
        started,
        Some(&samples),
        serde_json::json!({}),
    ));

    for (name, left, right) in combinations {
        let started = Instant::now();
        let mut samples = Vec::with_capacity(iterations);
        for _ in 0..iterations {
            let iteration_started = Instant::now();
            std::hint::black_box(recovery::recover_root_key(left, right, &set.manifest)?);
            samples.push(iteration_started.elapsed().as_secs_f64() * 1_000_000.0);
        }
        measurements.push(timing(
            &format!("recover_{name}"),
            iterations,
            started,
            Some(&samples),
            serde_json::json!({}),
        ));
    }
    measurements.push(Measurement {
        name: "factor_storage_bytes".to_string(),
        iterations: 1,
        total_ms: 0.0,
        mean_us: 0.0,
        median_us: None,
        p95_us: None,
        stddev_us: None,
        detail: serde_json::json!({
            "recoveryPhraseUtf8": set.recovery_phrase.len(),
            "recoveryPhraseWords": set.recovery_phrase.split_whitespace().count(),
            "managedEnvelopeJcs": serde_json_canonicalizer::to_vec(&set.managed_computer)?.len(),
            "usbEnvelopeJcs": serde_json_canonicalizer::to_vec(&set.usb)?.len(),
            "manifestJcs": serde_json_canonicalizer::to_vec(&set.manifest)?.len()
        }),
    });

    let descriptor = EncodingDescriptor::default();
    let mut derivation_record = CredentialDescriptionRecord::new(
        vault,
        1,
        1,
        "benchmark",
        "benchmark.example",
        "benchmark-account",
        "",
        descriptor.clone(),
    );
    derivation_record.service_id = Uuid::from_u128(0xaaaaaaaa111122223333444444444444);
    derivation_record.account_id = Uuid::from_u128(0xbbbbbbbb111122223333444444444444);
    derivation_record.policy_id = Uuid::from_u128(0xcccccccc111122223333444444444444);
    derivation_record.salt = "EREREREREREREREREREREQ==".to_string();
    let mut positions = vec![0_u64; descriptor.length];
    let mut characters = BTreeMap::<char, u64>::new();
    let started = Instant::now();
    let mut samples = Vec::with_capacity(iterations);
    for iteration in 0..iterations {
        let iteration_started = Instant::now();
        derivation_record.credential_generation = iteration as u64 + 1;
        let secret = kdf::derive_service_secret_v3(&root, &derivation_record)?;
        let password = encoder::encode_password(&secret, &descriptor)?;
        for (position, character) in password.chars().enumerate() {
            positions[position] += 1;
            *characters.entry(character).or_default() += 1;
        }
        samples.push(iteration_started.elapsed().as_secs_f64() * 1_000_000.0);
    }
    measurements.push(timing(
        "password_derivation_and_encoding",
        iterations,
        started,
        Some(&samples),
        serde_json::json!({
            "positionObservationCount": positions,
            "characterCounts": characters,
            "passwordSpaceUpperBoundLog2": encoder::password_space_upper_bound_log2(&descriptor)?
        }),
    ));

    for count in if full {
        vec![100, 1_000, 10_000, 100_000]
    } else {
        vec![100, 1_000]
    } {
        measurements.push(cdr_measurement(count, vault, &root)?);
    }

    let rotation_paths = 4_000;
    let started = Instant::now();
    let mut samples = Vec::with_capacity(rotation_paths);
    for iteration in 0..rotation_paths {
        let iteration_started = Instant::now();
        let active = CredentialDescriptionRecord::new(
            vault,
            1,
            1,
            "service",
            "service.example",
            "account",
            "",
            descriptor.clone(),
        );
        let mut pending = CredentialDescriptionRecord::rotation_from_with_contract(
            &active,
            descriptor.clone(),
            RotationContract::AtomicReplacement,
        );
        let probes = match iteration % 4 {
            0 => [
                (ProbedCredential::New, ProbeVerdict::Success),
                (ProbedCredential::Old, ProbeVerdict::ConclusiveFailure),
            ],
            1 => [
                (ProbedCredential::Old, ProbeVerdict::Success),
                (ProbedCredential::New, ProbeVerdict::ConclusiveFailure),
            ],
            2 => [
                (ProbedCredential::New, ProbeVerdict::Success),
                (ProbedCredential::Old, ProbeVerdict::Success),
            ],
            _ => [
                (ProbedCredential::New, ProbeVerdict::ConclusiveFailure),
                (ProbedCredential::Old, ProbeVerdict::ConclusiveFailure),
            ],
        };
        for (credential, verdict) in probes {
            apply_authentication_probe(
                &mut pending,
                AuthenticationProbe::now(credential, verdict, "benchmark-node"),
            )?;
        }
        samples.push(iteration_started.elapsed().as_secs_f64() * 1_000_000.0);
    }
    measurements.push(timing(
        "unknown_outcome_reconciliation",
        rotation_paths,
        started,
        Some(&samples),
        serde_json::json!({"successfulStateExecutions": rotation_paths, "outcomes": ["new_only", "old_only", "both", "neither"]}),
    ));

    measurements.push(freshness_measurement(
        if full { 10_000 } else { 1_000 },
        vault,
    )?);

    let environment = serde_json::json!({
        "os": std::env::consts::OS,
        "arch": std::env::consts::ARCH,
        "rustPackageVersion": env!("CARGO_PKG_VERSION"),
        "fullRun": full,
        "randomness": "OS CSPRNG for Shamir; fixed CDR fields with credentialGeneration=iteration+1 for deterministic derivation/encoder inputs",
        "command": if full {"KEYLESSPASS_FULL=1 cargo run --release --example research_evaluation -- <output>"} else {"cargo run --release --example research_evaluation -- <output>"}
    });
    std::fs::write(
        output.join("measurements.json"),
        serde_json::to_vec_pretty(&serde_json::json!({
            "environment": environment,
            "measurements": measurements
        }))?,
    )?;
    let mut csv =
        String::from("name,iterations,total_ms,mean_us,median_us,p95_us,stddev_us,detail_json\n");
    for item in &measurements {
        csv.push_str(&format!(
            "{},{},{:.6},{:.6},{},{},{},{}\n",
            item.name,
            item.iterations,
            item.total_ms,
            item.mean_us,
            optional_number(item.median_us),
            optional_number(item.p95_us),
            optional_number(item.stddev_us),
            csv_escape(&item.detail.to_string())
        ));
    }
    std::fs::write(output.join("measurements.csv"), csv)?;
    println!(
        "wrote {} measurements to {}",
        measurements.len(),
        output.display()
    );
    Ok(())
}

fn cdr_measurement(
    count: usize,
    vault: Uuid,
    root: &[u8; 32],
) -> Result<Measurement, Box<dyn std::error::Error>> {
    let directory = std::env::temp_dir().join(format!("keylesspass-eval-{}", Uuid::new_v4()));
    std::fs::create_dir_all(&directory)?;
    let database = directory.join("cdr.sqlite3");
    let store = CdrStore::new(&database);
    store.init()?;
    let mut records = Vec::with_capacity(count);
    for index in 0..count {
        let mut record = CredentialDescriptionRecord::new(
            vault,
            1,
            index as u64 + 1,
            format!("service-{index}"),
            format!("service-{index}.example"),
            format!("account-{index}"),
            "",
            EncodingDescriptor::default(),
        );
        record.set_mac(root)?;
        records.push(record);
    }
    let write_started = Instant::now();
    store.replace_all(&records)?;
    let write_ms = write_started.elapsed().as_secs_f64() * 1_000.0;
    let load_started = Instant::now();
    let loaded = store.list_all()?;
    let load_ms = load_started.elapsed().as_secs_f64() * 1_000.0;
    let query_started = Instant::now();
    let selected = store.get(records[count / 2].record_id, None)?;
    let query_us = query_started.elapsed().as_secs_f64() * 1_000_000.0;
    let mut fork = selected.clone();
    fork.notes = "concurrent".to_string();
    let conflict = compare_replicas(&selected, &fork)?;
    assert_eq!(conflict, ReplicaRelation::ConcurrentModification);
    let bytes = std::fs::metadata(&database)?.len();
    std::fs::remove_dir_all(&directory)?;
    Ok(Measurement {
        name: format!("cdr_scale_{count}"),
        iterations: count,
        total_ms: write_ms + load_ms,
        mean_us: (write_ms + load_ms) * 1_000.0 / count as f64,
        median_us: None,
        p95_us: None,
        stddev_us: None,
        detail: serde_json::json!({
            "databaseBytes": bytes,
            "writeMs": write_ms,
            "loadMs": load_ms,
            "queryUs": query_us,
            "loadedRecords": loaded.len(),
            "conflictDetection": "concurrent_modification"
        }),
    })
}

fn timing(
    name: &str,
    iterations: usize,
    started: Instant,
    samples_us: Option<&[f64]>,
    detail: serde_json::Value,
) -> Measurement {
    let total_ms = started.elapsed().as_secs_f64() * 1_000.0;
    let (median_us, p95_us, stddev_us) = samples_us
        .map(distribution)
        .map_or((None, None, None), |(median, p95, stddev)| {
            (Some(median), Some(p95), Some(stddev))
        });
    Measurement {
        name: name.to_string(),
        iterations,
        total_ms,
        mean_us: total_ms * 1_000.0 / iterations as f64,
        median_us,
        p95_us,
        stddev_us,
        detail,
    }
}

fn distribution(samples: &[f64]) -> (f64, f64, f64) {
    let mut sorted = samples.to_vec();
    sorted.sort_by(f64::total_cmp);
    let mean = sorted.iter().sum::<f64>() / sorted.len() as f64;
    let median = sorted[sorted.len() / 2];
    let p95 = sorted[((sorted.len() as f64 * 0.95).ceil() as usize).saturating_sub(1)];
    let variance = sorted
        .iter()
        .map(|value| (value - mean).powi(2))
        .sum::<f64>()
        / sorted.len() as f64;
    (median, p95, variance.sqrt())
}

fn freshness_measurement(
    iterations: usize,
    vault: Uuid,
) -> Result<Measurement, Box<dyn std::error::Error>> {
    let directory = std::env::temp_dir().join(format!("keylesspass-freshness-{}", Uuid::new_v4()));
    std::fs::create_dir_all(&directory)?;
    let database = directory.join("freshness.sqlite3");
    let service = SqliteFreshnessService::new(&database)?;
    let started = Instant::now();
    let mut samples = Vec::with_capacity(iterations);
    for index in 0..iterations {
        let iteration_started = Instant::now();
        service.compare_and_set(
            (index > 0).then_some(index as u64),
            FreshnessAnchor {
                vault_id: vault,
                root_generation: 1,
                share_set_generation: 1,
                cdr_epoch: index as u64 + 1,
                credentials: Default::default(),
                operation_log_digest: format!("digest-{index}"),
                updated_at: chrono::Utc::now(),
            },
        )?;
        samples.push(iteration_started.elapsed().as_secs_f64() * 1_000_000.0);
    }
    drop(service);
    let reopened = SqliteFreshnessService::new(&database)?;
    let persisted_epoch = reopened.read(vault)?.map(|anchor| anchor.cdr_epoch);
    let measurement = timing(
        "persistent_freshness_compare_and_set",
        iterations,
        started,
        Some(&samples),
        serde_json::json!({"persistedAfterRestart": persisted_epoch == Some(iterations as u64)}),
    );
    std::fs::remove_dir_all(directory)?;
    Ok(measurement)
}

fn optional_number(value: Option<f64>) -> String {
    value.map_or_else(String::new, |number| format!("{number:.6}"))
}

fn csv_escape(value: &str) -> String {
    format!("\"{}\"", value.replace('"', "\"\""))
}
