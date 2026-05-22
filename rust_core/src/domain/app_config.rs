use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppConfig {
    pub app_version: String,
    pub user_id: Uuid,
    pub platform: String,
    pub device_id: String,
    pub cdr_store_path: PathBuf,
    pub local_factor_path: PathBuf,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl AppConfig {
    pub fn new(
        app_version: impl Into<String>,
        user_id: Uuid,
        platform: impl Into<String>,
        device_id: impl Into<String>,
        cdr_store_path: PathBuf,
        local_factor_path: PathBuf,
    ) -> Self {
        let now = Utc::now();
        Self {
            app_version: app_version.into(),
            user_id,
            platform: platform.into(),
            device_id: device_id.into(),
            cdr_store_path,
            local_factor_path,
            created_at: now,
            updated_at: now,
        }
    }
}
