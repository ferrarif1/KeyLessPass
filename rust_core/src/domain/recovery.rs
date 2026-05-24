use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

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
