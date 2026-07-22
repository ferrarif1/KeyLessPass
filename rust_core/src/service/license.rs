use crate::crypto::signing::{b64url_decode, LicenseVerifier};
use crate::domain::{
    CustomerEntitlement, DeviceAuthorizationRequest, DeviceGrant, LicenseBundlePayload,
    LicenseStatus, SignedCustomerEntitlementEnvelope, SignedLicenseEnvelope,
    CUSTOMER_ENTITLEMENT_TYPE, LICENSE_ENVELOPE_TYPE, LICENSE_SCHEMA_VERSION,
    LICENSE_SIGNATURE_ALGORITHM,
};
use crate::error::{KeylessPassError, Result};
use crate::platform::{current_platform_provider, PlatformFactorProvider};
use crate::service::device_identity::DeviceIdentity;
use crate::storage::{LicenseSecurityState, LicenseStore, StoragePaths};
use chrono::{DateTime, Duration, Utc};
use hmac::{Hmac, Mac};
use serde::Deserialize;
use sha2::Sha256;
use std::{collections::BTreeMap, fs, path::PathBuf};
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
    let verifier = default_license_verifier();
    import_managed_license_if_present(&paths, provider.as_ref(), &verifier)
        .map_err(String::from)?;
    get_license_status_at(&paths, provider.as_ref(), &verifier).map_err(String::from)
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
    if status_authorizes_feature(&status, feature, current_build_channel()) {
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
    match verify_and_parse_bundle(&envelope, verifier).and_then(|bundle| {
        update_security_state(paths, provider, &bundle)?;
        status_from_bundle(context.clone(), &bundle)
    }) {
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
    let identity = DeviceIdentity::load_or_create(paths, provider)?;
    let mut authorization_request = DeviceAuthorizationRequest {
        schema_version: LICENSE_SCHEMA_VERSION,
        request_id: Uuid::new_v4().to_string(),
        organization_id: request
            .organization_id
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty()),
        commercial_device_id: context.commercial_device_id,
        device_fingerprint: context.device_fingerprint,
        device_key_id: context.device_key_id,
        device_public_key: context.device_public_key,
        device_proof: String::new(),
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
    };
    authorization_request.device_proof =
        identity.sign_b64url(&authorization_request.proof_message());
    Ok(authorization_request)
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
    update_security_state(paths, provider, &bundle)?;
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
    let mut keys = BTreeMap::new();
    // Trusted keys are build inputs. Reading this value from the process
    // environment would let a local user add their own signing key and mint a
    // valid-looking commercial license at runtime.
    if let Some(json) = option_env!("KEYLESSPASS_LICENSE_TRUSTED_KEYS_JSON") {
        if let Ok(configured) = serde_json::from_str::<BTreeMap<String, String>>(&json) {
            keys.extend(configured);
        }
    }
    keys.insert(key_id.to_string(), public_key.to_string());
    LicenseVerifier::new(keys)
}

fn import_managed_license_if_present(
    paths: &StoragePaths,
    provider: &dyn PlatformFactorProvider,
    verifier: &LicenseVerifier,
) -> Result<()> {
    let Some(path) = managed_license_path() else {
        return Ok(());
    };
    if !path.is_file() {
        return Ok(());
    }
    import_managed_license_file_at(paths, provider, verifier, &path)
}

pub(crate) fn import_managed_license_file_at(
    paths: &StoragePaths,
    provider: &dyn PlatformFactorProvider,
    verifier: &LicenseVerifier,
    path: &std::path::Path,
) -> Result<()> {
    import_license_bundle_at(
        paths,
        provider,
        verifier,
        ImportLicenseBundleRequest {
            bundle_json: fs::read_to_string(path)?,
        },
    )?;
    Ok(())
}

fn managed_license_path() -> Option<PathBuf> {
    std::env::var("KEYLESSPASS_MANAGED_LICENSE_FILE")
        .ok()
        .or_else(|| option_env!("KEYLESSPASS_MANAGED_LICENSE_FILE").map(str::to_string))
        .map(|value| PathBuf::from(value.trim()))
        .filter(|path| !path.as_os_str().is_empty())
        .or_else(|| commercial_enforcement_enabled().then(default_managed_license_path))
}

