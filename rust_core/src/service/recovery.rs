use crate::crypto::{b64_decode, kdf, mac, recovery as crypto_recovery};
use crate::domain::{AppConfig, LocalFactorPayload, UsbFactorPayload};
use crate::error::Result;
use crate::platform::{current_platform_provider, PlatformFactorProvider};
use crate::storage::{
    create_local_factor_package, create_usb_factor_package, load_local_factor_payload,
    load_usb_factor_payload, read_config, write_config, write_local_factor_package,
    write_recovery_metadata, write_usb_factor_package, StoragePaths,
};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecoverUsbRequest {
    pub mnemonic: String,
    pub usb_path: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecoverLocalRequest {
    pub mnemonic: String,
    pub usb_path: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RecoveryResponse {
    pub generation: u64,
    pub path: PathBuf,
}

pub fn recover_usb(request: RecoverUsbRequest) -> std::result::Result<RecoveryResponse, String> {
    let paths = StoragePaths::default().map_err(String::from)?;
    let provider = current_platform_provider(&paths.app_dir);
    recover_usb_with_provider(&paths, provider.as_ref(), request).map_err(String::from)
}

pub fn recover_local(
    request: RecoverLocalRequest,
) -> std::result::Result<RecoveryResponse, String> {
    let paths = StoragePaths::default().map_err(String::from)?;
    let provider = current_platform_provider(&paths.app_dir);
    recover_local_with_provider(&paths, provider.as_ref(), request).map_err(String::from)
}

pub fn recover_usb_with_provider(
    paths: &StoragePaths,
    provider: &dyn PlatformFactorProvider,
    request: RecoverUsbRequest,
) -> Result<RecoveryResponse> {
    let config = read_config(paths)?;
    let (_, local_payload) = load_local_factor_payload(provider, &config.local_factor_path)?;
    verify_mnemonic_against_payload(
        &request.mnemonic,
        config.user_id,
        &local_payload.mnemonic_salt,
        local_payload.mnemonic_verifier.as_deref(),
    )?;
    let generation = local_payload.recovery_generation + 1;
    let usb_payload = UsbFactorPayload {
        k_master: local_payload.k_master.clone(),
        usb_secret: local_payload.usb_secret.clone(),
        device_secret: local_payload.device_secret.clone(),
        mnemonic_salt: local_payload.mnemonic_salt.clone(),
        mnemonic_verifier: local_payload.mnemonic_verifier.clone(),
        recovery_generation: generation,
    };
    let package = create_usb_factor_package(
        &request.mnemonic,
        config.user_id,
        &config.device_id,
        &config.platform,
        &usb_payload,
    )?;
    let path = write_usb_factor_package(&request.usb_path, &package)?;
    let master_key = b64_decode(&local_payload.k_master)?;
    let recovery = crypto_recovery::build_recovery_metadata(&master_key, generation)?;
    write_recovery_metadata(&paths.recovery_path, &recovery)?;
    Ok(RecoveryResponse { generation, path })
}

pub fn recover_local_with_provider(
    paths: &StoragePaths,
    provider: &dyn PlatformFactorProvider,
    request: RecoverLocalRequest,
) -> Result<RecoveryResponse> {
    paths.ensure()?;
    let (usb_package, usb_payload) = load_usb_factor_payload(&request.mnemonic, &request.usb_path)?;
    let generation = usb_payload.recovery_generation + 1;
    let mnemonic_verifier = if let Some(verifier) = usb_payload.mnemonic_verifier.clone() {
        Some(verifier)
    } else {
        let salt = b64_decode(&usb_payload.mnemonic_salt)?;
        let f_m = kdf::derive_mnemonic_factor(&request.mnemonic, &usb_package.user_id, &salt)?;
        Some(kdf::derive_mnemonic_verifier(&f_m)?)
    };
    let local_payload = LocalFactorPayload {
        k_master: usb_payload.k_master.clone(),
        device_secret: usb_payload.device_secret.clone(),
        usb_secret: usb_payload.usb_secret.clone(),
        mnemonic_salt: usb_payload.mnemonic_salt.clone(),
        mnemonic_verifier,
        recovery_generation: generation,
    };
    let package = create_local_factor_package(
        provider,
        usb_package.user_id,
        &usb_package.device_id,
        &usb_package.platform,
        &local_payload,
    )?;
    write_local_factor_package(&paths.local_factor_path, &package)?;

    let config = AppConfig::new(
        env!("CARGO_PKG_VERSION"),
        usb_package.user_id,
        usb_package.platform,
        usb_package.device_id,
        paths.db_path.clone(),
        paths.local_factor_path.clone(),
    );
    write_config(paths, &config)?;

    let master_key = b64_decode(&usb_payload.k_master)?;
    let recovery = crypto_recovery::build_recovery_metadata(&master_key, generation)?;
    write_recovery_metadata(&paths.recovery_path, &recovery)?;
    Ok(RecoveryResponse {
        generation,
        path: paths.local_factor_path.clone(),
    })
}

fn verify_mnemonic_against_payload(
    mnemonic: &str,
    user_id: uuid::Uuid,
    mnemonic_salt_b64: &str,
    verifier_b64: Option<&str>,
) -> Result<()> {
    let Some(verifier_b64) = verifier_b64 else {
        return Ok(());
    };
    let salt = b64_decode(mnemonic_salt_b64)?;
    let f_m = kdf::derive_mnemonic_factor(mnemonic, &user_id, &salt)?;
    let actual = kdf::derive_mnemonic_verifier(&f_m)?;
    if !mac::constant_time_eq_b64(&actual, verifier_b64)? {
        return Err(crate::error::KeylessPassError::MissingFactor(
            "mnemonic phrase did not pass recovery verification".to_string(),
        ));
    }
    Ok(())
}
