use keylesspass_core::epscd::{derive_password, CredentialContext, SCHEME_VERSION_V2};
use keylesspass_core::permutation::Ff1CycleWalking;
use keylesspass_core::policy::{CharacterClassConstraint, CompiledPolicy, PolicySpec};
use rand::{rngs::StdRng, Rng, SeedableRng};
use serde::Serialize;
use std::collections::HashSet;
use std::time::Instant;
use uuid::Uuid;

const REJECTION_SUCCESSES: usize = 2_000;
const SEQUENCE_LENGTH: u64 = 100_000;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Output {
    schema_version: u32,
    rejection_density: Vec<DensityRow>,
    sequence: SequenceResult,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct DensityRow {
    alpha: f64,
    successful_outputs: usize,
    expected_retries: f64,
    observed_mean_retries: f64,
    median_retries: u64,
    p95_retries: u64,
    p99_retries: u64,
    p999_retries: u64,
    maximum_retries: u64,
    elapsed_millis: f64,
    successful_outputs_per_second: f64,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SequenceResult {
    scheme: &'static str,
    generations: u64,
    policy_violations: usize,
    duplicate_passwords: usize,
    replay_mismatches: usize,
    domain_size: String,
    effective_bits: f64,
    elapsed_seconds: f64,
    derivations_per_second_including_replay: f64,
    boundary: &'static str,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let rejection_density = [10_u64, 100, 1_000, 10_000]
        .into_iter()
        .map(measure_density)
        .collect();
    let sequence = measure_sequence()?;
    println!(
        "{}",
        serde_json::to_string_pretty(&Output {
            schema_version: 1,
            rejection_density,
            sequence,
        })?
    );
    Ok(())
}

fn measure_density(denominator: u64) -> DensityRow {
    let alpha = 1.0 / denominator as f64;
    let mut rng = StdRng::seed_from_u64(0x4550_5343_4400_0000 ^ denominator);
    let started = Instant::now();
    let mut retries = Vec::with_capacity(REJECTION_SUCCESSES);
    for _ in 0..REJECTION_SUCCESSES {
        let mut attempts = 1_u64;
        while rng.gen_range(0..denominator) != 0 {
            attempts += 1;
        }
        retries.push(attempts);
    }
    let elapsed = started.elapsed();
    retries.sort_unstable();
    DensityRow {
        alpha,
        successful_outputs: REJECTION_SUCCESSES,
        expected_retries: 1.0 / alpha,
        observed_mean_retries: retries.iter().sum::<u64>() as f64 / retries.len() as f64,
        median_retries: percentile(&retries, 0.50),
        p95_retries: percentile(&retries, 0.95),
        p99_retries: percentile(&retries, 0.99),
        p999_retries: percentile(&retries, 0.999),
        maximum_retries: *retries.last().unwrap_or(&0),
        elapsed_millis: elapsed.as_secs_f64() * 1_000.0,
        successful_outputs_per_second: REJECTION_SUCCESSES as f64 / elapsed.as_secs_f64(),
    }
}

fn measure_sequence() -> Result<SequenceResult, Box<dyn std::error::Error>> {
    let policy = CompiledPolicy::compile(sequence_policy())?;
    let context = CredentialContext {
        scheme_version: SCHEME_VERSION_V2,
        vault_id: Uuid::from_u128(1),
        service_id: Uuid::from_u128(2),
        account_id: Uuid::from_u128(3),
        lineage_id: Uuid::from_u128(4),
        credential_salt: [0x51; 16],
        root_generation: 1,
        policy_id: Uuid::from_u128(5),
        policy_version: 1,
        policy_epoch: 1,
    };
    let root = [0x73_u8; 32];
    let backend = Ff1CycleWalking::default();
    let started = Instant::now();
    let mut outputs = HashSet::with_capacity(SEQUENCE_LENGTH as usize);
    let mut violations = 0;
    let mut replay_mismatches = 0;
    for generation in 0..SEQUENCE_LENGTH {
        let first = derive_password(&root, &context, generation, &policy, &backend)?;
        if !policy.accepts(&first.password) {
            violations += 1;
        }
        let replay = derive_password(&root, &context, generation, &policy, &backend)?;
        if replay.password != first.password || replay.rank != first.rank {
            replay_mismatches += 1;
        }
        outputs.insert(first.password);
    }
    let elapsed = started.elapsed();
    Ok(SequenceResult {
        scheme: "EPSCD scheme v2",
        generations: SEQUENCE_LENGTH,
        policy_violations: violations,
        duplicate_passwords: SEQUENCE_LENGTH as usize - outputs.len(),
        replay_mismatches,
        domain_size: policy.total_count().to_string(),
        effective_bits: policy.entropy_bits(),
        elapsed_seconds: elapsed.as_secs_f64(),
        derivations_per_second_including_replay: (SEQUENCE_LENGTH * 2) as f64
            / elapsed.as_secs_f64(),
        boundary: "This is an implementation-scale check. Exact compliance and same-lineage non-repetition follow from the compiled-language and bijection/permutation arguments, not from observing this finite sample.",
    })
}

fn sequence_policy() -> PolicySpec {
    PolicySpec {
        policy_ir_version: 1,
        min_length: 12,
        max_length: 12,
        alphabet: "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789!@#$".to_string(),
        forbidden_characters: String::new(),
        classes: vec![
            class("lower", "abcdefghijklmnopqrstuvwxyz"),
            class("upper", "ABCDEFGHIJKLMNOPQRSTUVWXYZ"),
            class("digit", "0123456789"),
            class("symbol", "!@#$"),
        ],
        fixed_characters: Vec::new(),
        fixed_prefix: String::new(),
        fixed_suffix: String::new(),
        forbidden_first_characters: "!@#$".to_string(),
        forbidden_last_characters: "!@#$".to_string(),
        max_total_per_character: None,
        max_identical_run: Some(2),
        max_sequential_run: None,
        forbidden_substrings: Vec::new(),
    }
}

fn class(name: &str, alphabet: &str) -> CharacterClassConstraint {
    CharacterClassConstraint {
        name: name.to_string(),
        alphabet: alphabet.to_string(),
        min_count: 1,
        max_count: None,
    }
}

fn percentile(values: &[u64], fraction: f64) -> u64 {
    values[((values.len() - 1) as f64 * fraction).round() as usize]
}
