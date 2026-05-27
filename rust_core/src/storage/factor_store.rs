use crate::crypto::{b64_decode, b64_encode, kdf, mac};
use crate::domain::{
    FactorPackage, LocalFactorPayload, PackageType, RecoveryMetadata,
    FACTOR_PACKAGE_SCHEMA_VERSION, FACTOR_PAYLOAD_SCHEMA_VERSION,
};
use crate::error::{KeylessPassError, Result};
use crate::platform::PlatformFactorProvider;
use crate::storage::{read_json, write_json_private};
use std::path::Path;
use uuid::Uuid;

pub fn create_local_factor_package(
    provider: &dyn PlatformFactorProvider,
    user_id: Uuid,
    device_id: &str,
    platform: &str,
    payload: &LocalFactorPayload,
) -> Result<FactorPackage> {
    if payload.user_id != user_id || payload.device_id != device_id {
        return Err(KeylessPassError::Integrity(
            "local factor payload identity mismatch".to_string(),
        ));
    }
    let plaintext = serde_json::to_vec(payload)?;
    let mut package = FactorPackage::new(
        PackageType::Local,
        user_id,
        device_id,
        platform,
        payload.salt_c.clone(),
        b64_encode(&plaintext),
        "",
        "",
    );
    let f_c = local_factor_from_payload(provider, &package, payload)?;
    package.package_mac =
        mac::hmac_sha256_base64(&mac::package_mac_key(&f_c), &package.mac_payload()?)?;
    Ok(package)
}

pub fn write_local_factor_package(path: &Path, package: &FactorPackage) -> Result<()> {
    write_json_private(path, package)
}

pub fn read_local_factor_package(path: &Path) -> Result<FactorPackage> {
    if !path.exists() {
        return Err(KeylessPassError::MissingFactor(
            "local factor package not found".to_string(),
        ));
    }
    read_json(path)
}

pub fn load_local_factor_payload(
    provider: &dyn PlatformFactorProvider,
    path: &Path,
) -> Result<(FactorPackage, LocalFactorPayload)> {
    let package = read_local_factor_package(path)?;
    if package.package_type != PackageType::Local {
        return Err(KeylessPassError::Validation(
            "factor package is not local".to_string(),
        ));
    }
    if package.schema_version < FACTOR_PACKAGE_SCHEMA_VERSION {
        return Err(KeylessPassError::Validation(
            "legacy factor package stores master-key payload and does not support strict pairwise-wrapper recovery; please migrate with the old mnemonic available.".to_string(),
        ));
    }
    let plaintext = b64_decode(&package.encrypted_payload)?;
    let payload: LocalFactorPayload = serde_json::from_slice(&plaintext)?;
    if payload.schema_version != FACTOR_PAYLOAD_SCHEMA_VERSION {
        return Err(KeylessPassError::Validation(
            "unsupported local factor payload schema version".to_string(),
        ));
    }
    if payload.device_id != package.device_id {
        return Err(KeylessPassError::Integrity(
            "local factor device id mismatch".to_string(),
        ));
    }
    if payload.user_id != package.user_id {
        return Err(KeylessPassError::Integrity(
            "local factor user id mismatch".to_string(),
        ));
    }
    if payload.salt_c != package.kdf_salt {
        return Err(KeylessPassError::Integrity(
            "local factor salt mismatch".to_string(),
        ));
    }
    let f_c = local_factor_from_payload(provider, &package, &payload)?;
    let expected = mac::hmac_sha256_base64(&mac::package_mac_key(&f_c), &package.mac_payload()?)?;
    if !mac::constant_time_eq_b64(&expected, &package.package_mac)? {
        return Err(KeylessPassError::Integrity(
            "local package MAC mismatch".to_string(),
        ));
    }
    Ok((package, payload))
}

fn local_factor_from_payload(
    provider: &dyn PlatformFactorProvider,
    package: &FactorPackage,
    payload: &LocalFactorPayload,
) -> Result<[u8; 32]> {
    let device_secret = provider.get_or_create_device_secret()?;
    let salt_c = b64_decode(&payload.salt_c)?;
    kdf::derive_platform_factor(
        device_secret.expose(),
        &package.device_id,
        &package.user_id,
        &salt_c,
    )
}

pub fn write_recovery_metadata(path: &Path, metadata: &RecoveryMetadata) -> Result<()> {
    write_json_private(path, metadata)
}

pub fn read_recovery_metadata(path: &Path) -> Result<RecoveryMetadata> {
    read_json(path)
}
