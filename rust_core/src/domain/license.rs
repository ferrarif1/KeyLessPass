use serde::{Deserialize, Serialize};

pub const LICENSE_ENVELOPE_TYPE: &str = "keylesspass-license-bundle";
pub const LICENSE_SIGNATURE_ALGORITHM: &str = "Ed25519";
pub const LICENSE_SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SignedLicenseEnvelope {
    pub schema_version: u32,
    #[serde(rename = "type")]
    pub envelope_type: String,
    pub payload: String,
    pub signature_algorithm: String,
    pub key_id: String,
    pub signature: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LicenseBundlePayload {
    pub schema_version: u32,
    pub bundle_id: String,
    pub organization_license: OrganizationLicense,
    #[serde(default)]
    pub device_grants: Vec<DeviceGrant>,
    #[serde(default)]
    pub revoked_grant_ids: Vec<String>,
    pub issued_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OrganizationLicense {
    pub schema_version: u32,
    pub license_id: String,
    pub organization_id: String,
    pub organization_name: String,
    pub plan: String,
    pub max_seats: u32,
    pub valid_from: String,
    pub valid_until: String,
    #[serde(default)]
    pub features: Vec<String>,
    #[serde(default)]
    pub offline_grace_days: u32,
    #[serde(default)]
    pub allowed_major_versions: Vec<u32>,
    pub issued_at: String,
    pub issuer: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeviceGrant {
    pub schema_version: u32,
    pub grant_id: String,
    pub license_id: String,
    pub organization_id: String,
    pub commercial_device_id: String,
    pub device_fingerprint: String,
    #[serde(default)]
    pub seat_label: String,
    pub valid_from: String,
    pub valid_until: String,
    #[serde(default)]
    pub features: Vec<String>,
    #[serde(default)]
    pub offline_grace_days: u32,
    pub issued_at: String,
    pub issuer: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeviceAuthorizationRequest {
    pub schema_version: u32,
    pub request_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub organization_id: Option<String>,
    pub commercial_device_id: String,
    pub device_fingerprint: String,
    pub platform: String,
    pub app_version: String,
    pub build_channel: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seat_label: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LicenseStatus {
    pub status: String,
    pub authorized: bool,
    pub commercial_device_id: String,
    pub device_fingerprint: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub organization_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub organization_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub license_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub grant_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub plan: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seat_label: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub valid_until: Option<String>,
    #[serde(default)]
    pub features: Vec<String>,
    pub message: String,
}
