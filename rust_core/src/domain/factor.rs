use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::domain::PasswordDerivationAlgorithm;

pub const FACTOR_PACKAGE_SCHEMA_VERSION: u32 = 1;
pub const FACTOR_PACKAGE_VERSION: u32 = 1;

fn default_factor_package_schema_version() -> u32 {
    FACTOR_PACKAGE_SCHEMA_VERSION
}

fn default_factor_package_version() -> u32 {
    FACTOR_PACKAGE_VERSION
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PackageType {
    Local,
    Usb,
    Recovery,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FactorPackage {
    #[serde(default = "default_factor_package_schema_version")]
    pub schema_version: u32,
    #[serde(default = "default_factor_package_version")]
    pub package_version: u32,
    pub package_id: Uuid,
    pub package_type: PackageType,
    pub user_id: Uuid,
    pub device_id: String,
    pub platform: String,
    pub kdf_salt: String,
    pub encrypted_payload: String,
    pub nonce: String,
    pub aead_tag: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub package_mac: String,
}

impl FactorPackage {
    pub fn new(
        package_type: PackageType,
        user_id: Uuid,
        device_id: impl Into<String>,
        platform: impl Into<String>,
        kdf_salt: impl Into<String>,
        encrypted_payload: impl Into<String>,
        nonce: impl Into<String>,
        aead_tag: impl Into<String>,
    ) -> Self {
        let now = Utc::now();
        Self {
            schema_version: FACTOR_PACKAGE_SCHEMA_VERSION,
            package_version: FACTOR_PACKAGE_VERSION,
            package_id: Uuid::new_v4(),
            package_type,
            user_id,
            device_id: device_id.into(),
            platform: platform.into(),
            kdf_salt: kdf_salt.into(),
            encrypted_payload: encrypted_payload.into(),
            nonce: nonce.into(),
            aead_tag: aead_tag.into(),
            created_at: now,
            updated_at: now,
            package_mac: String::new(),
        }
    }

    pub fn mac_payload(&self) -> crate::error::Result<Vec<u8>> {
        let mut copy = self.clone();
        copy.package_mac.clear();
        Ok(serde_json::to_vec(&copy)?)
    }

    pub fn legacy_mac_payload(&self) -> crate::error::Result<Vec<u8>> {
        #[derive(Serialize)]
        #[serde(rename_all = "camelCase")]
        struct LegacyFactorPackage<'a> {
            package_id: &'a Uuid,
            package_type: &'a PackageType,
            user_id: &'a Uuid,
            device_id: &'a str,
            platform: &'a str,
            kdf_salt: &'a str,
            encrypted_payload: &'a str,
            nonce: &'a str,
            aead_tag: &'a str,
            created_at: &'a DateTime<Utc>,
            updated_at: &'a DateTime<Utc>,
            package_mac: &'a str,
        }

        let legacy = LegacyFactorPackage {
            package_id: &self.package_id,
            package_type: &self.package_type,
            user_id: &self.user_id,
            device_id: &self.device_id,
            platform: &self.platform,
            kdf_salt: &self.kdf_salt,
            encrypted_payload: &self.encrypted_payload,
            nonce: &self.nonce,
            aead_tag: &self.aead_tag,
            created_at: &self.created_at,
            updated_at: &self.updated_at,
            package_mac: "",
        };
        Ok(serde_json::to_vec(&legacy)?)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LocalFactorPayload {
    pub k_master: String,
    pub device_secret: String,
    pub usb_secret: String,
    pub mnemonic_salt: String,
    #[serde(default)]
    pub password_derivation_algorithm: PasswordDerivationAlgorithm,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mnemonic_verifier: Option<String>,
    pub recovery_generation: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UsbFactorPayload {
    pub k_master: String,
    pub usb_secret: String,
    pub device_secret: String,
    pub mnemonic_salt: String,
    #[serde(default)]
    pub password_derivation_algorithm: PasswordDerivationAlgorithm,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mnemonic_verifier: Option<String>,
    pub recovery_generation: u64,
}
