use crate::error::{KeylessPassError, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct FreshnessAnchor {
    pub vault_id: Uuid,
    pub root_generation: u64,
    pub cdr_epoch: u64,
    pub operation_log_digest: String,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum FreshnessStatus {
    Current,
    NeedsPublish,
    RollbackDetected,
    ForkDetected,
    OfflineReadOnly,
    LocalOnlyUnanchored,
}

pub trait FreshnessService: Send + Sync {
    fn read(&self, vault_id: Uuid) -> Result<Option<FreshnessAnchor>>;
    fn compare_and_set(&self, expected_epoch: Option<u64>, next: FreshnessAnchor) -> Result<()>;
}

#[derive(Default)]
pub struct InMemoryFreshnessService {
    anchors: Mutex<HashMap<Uuid, FreshnessAnchor>>,
}

impl FreshnessService for InMemoryFreshnessService {
    fn read(&self, vault_id: Uuid) -> Result<Option<FreshnessAnchor>> {
        Ok(self
            .anchors
            .lock()
            .map_err(|_| {
                KeylessPassError::Integrity("freshness service lock poisoned".to_string())
            })?
            .get(&vault_id)
            .cloned())
    }

    fn compare_and_set(&self, expected_epoch: Option<u64>, next: FreshnessAnchor) -> Result<()> {
        let mut anchors = self.anchors.lock().map_err(|_| {
            KeylessPassError::Integrity("freshness service lock poisoned".to_string())
        })?;
        let current = anchors.get(&next.vault_id);
        if current.map(|anchor| anchor.cdr_epoch) != expected_epoch {
            return Err(KeylessPassError::Integrity(
                "freshness compare-and-set conflict".to_string(),
            ));
        }
        if let Some(current) = current {
            if next.root_generation < current.root_generation || next.cdr_epoch <= current.cdr_epoch
            {
                return Err(KeylessPassError::Integrity(
                    "freshness anchor update is not monotonic".to_string(),
                ));
            }
        }
        anchors.insert(next.vault_id, next);
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct SqliteFreshnessService {
    path: PathBuf,
}

impl SqliteFreshnessService {
    pub fn new(path: impl AsRef<Path>) -> Result<Self> {
        let service = Self {
            path: path.as_ref().to_path_buf(),
        };
        if let Some(parent) = service.path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        service.connection()?.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS freshness_anchors (
                vault_id TEXT PRIMARY KEY,
                cdr_epoch INTEGER NOT NULL,
                payload_json TEXT NOT NULL
            );
            "#,
        )?;
        Ok(service)
    }

    fn connection(&self) -> Result<rusqlite::Connection> {
        Ok(rusqlite::Connection::open(&self.path)?)
    }
}

impl FreshnessService for SqliteFreshnessService {
    fn read(&self, vault_id: Uuid) -> Result<Option<FreshnessAnchor>> {
        use rusqlite::OptionalExtension;
        let connection = self.connection()?;
        let payload: Option<String> = connection
            .query_row(
                "SELECT payload_json FROM freshness_anchors WHERE vault_id = ?1",
                [vault_id.to_string()],
                |row| row.get(0),
            )
            .optional()?;
        payload
            .map(|value| serde_json::from_str(&value).map_err(KeylessPassError::from))
            .transpose()
    }

    fn compare_and_set(&self, expected_epoch: Option<u64>, next: FreshnessAnchor) -> Result<()> {
        use rusqlite::{params, OptionalExtension, TransactionBehavior};
        let mut connection = self.connection()?;
        let transaction = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
        let payload: Option<String> = transaction
            .query_row(
                "SELECT payload_json FROM freshness_anchors WHERE vault_id = ?1",
                [next.vault_id.to_string()],
                |row| row.get(0),
            )
            .optional()?;
        let current = payload
            .as_deref()
            .map(serde_json::from_str::<FreshnessAnchor>)
            .transpose()?;
        validate_compare_and_set(current.as_ref(), expected_epoch, &next)?;
        transaction.execute(
            r#"
            INSERT INTO freshness_anchors (vault_id, cdr_epoch, payload_json)
            VALUES (?1, ?2, ?3)
            ON CONFLICT(vault_id) DO UPDATE SET
                cdr_epoch = excluded.cdr_epoch,
                payload_json = excluded.payload_json
            "#,
            params![
                next.vault_id.to_string(),
                next.cdr_epoch,
                serde_json::to_string(&next)?
            ],
        )?;
        transaction.commit()?;
        Ok(())
    }
}

fn validate_compare_and_set(
    current: Option<&FreshnessAnchor>,
    expected_epoch: Option<u64>,
    next: &FreshnessAnchor,
) -> Result<()> {
    if current.map(|anchor| anchor.cdr_epoch) != expected_epoch {
        return Err(KeylessPassError::Integrity(
            "freshness compare-and-set conflict".to_string(),
        ));
    }
    if let Some(current) = current {
        if next.root_generation < current.root_generation || next.cdr_epoch <= current.cdr_epoch {
            return Err(KeylessPassError::Integrity(
                "freshness anchor update is not monotonic".to_string(),
            ));
        }
    }
    Ok(())
}

pub fn evaluate_freshness(
    local: &FreshnessAnchor,
    anchored: Option<&FreshnessAnchor>,
    enterprise_anchored: bool,
    service_available: bool,
) -> FreshnessStatus {
    if !enterprise_anchored {
        return FreshnessStatus::LocalOnlyUnanchored;
    }
    if !service_available {
        return FreshnessStatus::OfflineReadOnly;
    }
    let Some(anchored) = anchored else {
        return FreshnessStatus::NeedsPublish;
    };
    if local.vault_id != anchored.vault_id {
        return FreshnessStatus::ForkDetected;
    }
    if local.root_generation < anchored.root_generation || local.cdr_epoch < anchored.cdr_epoch {
        return FreshnessStatus::RollbackDetected;
    }
    if local.root_generation > anchored.root_generation || local.cdr_epoch > anchored.cdr_epoch {
        return FreshnessStatus::NeedsPublish;
    }
    if local.operation_log_digest != anchored.operation_log_digest {
        return FreshnessStatus::ForkDetected;
    }
    FreshnessStatus::Current
}

#[cfg(test)]
mod tests {
    use super::*;

    fn anchor(vault_id: Uuid, root: u64, epoch: u64, digest: &str) -> FreshnessAnchor {
        FreshnessAnchor {
            vault_id,
            root_generation: root,
            cdr_epoch: epoch,
            operation_log_digest: digest.to_string(),
            updated_at: Utc::now(),
        }
    }

    #[test]
    fn detects_rollback_fork_and_offline_degradation() {
        let vault = Uuid::new_v4();
        let remote = anchor(vault, 2, 10, "new");
        assert_eq!(
            evaluate_freshness(&anchor(vault, 1, 9, "old"), Some(&remote), true, true),
            FreshnessStatus::RollbackDetected
        );
        assert_eq!(
            evaluate_freshness(&anchor(vault, 2, 10, "fork"), Some(&remote), true, true),
            FreshnessStatus::ForkDetected
        );
        assert_eq!(
            evaluate_freshness(&remote, Some(&remote), true, false),
            FreshnessStatus::OfflineReadOnly
        );
        assert_eq!(
            evaluate_freshness(&remote, None, false, false),
            FreshnessStatus::LocalOnlyUnanchored
        );
    }

    #[test]
    fn compare_and_set_rejects_replay_and_concurrent_writers() {
        let service = InMemoryFreshnessService::default();
        let vault = Uuid::new_v4();
        service
            .compare_and_set(None, anchor(vault, 1, 1, "one"))
            .unwrap();
        assert!(service
            .compare_and_set(None, anchor(vault, 1, 2, "conflict"))
            .is_err());
        assert!(service
            .compare_and_set(Some(1), anchor(vault, 1, 1, "replay"))
            .is_err());
        service
            .compare_and_set(Some(1), anchor(vault, 1, 2, "two"))
            .unwrap();
    }

    #[test]
    fn sqlite_compare_and_set_survives_restart() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("freshness.sqlite3");
        let vault = Uuid::new_v4();
        let service = SqliteFreshnessService::new(&path).unwrap();
        service
            .compare_and_set(None, anchor(vault, 1, 1, "one"))
            .unwrap();
        drop(service);

        let reopened = SqliteFreshnessService::new(&path).unwrap();
        assert_eq!(reopened.read(vault).unwrap().unwrap().cdr_epoch, 1);
        assert!(reopened
            .compare_and_set(None, anchor(vault, 1, 2, "stale-writer"))
            .is_err());
        reopened
            .compare_and_set(Some(1), anchor(vault, 1, 2, "two"))
            .unwrap();
        assert_eq!(reopened.read(vault).unwrap().unwrap().cdr_epoch, 2);
    }
}