#[cfg(target_os = "macos")]
fn default_managed_license_path() -> PathBuf {
    PathBuf::from("/Library/Application Support/KeyLessPass/license-bundle.json")
}

#[cfg(target_os = "windows")]
fn default_managed_license_path() -> PathBuf {
    std::env::var("PROGRAMDATA")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from(r"C:\ProgramData"))
        .join("KeyLessPass")
        .join("license-bundle.json")
}

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
fn default_managed_license_path() -> PathBuf {
    PathBuf::from("/etc/keylesspass/license-bundle.json")
}

fn current_build_channel() -> &'static str {
    option_env!("KEYLESSPASS_BUILD_CHANNEL").unwrap_or("desktop")
}

fn current_app_major_version() -> u32 {
    option_env!("KEYLESSPASS_APP_MAJOR_VERSION")
        .and_then(|value| value.parse().ok())
        .unwrap_or(1)
}

fn status_authorizes_feature(status: &LicenseStatus, feature: &str, channel: &str) -> bool {
    if !status.authorized || !status.features.iter().any(|value| value == feature) {
        return false;
    }
    matches!(channel, "desktop" | "evaluation")
        || status
            .features
            .iter()
            .any(|value| value == &format!("channel:{channel}"))
}

fn verify_and_parse_bundle(
    envelope: &SignedLicenseEnvelope,
    vendor_verifier: &LicenseVerifier,
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
    let bundle: LicenseBundlePayload = serde_json::from_slice(&payload)?;
    if bundle.schema_version != LICENSE_SCHEMA_VERSION {
        return Err(KeylessPassError::Validation(
            "unsupported license bundle schema".to_string(),
        ));
    }
    let entitlement = verify_customer_entitlement(&bundle.customer_entitlement, vendor_verifier)?;
    let site_verifier = LicenseVerifier::new([(
        entitlement.site_key_id.clone(),
        entitlement.site_public_key.clone(),
    )]);
    site_verifier.verify(&envelope.key_id, &payload, &envelope.signature)?;
    validate_entitlement_constraints(&bundle, &entitlement)?;
    Ok(bundle)
}

fn verify_customer_entitlement(
    envelope: &SignedCustomerEntitlementEnvelope,
    vendor_verifier: &LicenseVerifier,
) -> Result<CustomerEntitlement> {
    if envelope.schema_version != LICENSE_SCHEMA_VERSION
        || envelope.envelope_type != CUSTOMER_ENTITLEMENT_TYPE
        || envelope.signature_algorithm != LICENSE_SIGNATURE_ALGORITHM
    {
        return Err(KeylessPassError::Validation(
            "customer entitlement envelope is invalid".to_string(),
        ));
    }
    let payload = b64url_decode(&envelope.payload)?;
    vendor_verifier.verify(&envelope.key_id, &payload, &envelope.signature)?;
    let entitlement: CustomerEntitlement = serde_json::from_slice(&payload)?;
    if entitlement.schema_version != LICENSE_SCHEMA_VERSION
        || entitlement.product != "KeyLessPass"
        || entitlement.max_concurrent_devices > entitlement.max_registered_devices
        || entitlement.max_offline_borrowed > entitlement.max_registered_devices
        || entitlement.authorized_device_key_ids.len() > entitlement.max_registered_devices as usize
    {
        return Err(KeylessPassError::Integrity(
            "customer entitlement payload is invalid".to_string(),
        ));
    }
    Ok(entitlement)
}

