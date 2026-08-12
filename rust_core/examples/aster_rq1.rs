//! Full public-corpus exact-policy and sequence experiment for ASTER RQ1.

use keylesspass_core::permutation::MIN_FF1_DOMAIN_SIZE;
use keylesspass_core::policy::{CompiledPolicy, PolicySpec, DEFAULT_MAX_POLICY_STATES};
use keylesspass_core::research::aster::{
    ApprovalAuthority, AsterRequest, CapabilityLedger, SemanticEvaluator, PROTOCOL_VERSION,
};
use num_bigint::BigUint;
use num_traits::ToPrimitive;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashSet;
use std::env;
use std::fs::{self, File};
use std::io::{BufWriter, Write};
use std::path::PathBuf;
use std::time::Instant;
use uuid::Uuid;

const ROOT: [u8; 32] = [0x31; 32];

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Corpus {
    records: Vec<CorpusRecord>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CorpusRecord {
    source_row: usize,
    website: String,
    translation_status: String,
    policy_spec: Option<PolicySpec>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct Rq1Record {
    source_row: usize,
    website: String,
    status: String,
    domain_size: Option<String>,
    effective_bits: Option<f64>,
    automaton_states: Option<usize>,
    compile_millis: u128,
    requested_generations: u64,
    generated_credentials: u64,
    policy_violations: u64,
    same_lineage_duplicates: u64,
    replay_mismatches: u64,
    second_lineage_equal_positions: u64,
    rank_unrank_failures: u64,
    derive_millis: u128,
    worker_threads: usize,
    error: Option<String>,
}

#[derive(Default)]
struct ChunkResult {
    outputs: HashSet<String>,
    generated: u64,
    policy_violations: u64,
    duplicates: u64,
    replay_mismatches: u64,
    error: Option<String>,
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 3 {
        eprintln!("usage: aster_rq1 <translated_corpus.json> <output.jsonl>");
        std::process::exit(2);
    }
    let corpus_path = PathBuf::from(&args[1]);
    let output_path = PathBuf::from(&args[2]);
    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent).expect("create output directory");
    }
    let corpus: Corpus =
        serde_json::from_slice(&fs::read(corpus_path).expect("read corpus")).expect("parse corpus");
    let file = File::create(output_path).expect("create result JSONL");
    let mut writer = BufWriter::new(file);
    let translated = corpus
        .records
        .into_iter()
        .filter(|record| record.translation_status == "translated")
        .collect::<Vec<_>>();
    for (index, record) in translated.into_iter().enumerate() {
        eprintln!(
            "RQ1 policy {} row {} {}",
            index + 1,
            record.source_row,
            record.website
        );
        let result = run_record(record);
        serde_json::to_writer(&mut writer, &result).expect("write result");
        writer.write_all(b"\n").expect("write newline");
        writer.flush().expect("flush result");
    }
}

fn run_record(record: CorpusRecord) -> Rq1Record {
    let spec = record
        .policy_spec
        .expect("translated policy has policySpec");
    let compile_start = Instant::now();
    let policy = match CompiledPolicy::compile_with_limit(spec, DEFAULT_MAX_POLICY_STATES) {
        Ok(policy) => policy,
        Err(error) => {
            return failure(
                record.source_row,
                record.website,
                "COMPILE_FAILED",
                compile_start.elapsed().as_millis(),
                error.to_string(),
            );
        }
    };
    let compile_millis = compile_start.elapsed().as_millis();
    let domain = policy.total_count().clone();
    let requested_generations = domain.to_u64().unwrap_or(100_000).min(100_000);
    let rank_unrank_failures = inverse_failures(&policy);
    if domain < BigUint::from(MIN_FF1_DOMAIN_SIZE) {
        return Rq1Record {
            source_row: record.source_row,
            website: record.website,
            status: "PERMUTATION_DOMAIN_UNSUPPORTED".into(),
            domain_size: Some(domain.to_string()),
            effective_bits: Some(policy.entropy_bits()),
            automaton_states: Some(policy.dfa().state_count()),
            compile_millis,
            requested_generations,
            generated_credentials: 0,
            policy_violations: 0,
            same_lineage_duplicates: 0,
            replay_mismatches: 0,
            second_lineage_equal_positions: 0,
            rank_unrank_failures,
            derive_millis: 0,
            worker_threads: 0,
            error: Some(
                "FF1 backend fails closed below the configured 1,000,000-element domain".into(),
            ),
        };
    }

    let workers = std::thread::available_parallelism()
        .map(|count| count.get().min(8))
        .unwrap_or(2)
        .min(requested_generations.max(1) as usize);
    let derive_start = Instant::now();
    let request = request_for(&policy, record.source_row as u128, Uuid::from_u128(0xA57E));
    let mut chunks = std::thread::scope(|scope| {
        let mut handles = Vec::new();
        for worker in 0..workers {
            let start = requested_generations * worker as u64 / workers as u64;
            let end = requested_generations * (worker as u64 + 1) / workers as u64;
            let policy_ref = &policy;
            let request = request.clone();
            handles.push(scope.spawn(move || derive_chunk(policy_ref, request, start, end)));
        }
        handles
            .into_iter()
            .map(|handle| handle.join().expect("RQ1 worker panicked"))
            .collect::<Vec<_>>()
    });
    let derive_millis = derive_start.elapsed().as_millis();
    let mut all_outputs = HashSet::new();
    let mut generated = 0;
    let mut violations = 0;
    let mut duplicates = 0;
    let mut replays = 0;
    let mut error = None;
    for chunk in &mut chunks {
        generated += chunk.generated;
        violations += chunk.policy_violations;
        duplicates += chunk.duplicates;
        replays += chunk.replay_mismatches;
        if error.is_none() {
            error = chunk.error.take();
        }
        for password in chunk.outputs.drain() {
            if !all_outputs.insert(password) {
                duplicates += 1;
            }
        }
    }
    let second_lineage_equal_positions = if error.is_none() {
        context_separation_equal_positions(&policy, &request, requested_generations.min(1_000))
    } else {
        0
    };
    let status = if error.is_some()
        || violations != 0
        || duplicates != 0
        || replays != 0
        || rank_unrank_failures != 0
    {
        "FAILED"
    } else {
        "SUCCESS"
    };
    Rq1Record {
        source_row: record.source_row,
        website: record.website,
        status: status.into(),
        domain_size: Some(domain.to_string()),
        effective_bits: Some(policy.entropy_bits()),
        automaton_states: Some(policy.dfa().state_count()),
        compile_millis,
        requested_generations,
        generated_credentials: generated,
        policy_violations: violations,
        same_lineage_duplicates: duplicates,
        replay_mismatches: replays,
        second_lineage_equal_positions,
        rank_unrank_failures,
        derive_millis,
        worker_threads: workers,
        error,
    }
}

