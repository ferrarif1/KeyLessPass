use crate::crypto::signing::{b64url_decode, LicenseVerifier};
use crate::domain::{
    DeviceAuthorizationRequest, DeviceGrant, LicenseBundlePayload, LicenseStatus,
    SignedLicenseEnvelope, LICENSE_ENVELOPE_TYPE, LICENSE_SCHEMA_VERSION,
    LICENSE_SIGNATURE_ALGORITHM,
};
use crate::error::{KeylessPassError, Result};
use crate::platform::{current_platform_provider, PlatformFactorProvider};
use crate::storage::{LicenseStore, StoragePaths};
use chrono::{DateTime, Duration, Utc};
use serde::Deserialize;
use sha2::{Digest, Sha256};
use uuid::Uuid;

const DEFAULT_LICENSE_KEY_ID: &str = "keylesspass-license-2026-q3";
const EVALUATION_LICENSE_PUBLIC_KEY_B64: &str = "QDNS+Maa3BNsG+4jFlo2UFNkcMr00G3+HOxc7I7lOCI=";

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportDeviceAuthorizationRequest {
    #[serde(default)]
    pub organization_id: Option<String>,
    #[serde(default)]
    pub seat_label: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportLicenseBundleRequest {
    pub bundle_json: String,
}

pub fn get_license_status() -> std::result::Result<LicenseStatus, String> {
    let paths = StoragePaths::default().map_err(String::from)?;
    let provider = current_platform_provider(&paths.app_dir);
    get_license_status_at(&paths, provider.as_ref(), &default_license_verifier())
        .map_err(String::from)
}

pub fn export_device_authorization_request(
    request: ExportDeviceAuthorizationRequest,
) -> std::result::Result<DeviceAuthorizationRequest, String> {
    let paths = StoragePaths::default().map_err(String::from)?;
    let provider = current_platform_provider(&paths.app_dir);
    export_device_authorization_request_at(&paths, provider.as_ref(), request).map_err(String::from)
}

pub fn import_license_bundle(
    request: ImportLicenseBundleRequest,
) -> std::result::Result<LicenseStatus, String> {
    let paths = StoragePaths::default().map_err(String::from)?;
    let provider = current_platform_provider(&paths.app_dir);
    import_license_bundle_at(
        &paths,
        provider.as_ref(),
        &default_license_verifier(),
        request,
    )
    .map_err(String::from)
}

pub fn clear_license() -> std::result::Result<LicenseStatus, String> {
    let paths = StoragePaths::default().map_err(String::from)?;
    let provider = current_platform_provider(&paths.app_dir);
    clear_license_at(&paths, provider.as_ref()).map_err(String::from)
}

pub fn require_license_feature(feature: &str) -> std::result::Result<(), String> {
    if !commercial_enforcement_enabled() {
        return Ok(());
    }
    let paths = StoragePaths::default().map_err(String::from)?;
    let provider = current_platform_provider(&paths.app_dir);
    require_license_feature_at(
        &paths,
        provider.as_ref(),
        &default_license_verifier(),
        feature,
    )
    .map_err(String::from)
}

pub fn require_license_feature_at(
    paths: &StoragePaths,
    provider: &dyn PlatformFactorProvider,
    verifier: &LicenseVerifier,
    feature: &str,
) -> Result<()> {
    if !commercial_enforcement_enabled() {
        return Ok(());
    }
    let status = get_license_status_at(paths, provider, verifier)?;
    if status.authorized
        && (status.features.iter().any(|value| value == feature)
            || status
                .features
                .iter()
                .any(|value| value == "desktop-client"))
    {
        return Ok(());
    }
    Err(KeylessPassError::Validation(format!(
        "commercial license does not authorize feature: {feature}"
    )))
}

fn commercial_enforcement_enabled() -> bool {
    matches!(
        option_env!("KEYLESSPASS_REQUIRE_LICENSE"),
        Some("1") | Some("true") | Some("TRUE") | Some("yes")
    ) || matches!(
        std::env::var("KEYLESSPASS_REQUIRE_LICENSE_RUNTIME")
            .ok()
            .as_deref(),
        Some("1") | Some("true") | Some("TRUE") | Some("yes")
    )
}

pub fn get_license_status_at(
    paths: &StoragePaths,
    provider: &dyn PlatformFactorProvider,
    verifier: &LicenseVerifier,
) -> Result<LicenseStatus> {
    let context = license_context(paths, provider)?;
    let store = LicenseStore::new(paths);
    let Some(envelope) = store.read_license_envelope()? else {
        return Ok(unlicensed_status(
            context.commercial_device_id,
            context.device_fingerprint,
            "No commercial device grant has been imported.",
        ));
    };
    match verify_and_parse_bundle(&envelope, verifier)
        .and_then(|bundle| status_from_bundle(context.clone(), &bundle))
    {
        Ok(status) => Ok(status),
        Err(error) => Ok(LicenseStatus {
            status: "invalid".to_string(),
            authorized: false,
            commercial_device_id: context.commercial_device_id,
            device_fingerprint: context.device_fingerprint,
            organization_id: None,
            organization_name: None,
            license_id: None,
            grant_id: None,
            plan: None,
            seat_label: None,
            valid_until: None,
            features: vec![],
            message: format!("Stored license could not be validated: {error}"),
        }),
    }
}

pub fn export_device_authorization_request_at(
    paths: &StoragePaths,
    provider: &dyn PlatformFactorProvider,
    request: ExportDeviceAuthorizationRequest,
) -> Result<DeviceAuthorizationRequest> {
    let context = license_context(paths, provider)?;
    Ok(DeviceAuthorizationRequest {
        schema_version: LICENSE_SCHEMA_VERSION,
        request_id: Uuid::new_v4().to_string(),
        organization_id: request
            .organization_id
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty()),
        commercial_device_id: context.commercial_device_id,
        device_fingerprint: context.device_fingerprint,
        platform: std::env::consts::OS.to_string(),
        app_version: env!("CARGO_PKG_VERSION").to_string(),
        build_channel: option_env!("KEYLESSPASS_BUILD_CHANNEL")
            .unwrap_or("desktop")
            .to_string(),
        seat_label: request
            .seat_label
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty()),
        created_at: Utc::now().to_rfc3339(),
    })
}