fn validate_entitlement_constraints(
    bundle: &LicenseBundlePayload,
    entitlement: &CustomerEntitlement,
) -> Result<()> {
    let organization = &bundle.organization_license;
    if organization.max_seats > entitlement.max_registered_devices
        || organization.max_seats > entitlement.max_concurrent_devices
        || organization.offline_grace_days > entitlement.max_offline_grace_days
        || organization
            .features
            .iter()
            .any(|feature| !entitlement.features.contains(feature))
        || organization
            .allowed_major_versions
            .iter()
            .any(|version| !entitlement.allowed_major_versions.contains(version))
        || bundle.device_grants.iter().any(|grant| {
            grant.offline_grace_days > entitlement.max_offline_grace_days
                || grant.features.iter().any(|feature| {
                    !entitlement.features.contains(feature)
                        || !organization.features.contains(feature)
                })
                || !entitlement
                    .authorized_device_key_ids
                    .contains(&grant.device_key_id)
        })
    {
        return Err(KeylessPassError::Integrity(
            "license bundle exceeds the vendor entitlement".to_string(),
        ));
    }
    let now = Utc::now();
    let entitlement_from = parse_rfc3339(&entitlement.valid_from)?;
    let entitlement_until = parse_rfc3339(&entitlement.valid_until)?;
    if now < entitlement_from
        || now > entitlement_until
        || parse_rfc3339(&organization.valid_until)? > entitlement_until
    {
        return Err(KeylessPassError::Validation(
            "vendor entitlement is expired or not yet valid".to_string(),
        ));
    }
    Ok(())
}

fn update_security_state(
    paths: &StoragePaths,
    provider: &dyn PlatformFactorProvider,
    bundle: &LicenseBundlePayload,
) -> Result<()> {
    let entitlement_payload = b64url_decode(&bundle.customer_entitlement.payload)?;
    let entitlement: CustomerEntitlement = serde_json::from_slice(&entitlement_payload)?;
    let now = Utc::now();
    let bundle_issued_at = parse_rfc3339(&bundle.issued_at)?;
    let store = LicenseStore::new(paths);
    let next = if let Some(current) = store.read_security_state(provider)? {
        let max_seen = parse_rfc3339(&current.max_seen_time)?;
        let latest_bundle = parse_rfc3339(&current.latest_bundle_issued_at)?;
        if now + Duration::minutes(5) < max_seen {
            return Err(KeylessPassError::Integrity(
                "system clock moved backwards beyond the allowed tolerance".to_string(),
            ));
        }
        if entitlement.entitlement_serial < current.max_entitlement_serial
            || bundle_issued_at < latest_bundle
        {
            return Err(KeylessPassError::Integrity(
                "license rollback was detected".to_string(),
            ));
        }
        LicenseSecurityState {
            schema_version: LICENSE_SCHEMA_VERSION,
            max_entitlement_serial: current
                .max_entitlement_serial
                .max(entitlement.entitlement_serial),
            latest_bundle_issued_at: bundle_issued_at.max(latest_bundle).to_rfc3339(),
            max_seen_time: now.max(max_seen).to_rfc3339(),
        }
    } else {
        LicenseSecurityState {
            schema_version: LICENSE_SCHEMA_VERSION,
            max_entitlement_serial: entitlement.entitlement_serial,
            latest_bundle_issued_at: bundle_issued_at.to_rfc3339(),
            max_seen_time: now.to_rfc3339(),
        }
    };
    store.write_security_state(provider, &next)
}

#[derive(Debug, Clone)]
struct LicenseContext {
    commercial_device_id: String,
    device_fingerprint: String,
    device_key_id: String,
    device_public_key: String,
}

fn license_context(
    paths: &StoragePaths,
    provider: &dyn PlatformFactorProvider,
) -> Result<LicenseContext> {
    let store = LicenseStore::new(paths);
    let commercial_device_id = store.read_or_create_commercial_device_id()?;
    let non_secret_device_id = provider.get_or_create_device_id()?;
    let device_secret = provider.get_or_create_device_secret()?;
    let identity = DeviceIdentity::load_or_create(paths, provider)?;
    identity.prove_possession()?;
    let device_key_id = identity.key_id();
    let device_public_key = identity.public_key_b64url();
    let device_fingerprint = device_fingerprint(
        &commercial_device_id,
        &non_secret_device_id,
        &device_key_id,
        device_secret.expose(),
    )?;
    Ok(LicenseContext {
        commercial_device_id,
        device_fingerprint,
        device_key_id,
        device_public_key,
    })
}