fn derive_chunk(policy: &CompiledPolicy, base: AsterRequest, start: u64, end: u64) -> ChunkResult {
    let authority = ApprovalAuthority::from_seed([0xA5; 32]);
    let mut evaluator = SemanticEvaluator::new(
        authority.verifying_key(),
        CapabilityLedger::in_memory().expect("ledger"),
        2,
        3,
    )
    .expect("evaluator");
    evaluator.install_epoch_for_test(1, ROOT).expect("epoch");
    let mut result = ChunkResult::default();
    result.outputs.reserve((end - start) as usize);
    for generation in start..end {
        let mut request = base.clone();
        request.generation = generation;
        let first = match evaluator.internal_password(&request, policy) {
            Ok(password) => password,
            Err(error) => {
                result.error = Some(error.to_string());
                break;
            }
        };
        let replay = evaluator
            .internal_password(&request, policy)
            .expect("replay");
        result.generated += 1;
        if first != replay {
            result.replay_mismatches += 1;
        }
        if policy.rank(&first).is_err() {
            result.policy_violations += 1;
        }
        if !result.outputs.insert(first) {
            result.duplicates += 1;
        }
    }
    result
}

fn inverse_failures(policy: &CompiledPolicy) -> u64 {
    let domain = policy.total_count();
    let one = BigUint::from(1_u8);
    let mut ranks = vec![BigUint::from(0_u8), domain - &one, domain / 2_u8];
    for index in 0_u64..32 {
        let digest = Sha256::digest(format!("ASTER-RQ1-RANK-{index}").as_bytes());
        ranks.push(BigUint::from_bytes_be(&digest) % domain);
    }
    ranks
        .into_iter()
        .filter(|rank| {
            policy
                .unrank(rank)
                .and_then(|password| policy.rank(&password))
                .map(|recovered| recovered != *rank)
                .unwrap_or(true)
        })
        .count() as u64
}

fn context_separation_equal_positions(
    policy: &CompiledPolicy,
    base: &AsterRequest,
    samples: u64,
) -> u64 {
    let authority = ApprovalAuthority::from_seed([0xA5; 32]);
    let mut evaluator = SemanticEvaluator::new(
        authority.verifying_key(),
        CapabilityLedger::in_memory().expect("ledger"),
        2,
        3,
    )
    .expect("evaluator");
    evaluator.install_epoch_for_test(1, ROOT).expect("epoch");
    let mut equal = 0;
    for generation in 0..samples {
        let mut first = base.clone();
        first.generation = generation;
        let mut second = first.clone();
        second.lineage_id = Uuid::from_u128(0xBEEF);
        if evaluator.internal_password(&first, policy).unwrap()
            == evaluator.internal_password(&second, policy).unwrap()
        {
            equal += 1;
        }
    }
    equal
}

fn request_for(policy: &CompiledPolicy, record: u128, lineage: Uuid) -> AsterRequest {
    AsterRequest {
        protocol_version: PROTOCOL_VERSION,
        operation: "derive".into(),
        vault_id: Uuid::from_u128(1),
        service_id: Uuid::from_u128(1_000 + record),
        account_id: Uuid::from_u128(2_000 + record),
        lineage_id: lineage,
        credential_salt: record.to_be_bytes(),
        policy_id: Uuid::from_u128(3_000 + record),
        policy_hash: policy.spec().policy_hash().expect("policy hash"),
        policy_epoch: 1,
        root_epoch: 1,
        generation: 0,
        freshness_generation: 1,
        expiry_unix_seconds: 2_000_000_000,
        nonce: [0; 16],
        use_budget: 1,
    }
}

fn failure(
    source_row: usize,
    website: String,
    status: &str,
    compile_millis: u128,
    error: String,
) -> Rq1Record {
    Rq1Record {
        source_row,
        website,
        status: status.into(),
        domain_size: None,
        effective_bits: None,
        automaton_states: None,
        compile_millis,
        requested_generations: 0,
        generated_credentials: 0,
        policy_violations: 0,
        same_lineage_duplicates: 0,
        replay_mismatches: 0,
        second_lineage_equal_positions: 0,
        rank_unrank_failures: 0,
        derive_millis: 0,
        worker_threads: 0,
        error: Some(error),
    }
}