pub fn import_license_bundle_at(
    paths: &StoragePaths,
    provider: &dyn PlatformFactorProvider,
    verifier: &LicenseVerifier,
    request: ImportLicenseBundleRequest,
) -> Result<LicenseStatus> {
    let envelope: SignedLicenseEnvelope = serde_json::from_str(&request.bundle_json)?;
    let bundle = verify_and_parse_bundle(&envelope, verifier)?;
    let status = status_from_bundle(license_context(paths, provider)?, &bundle)?;
    if status.status == "notForThisDevice" || status.status == "revoked" {
        return Err(KeylessPassError::Validation(status.message));
    }
    LicenseStore::new(paths).write_license_envelope(&envelope)?;
    Ok(status)
}

pub fn clear_license_at(
    paths: &StoragePaths,
    provider: &dyn PlatformFactorProvider,
) -> Result<LicenseStatus> {
    let context = license_context(paths, provider)?;
    LicenseStore::new(paths).clear_license()?;
    Ok(unlicensed_status(
        context.commercial_device_id,
        context.device_fingerprint,
        "Local commercial device grant has been cleared.",
    ))
}

pub fn default_license_verifier() -> LicenseVerifier {
    let key_id = option_env!("KEYLESSPASS_LICENSE_KEY_ID").unwrap_or(DEFAULT_LICENSE_KEY_ID);
    let public_key = option_env!("KEYLESSPASS_LICENSE_PUBLIC_KEY_B64")
        .unwrap_or(EVALUATION_LICENSE_PUBLIC_KEY_B64);
    LicenseVerifier::new([(key_id.to_string(), public_key.to_string())])
}

fn verify_and_parse_bundle(
    envelope: &SignedLicenseEnvelope,
    verifier: &LicenseVerifier,
) -> Result<LicenseBundlePayload> {
    if envelope.schema_version != LICENSE_SCHEMA_VERSION {
        return Err(KeylessPassError::Validation(
            "unsupported license envelope schema".to_string(),
        ));
    }
    if envelope.envelope_type != LICENSE_ENVELOPE_TYPE {
        return Err(KeylessPassError::Validation(
            "unsupported license envelope type".to_string(),
        ));
    }
    if envelope.signature_algorithm != LICENSE_SIGNATURE_ALGORITHM {
        return Err(KeylessPassError::Validation(
            "unsupported license signature algorithm".to_string(),
        ));
    }
    let payload = b64url_decode(&envelope.payload)?;
    verifier.verify(&envelope.key_id, &payload, &envelope.signature)?;
    let bundle: LicenseBundlePayload = serde_json::from_slice(&payload)?;
    if bundle.schema_version != LICENSE_SCHEMA_VERSION {
        return Err(KeylessPassError::Validation(
            "unsupported license bundle schema".to_string(),
        ));
    }
    Ok(bundle)
}

#[derive(Debug, Clone)]
struct LicenseContext {
    commercial_device_id: String,
    device_fingerprint: String,
}

fn license_context(
    paths: &StoragePaths,
    provider: &dyn PlatformFactorProvider,
) -> Result<LicenseContext> {
    let store = LicenseStore::new(paths);
    let commercial_device_id = store.read_or_create_commercial_device_id()?;
    let non_secret_device_id = provider.get_or_create_device_id()?;
    let device_fingerprint = device_fingerprint(&commercial_device_id, &non_secret_device_id);
    Ok(LicenseContext {
        commercial_device_id,
        device_fingerprint,
    })
}

