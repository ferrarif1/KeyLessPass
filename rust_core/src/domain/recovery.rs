use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecoveryMetadata {
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
            recovery_model: "2-of-3-local".to_string(),
            recovery_fragment_index,
            encrypted_fragment,
            fragment_mac,
            generation,
            refreshed_at: Utc::now(),
        }
    }
}
