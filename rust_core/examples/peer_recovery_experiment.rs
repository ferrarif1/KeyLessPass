use chrono::Utc;
use keylesspass_core::crypto::recovery::create_network_share_set;
use keylesspass_core::research::peer_recovery::{
    reconstruct_network_share, split_network_share, NetworkRecoveryNode, RecoveryApprover,
    RecoveryClientSession, ReleaseLedger,
};
use serde::Serialize;
use std::env;
use std::fs;
use std::path::PathBuf;
use std::time::Instant;
use uuid::Uuid;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct RecoveryMeasurements {
    full_run: bool,
    release_iterations: usize,
    approval_signing_mean_us: f64,
    three_node_crypto_release_mean_us: f64,
    response_bytes_mean: f64,
    three_of_five_combinations_checked: usize,
    unavailable_nodes_checked: Vec<usize>,
    network_latency: &'static str,
    human_approval_latency: &'static str,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let full_run = env::var("KEYLESSPASS_FULL").as_deref() == Ok("1");
    let iterations = if full_run { 1_000 } else { 100 };
    let output = env::args()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("../experiments/results/factor-recovery-quick"));

    let root = [7_u8; 32];
    let set = create_network_share_set(
        &root,
        Uuid::new_v4(),
        1,
        1,
        1,
        "managed-experiment",
        1,
        "usb-experiment",
        1,
    )?;
    let fragments = split_network_share(&set.network, &set.manifest)?;
    let approvers = [
        RecoveryApprover::from_seed("approver-a", [1_u8; 32]),
        RecoveryApprover::from_seed("approver-b", [2_u8; 32]),
    ];
    let trusted = approvers
        .iter()
        .map(|approver| (approver.approver_id.clone(), approver.verifying_key()))
        .collect::<Vec<_>>();
    let nodes = fragments
        .iter()
        .cloned()
        .map(|fragment| NetworkRecoveryNode::new(fragment, &trusted))
        .collect::<Result<Vec<_>, _>>()?;
    let node_ids = fragments
        .iter()
        .map(|fragment| fragment.node_id.clone())
        .collect::<Vec<_>>();

    let mut approval_ns = 0_u128;
    let mut release_ns = 0_u128;
    let mut response_bytes = 0_usize;
    let mut ledger = ReleaseLedger::default();
    for _ in 0..iterations {
        let now = Utc::now().timestamp();
        let mut session = RecoveryClientSession::begin(&set.manifest, node_ids.clone(), now, 600)?;
        let started = Instant::now();
        approvers[0].approve(&mut session.ticket)?;
        approvers[1].approve(&mut session.ticket)?;
        approval_ns += started.elapsed().as_nanos();

        let started = Instant::now();
        let responses = nodes[..3]
            .iter()
            .map(|node| node.release(&session.ticket, &mut ledger, now))
            .collect::<Result<Vec<_>, _>>()?;
        let recovered = reconstruct_network_share(&session, &responses, &set.manifest)?;
        release_ns += started.elapsed().as_nanos();
        response_bytes += responses
            .iter()
            .map(|response| serde_json::to_vec(response).map(|bytes| bytes.len()))
            .collect::<Result<Vec<_>, _>>()?
            .into_iter()
            .sum::<usize>();
        if recovered != set.network {
            return Err("network recovery returned a different top-level share".into());
        }
    }

    let combinations = [
        [0, 1, 2],
        [0, 1, 3],
        [0, 1, 4],
        [0, 2, 3],
        [0, 2, 4],
        [0, 3, 4],
        [1, 2, 3],
        [1, 2, 4],
        [1, 3, 4],
        [2, 3, 4],
    ];
    for combination in combinations {
        let now = Utc::now().timestamp();
        let mut session = RecoveryClientSession::begin(&set.manifest, node_ids.clone(), now, 600)?;
        approvers[0].approve(&mut session.ticket)?;
        approvers[1].approve(&mut session.ticket)?;
        let responses = combination
            .iter()
            .map(|index| nodes[*index].release(&session.ticket, &mut ledger, now))
            .collect::<Result<Vec<_>, _>>()?;
        if reconstruct_network_share(&session, &responses, &set.manifest)? != set.network {
            return Err("a valid 3-of-5 combination failed".into());
        }
    }

    let measurements = RecoveryMeasurements {
        full_run,
        release_iterations: iterations,
        approval_signing_mean_us: approval_ns as f64 / iterations as f64 / 1_000.0,
        three_node_crypto_release_mean_us: release_ns as f64 / iterations as f64 / 1_000.0,
        response_bytes_mean: response_bytes as f64 / iterations as f64,
        three_of_five_combinations_checked: combinations.len(),
        unavailable_nodes_checked: vec![0, 1, 2],
        network_latency: "not measured; no network transport in this prototype",
        human_approval_latency: "not measured; cryptographic signature cost is reported separately",
    };
    fs::create_dir_all(&output)?;
    fs::write(
        output.join("factor_recovery_measurements.json"),
        serde_json::to_vec_pretty(&measurements)?,
    )?;
    println!(
        "completed {iterations} authorized releases and {} distinct 3-of-5 combinations",
        combinations.len()
    );
    Ok(())
}
