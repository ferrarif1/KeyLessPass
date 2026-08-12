//! Durable ASTER Root-Epoch migration journal.
//!
//! Only public descriptors and evidence classes are persisted. Password values
//! are deliberately absent from the schema.

use crate::research::aster::{AsterError, Result};
use rusqlite::{params, Connection, OptionalExtension, TransactionBehavior};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::path::Path;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "camelCase")]
pub struct Descriptor {
    pub root_epoch: u64,
    pub generation: u64,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum MigrationState {
    Committed,
    Prepared,
    Submitted,
    Verifying,
    UnknownOutcome,
    Aborted,
}

impl MigrationState {
    fn as_str(self) -> &'static str {
        match self {
            Self::Committed => "COMMITTED",
            Self::Prepared => "PREPARED",
            Self::Submitted => "SUBMITTED",
            Self::Verifying => "VERIFYING",
            Self::UnknownOutcome => "UNKNOWN_OUTCOME",
            Self::Aborted => "ABORTED",
        }
    }

    fn parse(value: &str) -> Result<Self> {
        match value {
            "COMMITTED" => Ok(Self::Committed),
            "PREPARED" => Ok(Self::Prepared),
            "SUBMITTED" => Ok(Self::Submitted),
            "VERIFYING" => Ok(Self::Verifying),
            "UNKNOWN_OUTCOME" => Ok(Self::UnknownOutcome),
            "ABORTED" => Ok(Self::Aborted),
            _ => Err(AsterError::Migration(format!(
                "unknown journal state {value}"
            ))),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceClass {
    NewOnly,
    OldOnly,
    Both,
    Neither,
    Contradictory,
    Unavailable,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct JournalRecord {
    pub record_id: String,
    pub state: MigrationState,
    pub committed: Descriptor,
    pub old: Option<Descriptor>,
    pub candidate: Option<Descriptor>,
    pub last_evidence: Option<EvidenceClass>,
}

impl JournalRecord {
    pub fn old_reconstructible(&self) -> bool {
        self.old.is_some()
            || matches!(
                self.state,
                MigrationState::Committed | MigrationState::Aborted
            )
    }

    pub fn candidate_reconstructible(&self) -> bool {
        self.candidate.is_some() || self.state == MigrationState::Committed
    }
}

#[derive(Debug)]
pub struct MigrationJournal {
    connection: Connection,
}

impl MigrationJournal {
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        let connection = Connection::open(path)?;
        connection.execute_batch(
            "PRAGMA journal_mode=WAL;
             PRAGMA synchronous=FULL;
             CREATE TABLE IF NOT EXISTS aster_record (
               record_id TEXT PRIMARY KEY,
               state TEXT NOT NULL,
               committed_epoch INTEGER NOT NULL,
               committed_generation INTEGER NOT NULL,
               old_epoch INTEGER,
               old_generation INTEGER,
               candidate_epoch INTEGER,
               candidate_generation INTEGER,
               last_evidence TEXT
             );
             CREATE TABLE IF NOT EXISTS aster_history (
               record_id TEXT NOT NULL,
               root_epoch INTEGER NOT NULL,
               generation INTEGER NOT NULL,
               required INTEGER NOT NULL,
               PRIMARY KEY(record_id, root_epoch, generation)
             );",
        )?;
        Ok(Self { connection })
    }

    pub fn initialize(&self, record_id: &str, descriptor: Descriptor) -> Result<()> {
        self.connection.execute(
            "INSERT INTO aster_record(
               record_id,state,committed_epoch,committed_generation
             ) VALUES (?1,'COMMITTED',?2,?3)",
            params![record_id, descriptor.root_epoch, descriptor.generation],
        )?;
        Ok(())
    }

    /// Durably records both descriptors before any remote request can be sent.
    pub fn prepare(&mut self, record_id: &str, candidate: Descriptor) -> Result<()> {
        let current = self.load(record_id)?;
        if current.state != MigrationState::Committed {
            return Err(AsterError::Migration(
                "prepare requires committed state".to_string(),
            ));
        }
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        tx.execute(
            "UPDATE aster_record SET
               state='PREPARED',
               old_epoch=committed_epoch,
               old_generation=committed_generation,
               candidate_epoch=?2,
               candidate_generation=?3,
               last_evidence=NULL
             WHERE record_id=?1",
            params![record_id, candidate.root_epoch, candidate.generation],
        )?;
        tx.commit()?;
        Ok(())
    }

    pub fn mark_submitted(&self, record_id: &str) -> Result<()> {
        self.transition(
            record_id,
            &[MigrationState::Prepared],
            MigrationState::Submitted,
        )
    }

    pub fn mark_verifying(&self, record_id: &str) -> Result<()> {
        self.transition(
            record_id,
            &[MigrationState::Submitted, MigrationState::UnknownOutcome],
            MigrationState::Verifying,
        )
    }

    pub fn apply_evidence(&mut self, record_id: &str, evidence: EvidenceClass) -> Result<()> {
        let current = self.load(record_id)?;
        if !matches!(
            current.state,
            MigrationState::Prepared
                | MigrationState::Submitted
                | MigrationState::Verifying
                | MigrationState::UnknownOutcome
        ) {
            return Err(AsterError::Migration(
                "evidence requires an unresolved operation".to_string(),
            ));
        }
        let candidate = current
            .candidate
            .ok_or_else(|| AsterError::Migration("candidate descriptor missing".into()))?;
        let evidence_text = evidence_name(evidence);
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        match evidence {
            EvidenceClass::NewOnly => {
                let old = current
                    .old
                    .ok_or_else(|| AsterError::Migration("old descriptor missing".into()))?;
                tx.execute(
                    "INSERT OR IGNORE INTO aster_history(
                       record_id,root_epoch,generation,required
                     ) VALUES (?1,?2,?3,1)",
                    params![record_id, old.root_epoch, old.generation],
                )?;
                tx.execute(
                    "UPDATE aster_record SET
                       state='COMMITTED', committed_epoch=?2,
                       committed_generation=?3, old_epoch=NULL,
                       old_generation=NULL, candidate_epoch=NULL,
                       candidate_generation=NULL, last_evidence=?4
                     WHERE record_id=?1",
                    params![
                        record_id,
                        candidate.root_epoch,
                        candidate.generation,
                        evidence_text
                    ],
                )?;
            }
            EvidenceClass::OldOnly => {
                tx.execute(
                    "UPDATE aster_record SET
                       state='ABORTED', old_epoch=NULL, old_generation=NULL,
                       candidate_epoch=NULL, candidate_generation=NULL,
                       last_evidence=?2
                     WHERE record_id=?1",
                    params![record_id, evidence_text],
                )?;
            }
            EvidenceClass::Both
            | EvidenceClass::Neither
            | EvidenceClass::Contradictory
            | EvidenceClass::Unavailable => {
                tx.execute(
                    "UPDATE aster_record SET state='UNKNOWN_OUTCOME',last_evidence=?2
                     WHERE record_id=?1",
                    params![record_id, evidence_text],
                )?;
            }
        }
        tx.commit()?;
        Ok(())
    }

    pub fn normalize_aborted(&self, record_id: &str) -> Result<()> {
        self.transition(
            record_id,
            &[MigrationState::Aborted],
            MigrationState::Committed,
        )
    }

    pub fn set_history_required(
        &self,
        record_id: &str,
        descriptor: Descriptor,
        required: bool,
    ) -> Result<()> {
        self.connection.execute(
            "INSERT INTO aster_history(record_id,root_epoch,generation,required)
             VALUES (?1,?2,?3,?4)
             ON CONFLICT(record_id,root_epoch,generation)
             DO UPDATE SET required=excluded.required",
            params![
                record_id,
                descriptor.root_epoch,
                descriptor.generation,
                i64::from(required)
            ],
        )?;
        Ok(())
    }

    pub fn referenced_epochs(&self) -> Result<BTreeSet<u64>> {
        let mut epochs = BTreeSet::new();
        let mut statement = self
            .connection
            .prepare("SELECT committed_epoch,old_epoch,candidate_epoch FROM aster_record")?;
        let rows = statement.query_map([], |row| {
            Ok((
                row.get::<_, u64>(0)?,
                row.get::<_, Option<u64>>(1)?,
                row.get::<_, Option<u64>>(2)?,
            ))
        })?;
        for row in rows {
            let (committed, old, candidate) = row?;
            epochs.insert(committed);
            epochs.extend(old);
            epochs.extend(candidate);
        }
        let mut history = self
            .connection
            .prepare("SELECT DISTINCT root_epoch FROM aster_history WHERE required=1")?;
        for row in history.query_map([], |row| row.get::<_, u64>(0))? {
            epochs.insert(row?);
        }
        Ok(epochs)
    }

    pub fn can_retire_epoch(&self, epoch: u64) -> Result<bool> {
        Ok(!self.referenced_epochs()?.contains(&epoch))
    }

    pub fn load(&self, record_id: &str) -> Result<JournalRecord> {
        self.connection
            .query_row(
                "SELECT state,committed_epoch,committed_generation,
                        old_epoch,old_generation,candidate_epoch,candidate_generation,
                        last_evidence
                 FROM aster_record WHERE record_id=?1",
                params![record_id],
                |row| {
                    let state: String = row.get(0)?;
                    let evidence: Option<String> = row.get(7)?;
                    Ok((
                        state,
                        Descriptor {
                            root_epoch: row.get(1)?,
                            generation: row.get(2)?,
                        },
                        pair(row.get(3)?, row.get(4)?),
                        pair(row.get(5)?, row.get(6)?),
                        evidence,
                    ))
                },
            )
            .optional()?
            .ok_or_else(|| AsterError::Migration("record not found".into()))
            .and_then(|(state, committed, old, candidate, evidence)| {
                Ok(JournalRecord {
                    record_id: record_id.to_string(),
                    state: MigrationState::parse(&state)?,
                    committed,
                    old,
                    candidate,
                    last_evidence: evidence.as_deref().map(parse_evidence).transpose()?,
                })
            })
    }

    pub fn schema_contains_password_column(&self) -> Result<bool> {
        let mut statement = self.connection.prepare("PRAGMA table_info(aster_record)")?;
        let columns = statement.query_map([], |row| row.get::<_, String>(1))?;
        for column in columns {
            if column?.to_ascii_lowercase().contains("password") {
                return Ok(true);
            }
        }
        Ok(false)
    }

    fn transition(
        &self,
        record_id: &str,
        from: &[MigrationState],
        to: MigrationState,
    ) -> Result<()> {
        let current = self.load(record_id)?;
        if !from.contains(&current.state) {
            return Err(AsterError::Migration(format!(
                "invalid transition from {:?} to {:?}",
                current.state, to
            )));
        }
        self.connection.execute(
            "UPDATE aster_record SET state=?2 WHERE record_id=?1",
            params![record_id, to.as_str()],
        )?;
        Ok(())
    }
}

fn pair(epoch: Option<u64>, generation: Option<u64>) -> Option<Descriptor> {
    match (epoch, generation) {
        (Some(root_epoch), Some(generation)) => Some(Descriptor {
            root_epoch,
            generation,
        }),
        _ => None,
    }
}

fn evidence_name(evidence: EvidenceClass) -> &'static str {
    match evidence {
        EvidenceClass::NewOnly => "new_only",
        EvidenceClass::OldOnly => "old_only",
        EvidenceClass::Both => "both",
        EvidenceClass::Neither => "neither",
        EvidenceClass::Contradictory => "contradictory",
        EvidenceClass::Unavailable => "unavailable",
    }
}

