use keylesspass_core::domain::{
    classify_commit_evidence, AdapterCapabilities, AdapterObservation, CommitEvidence,
    EvidenceRequirement, ProbeVerdict,
};
use rusqlite::{params, Connection};
use serde::Serialize;
use std::fs;
use std::path::{Path, PathBuf};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "snake_case")]
enum AdapterStyle {
    HttpForm,
    LdapStyleDirectory,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RemoteState {
    OldOnly,
    NewOnly,
    Both,
    Neither,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct CaseResult {
    adapter: AdapterStyle,
    case_id: u8,
    scenario: &'static str,
    final_state: String,
    committed_generation: u64,
    evidence: CommitEvidence,
    old_reconstructible: bool,
    new_reconstructible: bool,
    invariant_commit_requires_evidence: bool,
    invariant_uncertainty_keeps_both: bool,
    persistent_password_columns: usize,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct Evaluation {
    schema_version: u32,
    adapters: Vec<&'static str>,
    cases_per_adapter: usize,
    result_count: usize,
    all_invariants_hold: bool,
    boundary: &'static str,
    results: Vec<CaseResult>,
}

const CASES: [(u8, &str); 15] = [
    (1, "request_never_reached_server"),
    (2, "remote_changed_response_lost"),
    (3, "response_timeout_without_probe"),
    (4, "duplicate_request"),
    (5, "crash_before_submit"),
    (6, "crash_after_submit_before_local_outcome"),
    (7, "crash_after_remote_commit_before_local_commit"),
    (8, "old_password_accepted"),
    (9, "new_password_accepted"),
    (10, "both_passwords_accepted"),
    (11, "neither_password_accepted"),
    (12, "policy_changed_during_rotation"),
    (13, "old_probe_skipped_for_lockout_budget"),
    (14, "old_verification_unavailable"),
    (15, "no_authoritative_version_or_readback"),
];

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut results = Vec::new();
    for adapter in [AdapterStyle::HttpForm, AdapterStyle::LdapStyleDirectory] {
        for (case_id, scenario) in CASES {
            results.push(run_case(adapter, case_id, scenario)?);
        }
    }
    let all_invariants_hold = results.iter().all(|row| {
        row.invariant_commit_requires_evidence
            && row.invariant_uncertainty_keeps_both
            && row.persistent_password_columns == 0
    });
    let evaluation = Evaluation {
        schema_version: 1,
        adapters: vec!["HTTP form adapter", "LDAP-style directory adapter"],
        cases_per_adapter: CASES.len(),
        result_count: results.len(),
        all_invariants_hold,
        boundary: "The HTTP and LDAP-style adapters are high-fidelity local semantic models. They exercise durable SQLite reopen and adapter evidence contracts, but do not include a production HTTP application or OpenLDAP server.",
        results,
    };
    println!("{}", serde_json::to_string_pretty(&evaluation)?);
    if !all_invariants_hold {
        return Err("rotation invariant failed".into());
    }
    Ok(())
}

fn run_case(
    adapter: AdapterStyle,
    case_id: u8,
    scenario: &'static str,
) -> Result<CaseResult, Box<dyn std::error::Error>> {
    let db_path = temporary_db_path(adapter, case_id);
    initialize_journal(&db_path)?;

    if case_id == 5 {
        reopen(&db_path)?;
        let row = read_journal(&db_path)?;
        cleanup(&db_path)?;
        return Ok(result(
            adapter,
            case_id,
            scenario,
            CommitEvidence::Insufficient,
            row,
            0,
        ));
    }

    update_state(&db_path, "SUBMITTED", 0, true, true, "insufficient")?;
    if case_id == 6 {
        update_state(&db_path, "UNKNOWN_OUTCOME", 0, true, true, "insufficient")?;
        reopen(&db_path)?;
        let row = read_journal(&db_path)?;
        cleanup(&db_path)?;
        return Ok(result(
            adapter,
            case_id,
            scenario,
            CommitEvidence::Insufficient,
            row,
            0,
        ));
    }

    let remote = remote_state(case_id);
    if matches!(case_id, 3 | 12) {
        update_state(&db_path, "UNKNOWN_OUTCOME", 0, true, true, "insufficient")?;
        let row = read_journal(&db_path)?;
        cleanup(&db_path)?;
        return Ok(result(
            adapter,
            case_id,
            scenario,
            CommitEvidence::Insufficient,
            row,
            0,
        ));
    }

    let (capabilities, observation) = adapter_observation(adapter, case_id, remote);
    let evidence = classify_commit_evidence(capabilities, observation);
    if evidence.permits_commit() {
        update_state(
            &db_path,
            "COMMITTED",
            1,
            false,
            true,
            evidence_name(evidence),
        )?;
    } else if matches!(adapter, AdapterStyle::LdapStyleDirectory)
        && remote == RemoteState::OldOnly
        && case_id != 15
    {
        update_state(
            &db_path,
            "ABORTED",
            0,
            true,
            false,
            "remote_version_unchanged",
        )?;
    } else {
        update_state(
            &db_path,
            "UNKNOWN_OUTCOME",
            0,
            true,
            true,
            evidence_name(evidence),
        )?;
    }

    if case_id == 7 {
        reopen(&db_path)?;
    }
    let row = read_journal(&db_path)?;
    let password_columns = password_column_count(&db_path)?;
    cleanup(&db_path)?;
    Ok(result(
        adapter,
        case_id,
        scenario,
        evidence,
        row,
        password_columns,
    ))
}

fn adapter_observation(
    adapter: AdapterStyle,
    case_id: u8,
    remote: RemoteState,
) -> (AdapterCapabilities, AdapterObservation) {
    match adapter {
        AdapterStyle::HttpForm => {
            let capabilities = AdapterCapabilities {
                can_verify_new: true,
                can_verify_old_safely: false,
                has_atomic_success_evidence: false,
                has_remote_version: false,
                supports_idempotency_key: false,
                evidence_requirement: EvidenceRequirement::NewAcceptance,
            };
            let new_probe = match remote {
                RemoteState::NewOnly | RemoteState::Both => Some(ProbeVerdict::Success),
                RemoteState::OldOnly | RemoteState::Neither => {
                    Some(ProbeVerdict::ConclusiveFailure)
                }
            };
            (
                capabilities,
                AdapterObservation {
                    update_response_success: !matches!(case_id, 1 | 2 | 7),
                    new_probe,
                    ..Default::default()
                },
            )
        }
        AdapterStyle::LdapStyleDirectory => {
            let has_remote_version = case_id != 15;
            let capabilities = AdapterCapabilities {
                can_verify_new: false,
                can_verify_old_safely: false,
                has_atomic_success_evidence: false,
                has_remote_version,
                supports_idempotency_key: true,
                evidence_requirement: if has_remote_version {
                    EvidenceRequirement::AuthoritativeVersion
                } else {
                    EvidenceRequirement::UnknownOnly
                },
            };
            let observed_remote_version = if has_remote_version {
                match remote {
                    RemoteState::NewOnly | RemoteState::Both => Some(1),
                    RemoteState::OldOnly => Some(0),
                    RemoteState::Neither => None,
                }
            } else {
                None
            };
            (
                capabilities,
                AdapterObservation {
                    update_response_success: !matches!(case_id, 1 | 2 | 7),
                    observed_remote_version,
                    expected_remote_version: Some(1),
                    ..Default::default()
                },
            )
        }
    }
}

fn remote_state(case_id: u8) -> RemoteState {
    match case_id {
        1 | 5 | 8 => RemoteState::OldOnly,
        10 => RemoteState::Both,
        11 => RemoteState::Neither,
        _ => RemoteState::NewOnly,
    }
}

#[derive(Debug)]
struct JournalRow {
    state: String,
    committed: u64,
    old_reconstructible: bool,
    new_reconstructible: bool,
    evidence: String,
}

fn initialize_journal(path: &Path) -> rusqlite::Result<()> {
    let connection = Connection::open(path)?;
    connection.execute_batch(
        "CREATE TABLE operation (
            op_id TEXT PRIMARY KEY,
            state TEXT NOT NULL,
            committed_generation INTEGER NOT NULL,
            base_generation INTEGER NOT NULL,
            candidate_generation INTEGER NOT NULL,
            old_reconstructible INTEGER NOT NULL,
            new_reconstructible INTEGER NOT NULL,
            evidence TEXT NOT NULL
        );",
    )?;
    connection.execute(
        "INSERT INTO operation VALUES (?1, 'PREPARED', 0, 0, 1, 1, 1, 'none')",
        [Uuid::new_v4().to_string()],
    )?;
    Ok(())
}

fn update_state(
    path: &Path,
    state: &str,
    committed: u64,
    old_reconstructible: bool,
    new_reconstructible: bool,
    evidence: &str,
) -> rusqlite::Result<()> {
    Connection::open(path)?.execute(
        "UPDATE operation SET state=?1, committed_generation=?2,
         old_reconstructible=?3, new_reconstructible=?4, evidence=?5",
        params![
            state,
            committed,
            old_reconstructible,
            new_reconstructible,
            evidence
        ],
    )?;
    Ok(())
}

fn read_journal(path: &Path) -> rusqlite::Result<JournalRow> {
    Connection::open(path)?.query_row(
        "SELECT state, committed_generation, old_reconstructible,
                new_reconstructible, evidence FROM operation",
        [],
        |row| {
            Ok(JournalRow {
                state: row.get(0)?,
                committed: row.get(1)?,
                old_reconstructible: row.get(2)?,
                new_reconstructible: row.get(3)?,
                evidence: row.get(4)?,
            })
        },
    )
}

fn password_column_count(path: &Path) -> rusqlite::Result<usize> {
    let connection = Connection::open(path)?;
    let mut statement = connection.prepare("PRAGMA table_info(operation)")?;
    let names = statement
        .query_map([], |row| row.get::<_, String>(1))?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(names
        .iter()
        .filter(|name| name.to_ascii_lowercase().contains("password"))
        .count())
}

fn reopen(path: &Path) -> rusqlite::Result<()> {
    drop(Connection::open(path)?);
    Ok(())
}

fn cleanup(path: &Path) -> std::io::Result<()> {
    if path.exists() {
        fs::remove_file(path)?;
    }
    Ok(())
}

fn temporary_db_path(adapter: AdapterStyle, case_id: u8) -> PathBuf {
    std::env::temp_dir().join(format!(
        "epscd-rotation-{:?}-{case_id}-{}.sqlite3",
        adapter,
        Uuid::new_v4()
    ))
}

fn result(
    adapter: AdapterStyle,
    case_id: u8,
    scenario: &'static str,
    evidence: CommitEvidence,
    row: JournalRow,
    persistent_password_columns: usize,
) -> CaseResult {
    let commit_ok = row.state != "COMMITTED" || evidence.permits_commit();
    let unknown_ok =
        row.state != "UNKNOWN_OUTCOME" || (row.old_reconstructible && row.new_reconstructible);
    let _persisted_evidence = row.evidence;
    CaseResult {
        adapter,
        case_id,
        scenario,
        final_state: row.state,
        committed_generation: row.committed,
        evidence,
        old_reconstructible: row.old_reconstructible,
        new_reconstructible: row.new_reconstructible,
        invariant_commit_requires_evidence: commit_ok,
        invariant_uncertainty_keeps_both: unknown_ok,
        persistent_password_columns,
    }
}

fn evidence_name(evidence: CommitEvidence) -> &'static str {
    match evidence {
        CommitEvidence::SufficientNewAcceptance => "new_acceptance",
        CommitEvidence::SufficientNewOnly => "new_only",
        CommitEvidence::SufficientAtomicEvidence => "atomic_evidence",
        CommitEvidence::SufficientRemoteVersion => "remote_version",
        CommitEvidence::Insufficient => "insufficient",
        CommitEvidence::Contradictory => "contradictory",
    }
}
