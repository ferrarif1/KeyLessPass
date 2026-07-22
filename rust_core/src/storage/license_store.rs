use crate::domain::SignedLicenseEnvelope;
use crate::error::Result;
use crate::platform::PlatformFactorProvider;
use crate::storage::{read_json, write_json_private, StoragePaths};
use std::fs;
use std::path::PathBuf;
use uuid::Uuid;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LicenseSecurityState {
    pub schema_version: u32,
    pub max_entitlement_serial: u64,
    pub latest_bundle_issued_at: String,
    pub max_seen_time: String,
}

#[derive(Debug, Clone)]
pub struct LicenseStore {
    root: PathBuf,
    commercial_device_id_path: PathBuf,
    license_envelope_path: PathBuf,
    security_state_path: PathBuf,
    history_marker_path: PathBuf,
}

impl LicenseStore {
    pub fn new(paths: &StoragePaths) -> Self {
        let root = paths.app_dir.join("license");
        Self {
            commercial_device_id_path: root.join("commercial-device-id"),
            license_envelope_path: root.join("license-envelope.json"),
            security_state_path: root.join("security-state-v2.bin"),
            history_marker_path: paths.app_dir.join(".license-history-v2.bin"),
            root,
        }
    }

    pub fn read_or_create_commercial_device_id(&self) -> Result<String> {
        if self.commercial_device_id_path.exists() {
            let value = fs::read_to_string(&self.commercial_device_id_path)?;
            let trimmed = value.trim();
            if !trimmed.is_empty() {
                return Ok(trimmed.to_string());
            }
        }
        fs::create_dir_all(&self.root)?;
        let value = Uuid::new_v4().to_string();
        crate::platform::fallback::write_private_file(
            &self.commercial_device_id_path,
            value.as_bytes(),
        )?;
        Ok(value)
    }

    pub fn read_license_envelope(&self) -> Result<Option<SignedLicenseEnvelope>> {
        if !self.license_envelope_path.exists() {
            return Ok(None);
        }
        Ok(Some(read_json(&self.license_envelope_path)?))
    }

    pub fn write_license_envelope(&self, envelope: &SignedLicenseEnvelope) -> Result<()> {
        fs::create_dir_all(&self.root)?;
        write_json_private(&self.license_envelope_path, envelope)
    }

    pub fn clear_license(&self) -> Result<()> {
        if self.license_envelope_path.exists() {
            fs::remove_file(&self.license_envelope_path)?;
        }
        Ok(())
    }

    pub fn read_security_state(
        &self,
        provider: &dyn PlatformFactorProvider,
    ) -> Result<Option<LicenseSecurityState>> {
        if !self.security_state_path.is_file() {
            return Ok(None);
        }
        let plaintext = provider.unprotect_local_package(&fs::read(&self.security_state_path)?)?;
        Ok(Some(serde_json::from_slice(&plaintext)?))
    }

    pub fn write_security_state(
        &self,
        provider: &dyn PlatformFactorProvider,
        state: &LicenseSecurityState,
    ) -> Result<()> {
        let protected = provider.protect_local_package(&serde_json::to_vec(state)?)?;
        crate::platform::fallback::write_private_file(&self.security_state_path, &protected)
    }

    pub fn has_license_history(&self, provider: &dyn PlatformFactorProvider) -> Result<bool> {
        if !self.history_marker_path.is_file() {
            return Ok(false);
        }
        let plaintext = provider.unprotect_local_package(&fs::read(&self.history_marker_path)?)?;
        Ok(plaintext == b"KeyLessPass/license-history/v2")
    }

    pub fn write_license_history(&self, provider: &dyn PlatformFactorProvider) -> Result<()> {
        let protected = provider.protect_local_package(b"KeyLessPass/license-history/v2")?;
        crate::platform::fallback::write_private_file(&self.history_marker_path, &protected)
    }
}
