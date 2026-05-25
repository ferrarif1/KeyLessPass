use crate::domain::{AppConfig, RecoveryMetadata};
use crate::platform::{current_platform_provider, current_security_status, PlatformSecurityStatus};
use crate::storage::{read_config, read_recovery_metadata, StoragePaths};
use serde::{Deserialize, Serialize};
use std::fs;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppStatus {
    pub enrolled: bool,
    pub config: Option<AppConfig>,
    pub security_status: PlatformSecurityStatus,
    pub recovery: Option<RecoveryMetadata>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResetApplicationDataRequest {
    pub confirmation: String,
}

pub fn get_app_status() -> std::result::Result<AppStatus, String> {
    let paths = StoragePaths::default().map_err(String::from)?;
    let provider = current_platform_provider(&paths.app_dir);
    let security_status = current_security_status(provider.as_ref());
    let config = read_config(&paths).ok();
    let recovery = read_recovery_metadata(&paths.recovery_path).ok();
    Ok(AppStatus {
        enrolled: config.is_some(),
        config,
        security_status,
        recovery,
    })
}

pub fn get_security_status() -> std::result::Result<PlatformSecurityStatus, String> {
    let paths = StoragePaths::default().map_err(String::from)?;
    let provider = current_platform_provider(&paths.app_dir);
    Ok(current_security_status(provider.as_ref()))
}

pub fn reset_application_data(
    request: ResetApplicationDataRequest,
) -> std::result::Result<(), String> {
    if request.confirmation.trim() != "RESET" {
        return Err("reset confirmation did not match".to_string());
    }

    let paths = StoragePaths::default().map_err(String::from)?;
    if paths.app_dir.exists() {
        fs::remove_dir_all(&paths.app_dir)
            .map_err(|_| "failed to reset local application data".to_string())?;
    }

    #[cfg(target_os = "macos")]
    crate::platform::macos::delete_keychain_password();

    Ok(())
}