fn device_fingerprint(
    commercial_device_id: &str,
    non_secret_device_id: &str,
    device_key_id: &str,
    device_secret: &[u8],
) -> Result<String> {
    let mut mac = Hmac::<Sha256>::new_from_slice(device_secret)
        .map_err(|_| KeylessPassError::Crypto("invalid device binding secret".to_string()))?;
    mac.update(b"KeyLessPass/license/device-fingerprint/v2");
    mac.update(commercial_device_id.as_bytes());
    mac.update(&[0]);
    mac.update(non_secret_device_id.as_bytes());
    mac.update(&[0]);
    mac.update(device_key_id.as_bytes());
    mac.update(&[0]);
    mac.update(std::env::consts::OS.as_bytes());
    mac.update(&[0]);
    mac.update(std::env::consts::ARCH.as_bytes());
    Ok(to_hex(&mac.finalize().into_bytes()))
}

fn status_from_bundle(
    context: LicenseContext,
    bundle: &LicenseBundlePayload,
) -> Result<LicenseStatus> {
    if bundle.device_grants.len() as u32 > bundle.organization_license.max_seats {
        return Err(KeylessPassError::Integrity(
            "license bundle exceeds organization maxSeats".to_string(),
        ));
    }
    if !bundle
        .organization_license
        .allowed_major_versions
        .is_empty()
        && !bundle
            .organization_license
            .allowed_major_versions
            .contains(&current_app_major_version())
    {
        return Ok(LicenseStatus {
            status: "versionNotAllowed".to_string(),
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
            features: vec![],
            message: "License does not permit this application major version.".to_string(),
        });
    }
    let grant = bundle.device_grants.iter().find(|grant| {
        grant.commercial_device_id == context.commercial_device_id
            && grant.device_fingerprint == context.device_fingerprint
            && grant.device_key_id == context.device_key_id
            && grant.device_public_key == context.device_public_key
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
    let valid_from = parse_rfc3339(&grant.valid_from)?
        .max(parse_rfc3339(&bundle.organization_license.valid_from)?);
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

#[cfg(test)]
mod tests {
    use super::*;

    fn status(features: &[&str]) -> LicenseStatus {
        LicenseStatus {
            status: "authorized".to_string(),
            authorized: true,
            commercial_device_id: "device".to_string(),
            device_fingerprint: "fingerprint".to_string(),
            organization_id: None,
            organization_name: None,
            license_id: None,
            grant_id: None,
            plan: None,
            seat_label: None,
            valid_until: None,
            features: features.iter().map(|value| value.to_string()).collect(),
            message: String::new(),
        }
    }

    #[test]
    fn commercial_channel_requires_channel_entitlement() {
        assert!(!status_authorizes_feature(
            &status(&["desktop-client"]),
            "desktop-client",
            "commercial"
        ));
        assert!(status_authorizes_feature(
            &status(&["desktop-client", "channel:commercial"]),
            "desktop-client",
            "commercial"
        ));
        assert!(status_authorizes_feature(
            &status(&["desktop-client"]),
            "desktop-client",
            "evaluation"
        ));
        assert!(!status_authorizes_feature(
            &status(&["desktop-client"]),
            "future-premium-feature",
            "evaluation"
        ));
    }

    #[test]
    fn commercial_compile_flag_rejects_an_unlicensed_device() {
        if !commercial_enforcement_enabled() {
            return;
        }
        let dir = tempfile::tempdir().unwrap();
        let paths = StoragePaths::from_app_dir(dir.path().to_path_buf());
        let provider = crate::platform::fallback::FallbackPlatformFactorProvider::new(
            paths.app_dir.clone(),
            "commercial-guard-test",
        );
        let verifier = LicenseVerifier::new(Vec::<(String, String)>::new());
        let error =
            require_license_feature_at(&paths, &provider, &verifier, "desktop-client").unwrap_err();
        assert!(error.to_string().contains("does not authorize"));
    }
}
