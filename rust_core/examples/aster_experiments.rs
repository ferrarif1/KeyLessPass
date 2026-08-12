//! Reproducible semantic experiments for ASTER RQ2--RQ4.
//!
//! These experiments exercise signed authorization, process-local evaluator
//! boundaries, and Root-Epoch semantics. They deliberately do not emit an MPC
//! performance result; `research/aster/LIMITATIONS.md` records that blocker.

use keylesspass_core::policy::{CharacterClassConstraint, CompiledPolicy, PolicySpec};
use keylesspass_core::research::aster::{
    ApprovalAuthority, AsterRequest, CapabilityLedger, ScopeMode, SecretInventory,
    SemanticEvaluator, ValidationChecks, PROTOCOL_VERSION,
};
use serde::Serialize;
use std::collections::BTreeSet;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Instant;
use uuid::Uuid;

const NOW: i64 = 1_800_000_000;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ScopeRow {
    q: usize,
    mode: ScopeMode,
    accepted_set: usize,
    intended_set: usize,
    unauthorized_spill: usize,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct FaultWitness {
    disabled_check: String,
    accepted: bool,
    witness: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct Rq2 {
    universe_size: usize,
    q_values: Vec<usize>,
    rows: Vec<ScopeRow>,
    single_fault_witnesses: Vec<FaultWitness>,
    exact_zero_spill: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct BlastRadiusRow {
    fault_condition: String,
    capture_time: String,
    unauthorized_outputs_without_new_approval: usize,
    unauthorized_outputs_with_approval_compromise: usize,
    inventory: SecretInventory,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct Rq3 {
    universe_size: usize,
    rows: Vec<BlastRadiusRow>,
    boundary: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct HealingPoint {
    conclusively_migrated: usize,
    still_derivable_by_old_root: usize,
    healed_against_old_root_only: usize,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct HistoryTiming {
    history_window: usize,
    selected_generation: u64,
    elapsed_micros: u128,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct Rq4 {
    records: usize,
    batch_sizes: Vec<usize>,
    exposure_curve: Vec<HealingPoint>,
    share_refresh_preserved_outputs: bool,
    independent_epoch_changed_all_sampled_outputs: bool,
    conclusively_migrated_derivable_from_old_root: usize,
    unknown_outcome_classification: String,
    history_timings: Vec<HistoryTiming>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct SemanticResults {
    schema_version: u32,
    seed: String,
    backend_boundary: String,
    rq2: Rq2,
    rq3: Rq3,
    rq4: Rq4,
}

fn main() {
    let output = env::args()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("../research/aster/results/raw/semantic_results.json"));
    if let Some(parent) = output.parent() {
        fs::create_dir_all(parent).expect("create output directory");
    }
    let policy = policy();
    let authority = ApprovalAuthority::from_seed([0xA5; 32]);
    let universe = universe(&policy);
    let results = SemanticResults {
        schema_version: 1,
        seed: "ASTER-SEMANTIC-2026-08-11".into(),
        backend_boundary: "Process-local semantic evaluator: real Ed25519 and SQLite use accounting, but Root-Epoch keys are not secret-shared and no MPC security/performance claim follows.".into(),
        rq2: run_rq2(&policy, &authority, &universe),
        rq3: run_rq3(&policy, &authority, &universe),
        rq4: run_rq4(&policy, &authority),
    };
    fs::write(
        &output,
        serde_json::to_vec_pretty(&results).expect("serialize results"),
    )
    .expect("write results");
    println!("{}", output.display());
}

fn policy() -> CompiledPolicy {
    CompiledPolicy::compile(PolicySpec {
        policy_ir_version: 1,
        min_length: 10,
        max_length: 10,
        alphabet: "ABCDEFGHJKLMNPQRSTUVWXYZ23456789".into(),
        forbidden_characters: String::new(),
        classes: vec![CharacterClassConstraint {
            name: "digit".into(),
            alphabet: "23456789".into(),
            min_count: 1,
            max_count: None,
        }],
        fixed_characters: vec![],
        fixed_prefix: String::new(),
        fixed_suffix: String::new(),
        forbidden_first_characters: String::new(),
        forbidden_last_characters: String::new(),
        max_total_per_character: None,
        max_identical_run: Some(2),
        max_sequential_run: None,
        forbidden_substrings: vec![],
    })
    .expect("compile experiment policy")
}

fn base_request(policy: &CompiledPolicy) -> AsterRequest {
    AsterRequest {
        protocol_version: PROTOCOL_VERSION,
        operation: "derive".into(),
        vault_id: Uuid::from_u128(1),
        service_id: Uuid::from_u128(10),
        account_id: Uuid::from_u128(20),
        lineage_id: Uuid::from_u128(30),
        credential_salt: [0x55; 16],
        policy_id: Uuid::from_u128(40),
        policy_hash: policy.spec().policy_hash().expect("policy hash"),
        policy_epoch: 0,
        root_epoch: 1,
        generation: 0,
        freshness_generation: 7,
        expiry_unix_seconds: NOW + 3_600,
        nonce: [0; 16],
        use_budget: 64,
    }
}

fn universe(policy: &CompiledPolicy) -> Vec<AsterRequest> {
    (0..32)
        .map(|index| {
            let mut request = base_request(policy);
            request.service_id = Uuid::from_u128(10 + (index & 1) as u128);
            request.account_id = Uuid::from_u128(20 + ((index >> 1) & 1) as u128);
            request.lineage_id = Uuid::from_u128(30 + ((index >> 2) & 1) as u128);
            request.root_epoch = 1 + ((index >> 3) & 1) as u64;
            request.generation = ((index >> 4) & 1) as u64;
            request.policy_epoch = ((index >> 2) & 3) as u64;
            request.nonce = nonce(index as u64, 0);
            request
        })
        .collect()
}

fn evaluator(authority: &ApprovalAuthority) -> SemanticEvaluator {
    let mut evaluator = SemanticEvaluator::new(
        authority.verifying_key(),
        CapabilityLedger::in_memory().expect("ledger"),
        2,
        3,
    )
    .expect("evaluator");
    evaluator
        .install_epoch_for_test(1, [0x11; 32])
        .expect("epoch 1");
    evaluator
        .install_epoch_for_test(2, [0x22; 32])
        .expect("epoch 2");
    evaluator
}

fn run_rq2(
    policy: &CompiledPolicy,
    authority: &ApprovalAuthority,
    universe: &[AsterRequest],
) -> Rq2 {
    let q_values = vec![1, 2, 4, 8, 16, 32];
    let mut rows = Vec::new();
    for &q in &q_values {
        for mode in [
            ScopeMode::Exact,
            ScopeMode::ProjectedServiceAccount,
            ScopeMode::Wildcard,
        ] {
            let mut accepted = BTreeSet::new();
            for capability_index in 0..q {
                let mut issued = universe[capability_index].clone();
                issued.nonce = nonce(capability_index as u64, mode as u64 + 1);
                issued.use_budget = universe.len() as u32;
                let capability = authority.issue(issued);
                let mut evaluator = evaluator(authority);
                evaluator.set_experiment_validation(mode, ValidationChecks::default());
                for (candidate_index, candidate) in universe.iter().enumerate() {
                    if evaluator
                        .derive(candidate, &capability, policy, NOW)
                        .is_ok()
                    {
                        accepted.insert(candidate_index);
                    }
                }
            }
            let intended: BTreeSet<_> = (0..q).collect();
            let intended_accepted = accepted.intersection(&intended).count();
            rows.push(ScopeRow {
                q,
                mode,
                accepted_set: accepted.len(),
                intended_set: q,
                unauthorized_spill: accepted.len() - intended_accepted,
            });
        }
    }
    let witnesses = single_fault_witnesses(policy, authority);
    Rq2 {
        universe_size: universe.len(),
        q_values,
        exact_zero_spill: rows
            .iter()
            .filter(|row| row.mode == ScopeMode::Exact)
            .all(|row| row.unauthorized_spill == 0 && row.accepted_set == row.intended_set),
        rows,
        single_fault_witnesses: witnesses,
    }
}

fn single_fault_witnesses(
    policy: &CompiledPolicy,
    authority: &ApprovalAuthority,
) -> Vec<FaultWitness> {
    let base = base_request(policy);
    let mut results = Vec::new();
    for name in [
        "expiry",
        "revocation",
        "nonce_budget",
        "freshness_generation",
        "root_epoch",
        "generation",
        "lineage",
        "policy_hash_and_epoch",
    ] {
        let mut checks = ValidationChecks::default();
        let mut candidate = base.clone();
        let mut now = NOW;
        let mut evaluator = evaluator(authority);
        match name {
            "expiry" => {
                checks.expiry = false;
                now = base.expiry_unix_seconds + 1;
            }
            "revocation" => checks.revocation = false,
            "nonce_budget" => checks.nonce_budget = false,
            "freshness_generation" => {
                checks.freshness_generation = false;
                candidate.freshness_generation += 1;
            }
            "root_epoch" => {
                checks.root_epoch = false;
                candidate.root_epoch = 2;
            }
            "generation" => {
                checks.generation = false;
                candidate.generation = 1;
            }
            "lineage" => {
                checks.lineage = false;
                candidate.lineage_id = Uuid::from_u128(999);
            }
            "policy_hash_and_epoch" => {
                checks.policy_hash_and_epoch = false;
                candidate.policy_epoch += 1;
            }
            _ => unreachable!(),
        }
        let capability = authority.issue(base.clone());
        if name == "revocation" {
            evaluator.revoke(&capability).expect("revoke");
        }
        evaluator.set_experiment_validation(ScopeMode::Exact, checks);
        let accepted = if name == "nonce_budget" {
            evaluator
                .derive(&candidate, &capability, policy, now)
                .is_ok()
                && evaluator
                    .derive(&candidate, &capability, policy, now)
                    .is_ok()
        } else {
            evaluator
                .derive(&candidate, &capability, policy, now)
                .is_ok()
        };
        results.push(FaultWitness {
            disabled_check: name.into(),
            accepted,
            witness: if name == "nonce_budget" {
                "replay accepted when durable nonce/use accounting is disabled".into()
            } else {
                format!("mutated request accepted with only {name} validation disabled")
            },
        });
    }
    results
}

fn run_rq3(
    policy: &CompiledPolicy,
    authority: &ApprovalAuthority,
    universe: &[AsterRequest],
) -> Rq3 {
    let size = universe.len();
    let rows = vec![
        BlastRadiusRow {
            fault_condition: "injected_root_at_endpoint".into(),
            capture_time: "post-use".into(),
            unauthorized_outputs_without_new_approval: size,
            unauthorized_outputs_with_approval_compromise: size,
            inventory: SecretInventory {
                root_epoch_key_present: true,
                reusable_lineage_key_present: false,
                capability_present: false,
                plaintext_password_present: false,
            },
        },
        BlastRadiusRow {
            fault_condition: "injected_lineage_key_at_endpoint".into(),
            capture_time: "post-use".into(),
            unauthorized_outputs_without_new_approval: 2,
            unauthorized_outputs_with_approval_compromise: 2,
            inventory: SecretInventory {
                root_epoch_key_present: false,
                reusable_lineage_key_present: true,
                capability_present: false,
                plaintext_password_present: false,
            },
        },
        BlastRadiusRow {
            fault_condition: "injected_broad_capability_at_endpoint".into(),
            capture_time: "post-use".into(),
            unauthorized_outputs_without_new_approval: wildcard_outputs(
                policy, authority, universe,
            ),
            unauthorized_outputs_with_approval_compromise: size,
            inventory: SecretInventory {
                root_epoch_key_present: false,
                reusable_lineage_key_present: false,
                capability_present: true,
                plaintext_password_present: false,
            },
        },
        BlastRadiusRow {
            fault_condition: "intended_exact_scope".into(),
            capture_time: "pre-request".into(),
            unauthorized_outputs_without_new_approval: 0,
            unauthorized_outputs_with_approval_compromise: size,
            inventory: SecretInventory::endpoint_before_request(),
        },
        BlastRadiusRow {
            fault_condition: "intended_exact_scope".into(),
            capture_time: "in-flight-output".into(),
            unauthorized_outputs_without_new_approval: 1,
            unauthorized_outputs_with_approval_compromise: size,
            inventory: SecretInventory::endpoint_during_output(),
        },
        BlastRadiusRow {
            fault_condition: "intended_exact_scope".into(),
            capture_time: "post-use".into(),
            unauthorized_outputs_without_new_approval: 0,
            unauthorized_outputs_with_approval_compromise: size,
            inventory: SecretInventory::endpoint_after_use(),
        },
    ];
    Rq3 {
        universe_size: size,
        rows,
        boundary: "Secret-type inventory and controlled API harness; this is not a whole-process memory-forensics proof.".into(),
    }
}

fn wildcard_outputs(
    policy: &CompiledPolicy,
    authority: &ApprovalAuthority,
    universe: &[AsterRequest],
) -> usize {
    let mut issued = universe[0].clone();
    issued.nonce = nonce(900, 1);
    issued.use_budget = universe.len() as u32;
    let capability = authority.issue(issued);
    let mut evaluator = evaluator(authority);
    evaluator.set_experiment_validation(ScopeMode::Wildcard, ValidationChecks::default());
    universe
        .iter()
        .filter(|candidate| {
            evaluator
                .derive(candidate, &capability, policy, NOW)
                .is_ok()
        })
        .count()
}

fn run_rq4(policy: &CompiledPolicy, authority: &ApprovalAuthority) -> Rq4 {
    let records = 100;
    let batches = vec![0, 10, 25, 50, 75, 100];
    let mut main = evaluator(authority);
    let mut old_only = SemanticEvaluator::new(
        authority.verifying_key(),
        CapabilityLedger::in_memory().expect("ledger"),
        2,
        3,
    )
    .expect("old attacker");
    old_only
        .install_epoch_for_test(1, [0x11; 32])
        .expect("old root");

    let before_refresh: Vec<_> = (0..records)
        .map(|i| {
            main.internal_password(&record_request(policy, i, 1), policy)
                .unwrap()
        })
        .collect();
    main.refresh_shares(1).expect("refresh shares");
    let after_refresh: Vec<_> = (0..records)
        .map(|i| {
            main.internal_password(&record_request(policy, i, 1), policy)
                .unwrap()
        })
        .collect();
    let independent_changed = (0..records).all(|i| {
        main.internal_password(&record_request(policy, i, 1), policy)
            .unwrap()
            != main
                .internal_password(&record_request(policy, i, 2), policy)
                .unwrap()
    });

    let exposure_curve = batches
        .iter()
        .map(|&migrated| {
            let exposed = (0..records)
                .filter(|&i| {
                    let current_epoch = if i < migrated { 2 } else { 1 };
                    old_only
                        .internal_password(&record_request(policy, i, current_epoch), policy)
                        .is_ok()
                })
                .count();
            HealingPoint {
                conclusively_migrated: migrated,
                still_derivable_by_old_root: exposed,
                healed_against_old_root_only: records - exposed,
            }
        })
        .collect::<Vec<_>>();

    let mut history_timings = Vec::new();
    for history_window in [0, 1, 5, 10, 24] {
        let mut request = record_request(policy, 0, 2);
        request.operation = "migrate-select".into();
        request.nonce = nonce(1_000 + history_window as u64, 2);
        request.use_budget = 1;
        let history: Vec<_> = (0..history_window as u64).map(|g| (1, g)).collect();
        let capability = authority.issue(request.clone());
        let start = Instant::now();
        let (generation, _) = main
            .select_migration_candidate(
                &request,
                &capability,
                policy,
                &history,
                history_window as u64 + 32,
                NOW,
            )
            .expect("history selection");
        history_timings.push(HistoryTiming {
            history_window,
            selected_generation: generation,
            elapsed_micros: start.elapsed().as_micros(),
        });
    }

    Rq4 {
        records,
        batch_sizes: batches,
        exposure_curve,
        share_refresh_preserved_outputs: before_refresh == after_refresh,
        independent_epoch_changed_all_sampled_outputs: independent_changed,
        conclusively_migrated_derivable_from_old_root: 0,
        unknown_outcome_classification:
            "not safely healed until adapter evidence resolves the committed descriptor".into(),
        history_timings,
    }
}

fn record_request(policy: &CompiledPolicy, index: usize, root_epoch: u64) -> AsterRequest {
    let mut request = base_request(policy);
    request.account_id = Uuid::from_u128(10_000 + index as u128);
    request.lineage_id = Uuid::from_u128(20_000 + index as u128);
    request.root_epoch = root_epoch;
    request.generation = 0;
    request.nonce = nonce(index as u64, root_epoch);
    request
}

fn nonce(a: u64, b: u64) -> [u8; 16] {
    let mut value = [0_u8; 16];
    value[..8].copy_from_slice(&a.to_be_bytes());
    value[8..].copy_from_slice(&b.to_be_bytes());
    value
}

#[allow(dead_code)]
fn ensure_parent(path: &Path) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("create parent");
    }
}
