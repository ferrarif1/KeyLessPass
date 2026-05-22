use crate::domain::{CredentialDescriptionRecord, CredentialState};
use crate::error::{KeylessPassError, Result};
use rusqlite::{params, Connection, OptionalExtension};
use std::path::{Path, PathBuf};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct CdrStore {
    path: PathBuf,
}

impl CdrStore {
    pub fn new(path: impl AsRef<Path>) -> Self {
        Self {
            path: path.as_ref().to_path_buf(),
        }
    }

    pub fn init(&self) -> Result<()> {
        if let Some(parent) = self.path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let conn = self.connection()?;
        conn.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS cdr_records (
                record_id TEXT NOT NULL,
                version INTEGER NOT NULL,
                record_seq INTEGER NOT NULL,
                state TEXT NOT NULL,
                display_name TEXT NOT NULL,
                service_hint TEXT NOT NULL,
                account_hint TEXT NOT NULL,
                payload_json TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                PRIMARY KEY(record_id, version)
            );
            CREATE INDEX IF NOT EXISTS idx_cdr_seq ON cdr_records(record_seq);
            CREATE INDEX IF NOT EXISTS idx_cdr_state ON cdr_records(state);
            "#,
        )?;
        Ok(())
    }

    pub fn insert(&self, record: &CredentialDescriptionRecord) -> Result<()> {
        let conn = self.connection()?;
        conn.execute(
            r#"
            INSERT INTO cdr_records (
                record_id, version, record_seq, state, display_name, service_hint,
                account_hint, payload_json, updated_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
            "#,
            params![
                record.record_id.to_string(),
                record.version,
                record.record_seq,
                state_to_str(&record.state),
                record.display_name,
                record.service_hint,
                record.account_hint,
                serde_json::to_string(record)?,
                record.updated_at.to_rfc3339(),
            ],
        )?;
        Ok(())
    }

    pub fn update(&self, record: &CredentialDescriptionRecord) -> Result<()> {
        let conn = self.connection()?;
        let affected = conn.execute(
            r#"
            UPDATE cdr_records
            SET state = ?3,
                display_name = ?4,
                service_hint = ?5,
                account_hint = ?6,
                payload_json = ?7,
                updated_at = ?8
            WHERE record_id = ?1 AND version = ?2
            "#,
            params![
                record.record_id.to_string(),
                record.version,
                state_to_str(&record.state),
                record.display_name,
                record.service_hint,
                record.account_hint,
                serde_json::to_string(record)?,
                record.updated_at.to_rfc3339(),
            ],
        )?;
        if affected == 0 {
            return Err(KeylessPassError::Validation(
                "CDR record/version not found".to_string(),
            ));
        }
        Ok(())
    }

    pub fn list_all(&self) -> Result<Vec<CredentialDescriptionRecord>> {
        let conn = self.connection()?;
        let mut stmt = conn.prepare(
            "SELECT payload_json FROM cdr_records ORDER BY record_seq ASC, version DESC",
        )?;
        let rows = stmt.query_map([], |row| {
            let payload: String = row.get(0)?;
            Ok(payload)
        })?;
        let mut records = Vec::new();
        for row in rows {
            records.push(serde_json::from_str(&row?)?);
        }
        Ok(records)
    }

    pub fn list_latest_visible(&self) -> Result<Vec<CredentialDescriptionRecord>> {
        let records = self.list_all()?;
        let mut latest = Vec::new();
        let mut seen = std::collections::HashSet::new();
        for record in records {
            if record.state == CredentialState::Retired {
                continue;
            }
            if seen.insert(record.record_id) {
                latest.push(record);
            }
        }
        latest.sort_by_key(|record| record.record_seq);
        Ok(latest)
    }

    pub fn get(
        &self,
        record_id: Uuid,
        version: Option<u32>,
    ) -> Result<CredentialDescriptionRecord> {
        let conn = self.connection()?;
        let payload: Option<String> = if let Some(version) = version {
            conn.query_row(
                "SELECT payload_json FROM cdr_records WHERE record_id = ?1 AND version = ?2",
                params![record_id.to_string(), version],
                |row| row.get(0),
            )
            .optional()?
        } else {
            conn.query_row(
                r#"
                SELECT payload_json FROM cdr_records
                WHERE record_id = ?1 AND state != 'retired'
                ORDER BY version DESC LIMIT 1
                "#,
                params![record_id.to_string()],
                |row| row.get(0),
            )
            .optional()?
        };
        let payload = payload.ok_or_else(|| {
            KeylessPassError::Validation("CDR record/version not found".to_string())
        })?;
        Ok(serde_json::from_str(&payload)?)
    }

    pub fn max_record_seq(&self) -> Result<u64> {
        let conn = self.connection()?;
        let max: Option<u64> =
            conn.query_row("SELECT MAX(record_seq) FROM cdr_records", [], |row| {
                row.get(0)
            })?;
        Ok(max.unwrap_or(0))
    }

    #[cfg(test)]
    pub fn corrupt_mac_for_test(&self, record_id: Uuid, version: u32) -> Result<()> {
        let mut record = self.get(record_id, Some(version))?;
        record.mac_tag = crate::crypto::b64_encode(&[0_u8; 32]);
        let conn = self.connection()?;
        conn.execute(
            "UPDATE cdr_records SET payload_json = ?3 WHERE record_id = ?1 AND version = ?2",
            params![
                record_id.to_string(),
                version,
                serde_json::to_string(&record)?
            ],
        )?;
        Ok(())
    }

    fn connection(&self) -> Result<Connection> {
        Ok(Connection::open(&self.path)?)
    }
}

fn state_to_str(state: &CredentialState) -> &'static str {
    match state {
        CredentialState::Active => "active",
        CredentialState::Retired => "retired",
        CredentialState::PendingRotation => "pending_rotation",
    }
}
