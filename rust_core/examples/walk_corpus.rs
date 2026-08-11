use keylesspass_core::epscd::{
    derive_credential_key, permutation_tweak, CredentialContext, SCHEME_VERSION_V1,
};
use keylesspass_core::permutation::{Ff1CycleWalking, MAX_FF1_DOMAIN_BITS, MIN_FF1_DOMAIN_SIZE};
use keylesspass_core::policy::{CompiledPolicy, PolicySpec};
use num_bigint::BigUint;
use serde_json::{json, Value};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::PathBuf;
use uuid::Uuid;

const GENERATIONS_PER_POLICY: u64 = 32;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..");
    let corpus: Value = serde_json::from_slice(&fs::read(
        root.join("experiments/real_policy_corpus/translated_corpus.json"),
    )?)?;
    let metrics: Value = serde_json::from_slice(&fs::read(
        root.join("experiments/real_policy_corpus/policy_metrics.json"),
    )?)?;
    let successful = metrics["records"]
        .as_array()
        .ok_or("missing metric records")?
        .iter()
        .filter(|row| row["compileStatus"] == "SUCCESS")
        .filter_map(|row| row["sourceRow"].as_u64())
        .collect::<HashSet<_>>();
    let source = corpus["records"]
        .as_array()
        .ok_or("missing corpus records")?
        .iter()
        .filter_map(|record| record["sourceRow"].as_u64().map(|row| (row, record)))
        .collect::<HashMap<_, _>>();

    let backend = Ff1CycleWalking::default();
    let mut rows = Vec::new();
    let mut all_walks = Vec::new();
    let mut cap_hits = 0_u64;
    for source_row in successful.iter().copied() {
        let record = source.get(&source_row).ok_or("successful row missing")?;
        let spec: PolicySpec = serde_json::from_value(record["policySpec"].clone())?;
        let compiled = CompiledPolicy::compile(spec.clone())?;
        let bits = (compiled.total_count() - BigUint::from(1_u8)).bits();
        let density = 2_f64.powf(compiled.entropy_bits() - bits as f64);
        if compiled.total_count() < &BigUint::from(MIN_FF1_DOMAIN_SIZE)
            || bits > MAX_FF1_DOMAIN_BITS
        {
            rows.push(json!({
                "sourceRow": source_row,
                "website": record["website"],
                "status": "BACKEND_DOMAIN_LIMIT",
                "domainBits": bits,
                "domainDensity": density,
                "reason": if bits > MAX_FF1_DOMAIN_BITS { "domain_exceeds_512_bits" } else { "domain_below_1000000" }
            }));
            continue;
        }

        let context = context_for(source_row);
        let key = derive_credential_key(&[0x57_u8; 32], &context)?;
        let tweak = permutation_tweak(&context, &spec.policy_hash()?)?;
        let mut walks = Vec::new();
        let mut policy_cap_hits = 0_u64;
        for generation in 0..GENERATIONS_PER_POLICY {
            match backend.permute_with_walk_count(
                &key,
                &tweak,
                compiled.total_count(),
                &BigUint::from(generation),
            ) {
                Ok((_, count)) => {
                    walks.push(count);
                    all_walks.push(count);
                }
                Err(error) if error.to_string().contains("cycle-walk limit exceeded") => {
                    policy_cap_hits += 1;
                    cap_hits += 1;
                }
                Err(error) => return Err(error.into()),
            }
        }
        rows.push(json!({
            "sourceRow": source_row,
            "website": record["website"],
            "status": "MEASURED",
            "domainBits": bits,
            "domainDensity": density,
            "generations": GENERATIONS_PER_POLICY,
            "walks": integer_summary(walks),
            "capHits": policy_cap_hits
        }));
    }
    rows.sort_by_key(|row| row["sourceRow"].as_u64().unwrap_or_default());
    let measured = rows
        .iter()
        .filter(|row| row["status"] == "MEASURED")
        .count();
    let limited = rows.len() - measured;
    let output = json!({
        "schemaVersion": 1,
        "successfulCompiledPolicies": successful.len(),
        "measuredPolicies": measured,
        "backendDomainLimitPolicies": limited,
        "generationsPerMeasuredPolicy": GENERATIONS_PER_POLICY,
        "aggregateWalks": integer_summary(all_walks),
        "capHitCount": cap_hits,
        "backend": {
            "minimumDomain": MIN_FF1_DOMAIN_SIZE,
            "maximumDomainBits": MAX_FF1_DOMAIN_BITS,
            "maximumWalks": backend.max_walks
        },
        "boundary": "Walk counts are concrete FF1 artifact observations for successfully compiled policies inside the backend's domain range. Policies outside that range remain compiler successes but have no FF1 measurement.",
        "records": rows
    });
    fs::write(
        root.join("experiments/performance/walk_corpus.json"),
        serde_json::to_vec_pretty(&output)?,
    )?;
    Ok(())
}

fn context_for(source_row: u64) -> CredentialContext {
    let seed = source_row as u128 + 10_000;
    CredentialContext {
        scheme_version: SCHEME_VERSION_V1,
        vault_id: Uuid::from_u128(seed),
        service_id: Uuid::from_u128(seed + 1),
        account_id: Uuid::from_u128(seed + 2),
        lineage_id: Uuid::nil(),
        credential_salt: [source_row as u8; 16],
        root_generation: 1,
        policy_id: Uuid::from_u128(seed + 3),
        policy_version: 1,
        policy_epoch: 1,
    }
}

fn integer_summary(mut values: Vec<u32>) -> Value {
    values.sort_unstable();
    let floats = values.iter().map(|value| *value as f64).collect::<Vec<_>>();
    json!({
        "samples": values.len(),
        "mean": if floats.is_empty() { Value::Null } else { json!(floats.iter().sum::<f64>() / floats.len() as f64) },
        "median": percentile(&floats, 0.50),
        "p95": percentile(&floats, 0.95),
        "p99": percentile(&floats, 0.99),
        "maximum": values.last().copied()
    })
}

fn percentile(sorted: &[f64], fraction: f64) -> Value {
    if sorted.is_empty() {
        Value::Null
    } else {
        json!(sorted[((sorted.len() - 1) as f64 * fraction).round() as usize])
    }
}
