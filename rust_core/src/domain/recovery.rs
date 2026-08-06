use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub const RECOVERY_METADATA_SCHEMA_VERSION: u32 = 1;

fn default_recovery_metadata_schema_version() -> u32 {
    RECOVERY_METADATA_SCHEMA_VERSION
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecoveryMetadata {
    #[serde(default = "default_recovery_metadata_schema_version")]
    pub schema_version: u32,
    pub recovery_model: String,
    pub recovery_fragment_index: u8,
    pub encrypted_fragment: String,
    pub fragment_mac: String,
    pub generation: u64,
    pub refreshed_at: DateTime<Utc>,
}

impl RecoveryMetadata {
    pub fn new(
        recovery_fragment_index: u8,
        encrypted_fragment: String,
        fragment_mac: String,
        generation: u64,
    ) -> Self {
        Self {
            schema_version: RECOVERY_METADATA_SCHEMA_VERSION,
            recovery_model: "2-of-3-local".to_string(),
            recovery_fragment_index,
            encrypted_fragment,
            fragment_mac,
            generation,
            refreshed_at: Utc::now(),
        }
    }
}

pub const SHARE_ENVELOPE_SCHEMA_VERSION: u32 = 3;
pub const RECOVERY_SCHEME_VERSION: u32 = 1;
pub const RECOVERY_CRYPTO_SUITE_VERSION: u32 = 1;
pub const RECOVERY_PHRASE_ENCODING_VERSION: u32 = 1;
pub const RECOVERY_THRESHOLD: u8 = 2;
pub const RECOVERY_SHARE_COUNT: u8 = 3;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RecoveryFactorType {
    Recovery,
    ManagedComputer,
    Usb,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ShareEnvelope {
    pub schema_version: u32,
    pub scheme_version: u32,
    pub crypto_suite_version: u32,
    pub vault_id: Uuid,
    pub root_generation: u64,
    pub share_set_id: Uuid,
    pub share_index: u8,
    pub threshold: u8,
    pub share_count: u8,
    pub factor_type: RecoveryFactorType,
    pub factor_id: String,
    pub factor_generation: u64,
    pub created_at: DateTime<Utc>,
    pub share_data: String,
    pub encoding_version: u32,
    pub metadata_mac: String,
}

impl ShareEnvelope {
    pub fn canonical_mac_payload(&self) -> crate::error::Result<Vec<u8>> {
        let mut copy = self.clone();
        copy.metadata_mac.clear();
        Ok(serde_json_canonicalizer::to_vec(&copy)?)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RecoveryManifest {
    pub schema_version: u32,
    pub scheme_version: u32,
    pub crypto_suite_version: u32,
    pub vault_id: Uuid,
    pub root_generation: u64,
    pub share_set_id: Uuid,
    pub threshold: u8,
    pub share_count: u8,
    pub committed_at: DateTime<Utc>,
    pub key_confirmation_value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecoveryShareSet {
    pub recovery_phrase: String,
    pub managed_computer: ShareEnvelope,
    pub usb: ShareEnvelope,
    pub manifest: RecoveryManifest,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SuccessfulRecoveryPair {
    pub left: RecoveryFactorType,
    pub right: RecoveryFactorType,
}

pub struct RecoveryAttemptReport {
    pub root_key: [u8; 32],
    pub successful_pairs: Vec<SuccessfulRecoveryPair>,
    pub suspected_damaged_factor: Option<RecoveryFactorType>,
}
