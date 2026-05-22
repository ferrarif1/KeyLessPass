use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

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
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LocalFactorPayload {
    pub k_master: String,
    pub device_secret: String,
    pub usb_secret: String,
    pub mnemonic_salt: String,
    pub recovery_generation: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UsbFactorPayload {
    pub k_master: String,
    pub usb_secret: String,
    pub device_secret: String,
    pub mnemonic_salt: String,
    pub recovery_generation: u64,
}
