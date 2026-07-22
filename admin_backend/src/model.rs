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
pub struct OrganizationRecord {
    pub id: String,
    pub license_id: String,
    pub activation_code: String,
    pub name: String,
    pub plan: String,
    pub max_seats: u32,
    pub valid_from: String,
    pub valid_until: String,
    pub features: Vec<String>,
    pub offline_grace_days: u32,
    pub allowed_major_versions: Vec<u32>,
    pub issuer: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeviceRecord {
    pub id: String,
    pub organization_id: String,
    pub request_id: String,
    pub commercial_device_id: String,
    pub device_fingerprint: String,
    pub platform: String,
    pub app_version: String,
    pub build_channel: String,
    pub seat_label: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GrantRecord {
    pub id: String,
    pub grant_id: String,
    pub bundle_id: String,
    pub organization_id: String,
    pub device_id: String,
    pub commercial_device_id: String,
    pub seat_label: String,
    pub valid_until: String,
    pub revoked: bool,
    pub created_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub revoked_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BundleRecord {
    pub id: String,
    pub bundle_id: String,
    pub organization_id: String,
    pub license_id: String,
    pub device_count: u32,
    pub revoked_count: u32,
    pub valid_until: String,
    pub issued_at: String,
    pub envelope_json: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateOrganizationRequest {
    #[serde(default)]
    pub organization_id: Option<String>,
    #[serde(default)]
    pub activation_code: Option<String>,
    pub name: String,
    #[serde(default)]
    pub plan: Option<String>,
    #[serde(default)]
    pub max_seats: Option<u32>,
    #[serde(default)]
    pub valid_days: Option<i64>,
    #[serde(default)]
    pub valid_until: Option<String>,
    #[serde(default)]
    pub features: Vec<String>,
    #[serde(default)]
    pub offline_grace_days: Option<u32>,
    #[serde(default)]
    pub allowed_major_versions: Vec<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportDeviceRequestBody {
    pub request_json: String,
    #[serde(default)]
    pub organization_id: Option<String>,
    #[serde(default)]
    pub seat_label: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IssueBundleRequest {
    pub organization_id: String,
    #[serde(default)]
    pub device_ids: Vec<String>,
    #[serde(default)]
    pub valid_days: Option<i64>,
    #[serde(default)]
    pub valid_until: Option<String>,
    #[serde(default)]
    pub include_revocations: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActivateLicenseRequest {
    pub activation_code: String,
    pub request_json: String,
    #[serde(default)]
    pub seat_label: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuditRecord {
    pub id: String,
    pub actor: String,
    pub role: String,
    pub action: String,
    pub target: String,
    pub created_at: String,
    pub details_json: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BulkImportResult {
    pub imported: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AdminStatus {
    pub service: String,
    pub key_id: String,
    pub public_key_b64: String,
    pub public_key_b64url: String,
    pub database_path: String,
    pub organization_count: u32,
    pub device_count: u32,
    pub bundle_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AdminSnapshot {
    pub status: AdminStatus,
    pub organizations: Vec<OrganizationRecord>,
    pub devices: Vec<DeviceRecord>,
    pub grants: Vec<GrantRecord>,
    pub bundles: Vec<BundleRecord>,
    pub audit_log: Vec<AuditRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiMessage {
    pub message: String,
}
