use crate::crypto::{b64_decode, b64_encode, mac};
use crate::domain::{FactorPackage, LocalFactorPayload, PackageType, RecoveryMetadata};
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
    let plaintext = serde_json::to_vec(payload)?;
    let protected = provider.protect_local_package(&plaintext)?;
    let mut package = FactorPackage::new(
        PackageType::Local,
        user_id,
        device_id,
        platform,
        payload.mnemonic_salt.clone(),
        b64_encode(&protected),
        "",
        "",
    );
    let master_key = b64_decode(&payload.k_master)?;
    package.package_mac =
        mac::hmac_sha256_base64(&mac::package_mac_key(&master_key), &package.mac_payload()?)?;
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
    let protected = b64_decode(&package.encrypted_payload)?;
    let plaintext = provider.unprotect_local_package(&protected)?;
    let payload: LocalFactorPayload = serde_json::from_slice(&plaintext)?;
    let master_key = b64_decode(&payload.k_master)?;
    let expected =
        mac::hmac_sha256_base64(&mac::package_mac_key(&master_key), &package.mac_payload()?)?;
    let legacy_expected = mac::hmac_sha256_base64(
        &mac::package_mac_key(&master_key),
        &package.legacy_mac_payload()?,
    )?;
    if !mac::constant_time_eq_b64(&expected, &package.package_mac)?
        && !mac::constant_time_eq_b64(&legacy_expected, &package.package_mac)?
    {
        return Err(KeylessPassError::Integrity(
            "local package MAC mismatch".to_string(),
        ));
    }
    Ok((package, payload))
}

pub fn write_recovery_metadata(path: &Path, metadata: &RecoveryMetadata) -> Result<()> {
    write_json_private(path, metadata)
}

pub fn read_recovery_metadata(path: &Path) -> Result<RecoveryMetadata> {
    read_json(path)
}