fn device_fingerprint(commercial_device_id: &str, non_secret_device_id: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(b"KeyLessPass/license/device-fingerprint/v1");
    hasher.update(commercial_device_id.as_bytes());
    hasher.update([0]);
    hasher.update(non_secret_device_id.as_bytes());
    hasher.update([0]);
    hasher.update(std::env::consts::OS.as_bytes());
    hasher.update([0]);
    hasher.update(std::env::consts::ARCH.as_bytes());
    to_hex(&hasher.finalize())
}

fn status_from_bundle(
    context: LicenseContext,
    bundle: &LicenseBundlePayload,
) -> Result<LicenseStatus> {
    let grant = bundle.device_grants.iter().find(|grant| {
        grant.commercial_device_id == context.commercial_device_id
            && grant.device_fingerprint == context.device_fingerprint
    });
    let Some(grant) = grant else {
        return Ok(LicenseStatus {
            status: "notForThisDevice".to_string(),
            authorized: false,
            commercial_device_id: context.commercial_device_id,
            device_fingerprint: context.device_fingerprint,
            organization_id: Some(bundle.organization_license.organization_id.clone()),
            organization_name: Some(bundle.organization_license.organization_name.clone()),
            license_id: Some(bundle.organization_license.license_id.clone()),
            grant_id: None,
            plan: Some(bundle.organization_license.plan.clone()),
            seat_label: None,
            valid_until: Some(bundle.organization_license.valid_until.clone()),
            features: bundle.organization_license.features.clone(),
            message: "License bundle does not contain a grant for this device.".to_string(),
        });
    };
    if bundle
        .revoked_grant_ids
        .iter()
        .any(|id| id == &grant.grant_id)
    {
        return Ok(status_for_grant(
            context,
            bundle,
            grant,
            "revoked",
            false,
            "This device grant has been revoked.",
        ));
    }
    if grant.license_id != bundle.organization_license.license_id
        || grant.organization_id != bundle.organization_license.organization_id
    {
        return Err(KeylessPassError::Integrity(
            "device grant does not match organization license".to_string(),
        ));
    }

    let now = Utc::now();
    let valid_from = parse_rfc3339(&grant.valid_from)?;
    let valid_until = parse_rfc3339(&grant.valid_until)?;
    let org_valid_until = parse_rfc3339(&bundle.organization_license.valid_until)?;
    let effective_until = valid_until.min(org_valid_until);
    let grace_days = grant
        .offline_grace_days
        .max(bundle.organization_license.offline_grace_days) as i64;
    if now < valid_from {
        return Ok(status_for_grant(
            context,
            bundle,
            grant,
            "notYetValid",
            false,
            "This device grant is not valid yet.",
        ));
    }
    if now <= effective_until {
        return Ok(status_for_grant(
            context,
            bundle,
            grant,
            "authorized",
            true,
            "This device is commercially authorized.",
        ));
    }
    if now <= effective_until + Duration::days(grace_days) {
        return Ok(status_for_grant(
            context,
            bundle,
            grant,
            "grace",
            true,
            "License expired, but offline grace period is active.",
        ));
    }
    Ok(status_for_grant(
        context,
        bundle,
        grant,
        "expired",
        false,
        "License and grace period have expired.",
    ))
}

fn status_for_grant(
    context: LicenseContext,
    bundle: &LicenseBundlePayload,
    grant: &DeviceGrant,
    status: &str,
    authorized: bool,
    message: &str,
) -> LicenseStatus {
    let features = if grant.features.is_empty() {
        bundle.organization_license.features.clone()
    } else {
        grant.features.clone()
    };
    LicenseStatus {
        status: status.to_string(),
        authorized,
        commercial_device_id: context.commercial_device_id,
        device_fingerprint: context.device_fingerprint,
        organization_id: Some(bundle.organization_license.organization_id.clone()),
        organization_name: Some(bundle.organization_license.organization_name.clone()),
        license_id: Some(bundle.organization_license.license_id.clone()),
        grant_id: Some(grant.grant_id.clone()),
        plan: Some(bundle.organization_license.plan.clone()),
        seat_label: if grant.seat_label.is_empty() {
            None
        } else {
            Some(grant.seat_label.clone())
        },
        valid_until: Some(grant.valid_until.clone()),
        features,
        message: message.to_string(),
    }
}

fn unlicensed_status(
    commercial_device_id: String,
    device_fingerprint: String,
    message: &str,
) -> LicenseStatus {
    LicenseStatus {
        status: "unlicensed".to_string(),
        authorized: false,
        commercial_device_id,
        device_fingerprint,
        organization_id: None,
        organization_name: None,
        license_id: None,
        grant_id: None,
        plan: None,
        seat_label: None,
        valid_until: None,
        features: vec![],
        message: message.to_string(),
    }
}

fn parse_rfc3339(value: &str) -> Result<DateTime<Utc>> {
    DateTime::parse_from_rfc3339(value)
        .map(|value| value.with_timezone(&Utc))
        .map_err(|_| KeylessPassError::Validation("invalid license date".to_string()))
}

fn to_hex(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        output.push(HEX[(byte >> 4) as usize] as char);
        output.push(HEX[(byte & 0x0f) as usize] as char);
    }
    output
}