fn parse_evidence(value: &str) -> Result<EvidenceClass> {
    match value {
        "new_only" => Ok(EvidenceClass::NewOnly),
        "old_only" => Ok(EvidenceClass::OldOnly),
        "both" => Ok(EvidenceClass::Both),
        "neither" => Ok(EvidenceClass::Neither),
        "contradictory" => Ok(EvidenceClass::Contradictory),
        "unavailable" => Ok(EvidenceClass::Unavailable),
        _ => Err(AsterError::Migration(format!("unknown evidence {value}"))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn crash_reopen_preserves_dual_descriptors() {
        let directory = tempdir().unwrap();
        let path = directory.path().join("journal.sqlite");
        {
            let mut journal = MigrationJournal::open(&path).unwrap();
            journal
                .initialize(
                    "r1",
                    Descriptor {
                        root_epoch: 1,
                        generation: 7,
                    },
                )
                .unwrap();
            journal
                .prepare(
                    "r1",
                    Descriptor {
                        root_epoch: 2,
                        generation: 0,
                    },
                )
                .unwrap();
            journal.mark_submitted("r1").unwrap();
        }
        let mut reopened = MigrationJournal::open(&path).unwrap();
        let record = reopened.load("r1").unwrap();
        assert_eq!(record.state, MigrationState::Submitted);
        assert_eq!(record.old.unwrap().root_epoch, 1);
        assert_eq!(record.candidate.unwrap().root_epoch, 2);
        reopened
            .apply_evidence("r1", EvidenceClass::Unavailable)
            .unwrap();
        let unknown = reopened.load("r1").unwrap();
        assert_eq!(unknown.state, MigrationState::UnknownOutcome);
        assert!(unknown.old_reconstructible());
        assert!(unknown.candidate_reconstructible());
        assert!(!reopened.can_retire_epoch(1).unwrap());
        assert!(!reopened.can_retire_epoch(2).unwrap());
    }

    #[test]
    fn conclusive_commit_advances_only_to_candidate() {
        let mut journal = MigrationJournal::open(":memory:").unwrap();
        journal
            .initialize(
                "r1",
                Descriptor {
                    root_epoch: 1,
                    generation: 7,
                },
            )
            .unwrap();
        journal
            .prepare(
                "r1",
                Descriptor {
                    root_epoch: 2,
                    generation: 3,
                },
            )
            .unwrap();
        journal
            .apply_evidence("r1", EvidenceClass::NewOnly)
            .unwrap();
        let record = journal.load("r1").unwrap();
        assert_eq!(record.state, MigrationState::Committed);
        assert_eq!(record.committed.root_epoch, 2);
        assert_eq!(record.committed.generation, 3);
        assert!(record.old.is_none() && record.candidate.is_none());
        assert!(!journal.can_retire_epoch(1).unwrap());
        journal
            .set_history_required(
                "r1",
                Descriptor {
                    root_epoch: 1,
                    generation: 7,
                },
                false,
            )
            .unwrap();
        assert!(journal.can_retire_epoch(1).unwrap());
    }

    #[test]
    fn transport_ambiguity_never_commits_and_schema_has_no_password() {
        for evidence in [
            EvidenceClass::Both,
            EvidenceClass::Neither,
            EvidenceClass::Contradictory,
            EvidenceClass::Unavailable,
        ] {
            let mut journal = MigrationJournal::open(":memory:").unwrap();
            journal
                .initialize(
                    "r1",
                    Descriptor {
                        root_epoch: 1,
                        generation: 0,
                    },
                )
                .unwrap();
            journal
                .prepare(
                    "r1",
                    Descriptor {
                        root_epoch: 2,
                        generation: 0,
                    },
                )
                .unwrap();
            journal.apply_evidence("r1", evidence).unwrap();
            assert_eq!(
                journal.load("r1").unwrap().state,
                MigrationState::UnknownOutcome
            );
            assert!(!journal.schema_contains_password_column().unwrap());
        }
    }
}
