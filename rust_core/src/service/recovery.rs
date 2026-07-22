use crate::crypto::{b64_encode, kdf, recovery as crypto_recovery};
use crate::domain::{
    AppConfig, LocalFactorPayload, UsbFactorPayload, FACTOR_PAYLOAD_SCHEMA_VERSION, WRAP_LABEL_CU,
    WRAP_LABEL_MC, WRAP_LABEL_MU,
};
use crate::error::{KeylessPassError, Result};
use crate::platform::{current_platform_provider, PlatformFactorProvider};
use crate::service::factor_keys::{
    cu_wrap_aad, derive_mnemonic_factor_checked, load_local_context, load_usb_context,
    master_key_from_local_usb, master_key_from_mnemonic_local, master_key_from_mnemonic_usb,
    mc_wrap_aad, mu_wrap_aad, remember_master_key, wrap_master_key,
};
use crate::storage::{
    create_local_factor_package, create_usb_factor_package, read_config, verify_usb_cdr_backup,
    write_config, write_local_factor_package, write_recovery_metadata, write_usb_cdr_backup,
    write_usb_factor_package, CdrStore, StoragePaths,
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

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResetMnemonicRequest {
    pub new_mnemonic: String,
    pub usb_path: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RecoveryResponse {
    pub generation: u64,
    pub path: PathBuf,
}

pub fn recover_usb(request: RecoverUsbRequest) -> std::result::Result<RecoveryResponse, String> {
    crate::service::license::require_license_feature("desktop-client")?;
    let paths = StoragePaths::default().map_err(String::from)?;
    let provider = current_platform_provider(&paths.app_dir);
    recover_usb_with_provider(&paths, provider.as_ref(), request).map_err(String::from)
}

pub fn recover_local(
    request: RecoverLocalRequest,
) -> std::result::Result<RecoveryResponse, String> {
    crate::service::license::require_license_feature("desktop-client")?;
    let paths = StoragePaths::default().map_err(String::from)?;
    let provider = current_platform_provider(&paths.app_dir);
    recover_local_with_provider(&paths, provider.as_ref(), request).map_err(String::from)
}

pub fn reset_mnemonic(
    request: ResetMnemonicRequest,
) -> std::result::Result<RecoveryResponse, String> {
    crate::service::license::require_license_feature("desktop-client")?;
    let paths = StoragePaths::default().map_err(String::from)?;
    let provider = current_platform_provider(&paths.app_dir);
    reset_mnemonic_with_provider(&paths, provider.as_ref(), request).map_err(String::from)
}

pub fn recover_usb_with_provider(
    paths: &StoragePaths,
    provider: &dyn PlatformFactorProvider,
    request: RecoverUsbRequest,
) -> Result<RecoveryResponse> {
    crate::service::license::require_license_feature_at(
        paths,
        provider,
        &crate::service::license::default_license_verifier(),
        "desktop-client",
    )?;
    let config = read_config(paths)?;
    let local = load_local_context(provider, &config.local_factor_path)?;
    ensure_local_matches_config(&config, &local.package)?;
    let master_key = master_key_from_mnemonic_local(&request.mnemonic, &local)?;
    let f_m = derive_mnemonic_factor_checked(
        &request.mnemonic,
        &local.payload.mnemonic_salt,
        local.payload.mnemonic_verifier.as_deref(),
    )?;
    let generation = local.payload.recovery_generation + 1;
    let usb_id = uuid::Uuid::new_v4().to_string();
    let usb_secret = crate::crypto::random_bytes(32);
    let usb_salt = crate::crypto::random_bytes(16);
    let usb_salt_b64 = b64_encode(&usb_salt);
    let f_u = kdf::derive_usb_factor(&usb_secret, &usb_id, &config.user_id, &usb_salt)?;
    let w_mu = wrap_master_key(
        &master_key,
        &f_m,
        &f_u,
        WRAP_LABEL_MU,
        &mu_wrap_aad(
            config.user_id,
            &usb_id,
            &local.payload.mnemonic_salt,
            &usb_salt_b64,
        ),
    )?;
    let w_cu = wrap_master_key(
        &master_key,
        &local.f_c,
        &f_u,
        WRAP_LABEL_CU,
        &cu_wrap_aad(
            config.user_id,
            &config.device_id,
            &usb_id,
            &local.payload.salt_c,
            &usb_salt_b64,
        ),
    )?;
    let usb_payload = UsbFactorPayload {
        schema_version: FACTOR_PAYLOAD_SCHEMA_VERSION,
        user_id: config.user_id,
        usb_id,
        usb_secret: b64_encode(&usb_secret),
        salt_u: usb_salt_b64,
        mnemonic_salt: local.payload.mnemonic_salt.clone(),
        password_derivation_algorithm: local.payload.password_derivation_algorithm,
        mnemonic_verifier: local.payload.mnemonic_verifier.clone(),
        recovery_generation: generation,
        w_mu,
        w_cu: w_cu.clone(),
    };
    let package = create_usb_factor_package(
        config.user_id,
        &config.device_id,
        &config.platform,
        &usb_payload,
    )?;
    let path = write_usb_factor_package(&request.usb_path, &package)?;
    let mut updated_local_payload = local.payload.clone();
    updated_local_payload.recovery_generation = generation;
    updated_local_payload.w_cu = Some(w_cu);
    let local_package = create_local_factor_package(
        provider,
        config.user_id,
        &config.device_id,
        &config.platform,
        &updated_local_payload,
    )?;
    write_local_factor_package(&config.local_factor_path, &local_package)?;
    let store = CdrStore::new(&config.cdr_store_path);
    let records = store.list_all()?;
    write_usb_cdr_backup(&request.usb_path, config.user_id, &master_key, &records)?;
    let recovery = crypto_recovery::build_recovery_metadata(&master_key, generation)?;
    write_recovery_metadata(&paths.recovery_path, &recovery)?;
    remember_master_key(&config, &master_key)?;
    Ok(RecoveryResponse { generation, path })
}

pub fn recover_local_with_provider(
    paths: &StoragePaths,
    provider: &dyn PlatformFactorProvider,
    request: RecoverLocalRequest,
) -> Result<RecoveryResponse> {
    crate::service::license::require_license_feature_at(
        paths,
        provider,
        &crate::service::license::default_license_verifier(),
        "desktop-client",
    )?;
    paths.ensure()?;
    let usb = load_usb_context(&request.usb_path)?;
    let master_key = master_key_from_mnemonic_usb(&request.mnemonic, &usb)?;
    let f_m = derive_mnemonic_factor_checked(
        &request.mnemonic,
        &usb.payload.mnemonic_salt,
        usb.payload.mnemonic_verifier.as_deref(),
    )?;
    let device_id = provider.get_or_create_device_id()?;
    let device_secret = provider.get_or_create_device_secret()?;
    let platform = provider.platform_name();
    let device_salt = crate::crypto::random_bytes(16);
    let device_salt_b64 = b64_encode(&device_salt);
    let f_c = kdf::derive_platform_factor(
        device_secret.expose(),
        &device_id,
        &usb.package.user_id,
        &device_salt,
    )?;
    let generation = usb.payload.recovery_generation + 1;
    let w_mc = wrap_master_key(
        &master_key,
        &f_m,
        &f_c,
        WRAP_LABEL_MC,
        &mc_wrap_aad(
            usb.package.user_id,
            &device_id,
            &usb.payload.mnemonic_salt,
            &device_salt_b64,
        ),
    )?;
    let local_payload = LocalFactorPayload {
        schema_version: FACTOR_PAYLOAD_SCHEMA_VERSION,
        user_id: usb.package.user_id,
        device_id: device_id.clone(),
        salt_c: device_salt_b64.clone(),
        mnemonic_salt: usb.payload.mnemonic_salt.clone(),
        password_derivation_algorithm: usb.payload.password_derivation_algorithm,
        mnemonic_verifier: usb.payload.mnemonic_verifier.clone(),
        recovery_generation: generation,
        w_mc: w_mc.clone(),
        w_cu: None,
    };
    let w_mu = wrap_master_key(
        &master_key,
        &f_m,
        &usb.f_u,
        WRAP_LABEL_MU,
        &mu_wrap_aad(
            usb.package.user_id,
            &usb.payload.usb_id,
            &usb.payload.mnemonic_salt,
            &usb.payload.salt_u,
        ),
    )?;
    let w_cu = wrap_master_key(
        &master_key,
        &f_c,
        &usb.f_u,
        WRAP_LABEL_CU,
        &cu_wrap_aad(
            usb.package.user_id,
            &device_id,
            &usb.payload.usb_id,
            &device_salt_b64,
            &usb.payload.salt_u,
        ),
    )?;
    let local_payload = LocalFactorPayload {
        w_cu: Some(w_cu.clone()),
        ..local_payload
    };
    let package = create_local_factor_package(
        provider,
        usb.package.user_id,
        &device_id,
        &platform,
        &local_payload,
    )?;
    write_local_factor_package(&paths.local_factor_path, &package)?;
    let updated_usb_payload = UsbFactorPayload {
        schema_version: FACTOR_PAYLOAD_SCHEMA_VERSION,
        user_id: usb.payload.user_id,
        usb_id: usb.payload.usb_id.clone(),
        usb_secret: usb.payload.usb_secret.clone(),
        salt_u: usb.payload.salt_u.clone(),
        mnemonic_salt: usb.payload.mnemonic_salt.clone(),
        password_derivation_algorithm: usb.payload.password_derivation_algorithm,
        mnemonic_verifier: usb.payload.mnemonic_verifier.clone(),
        recovery_generation: generation,
        w_mu,
        w_cu,
    };
    let updated_usb_package = create_usb_factor_package(
        usb.package.user_id,
        &device_id,
        &platform,
        &updated_usb_payload,
    )?;
    write_usb_factor_package(&request.usb_path, &updated_usb_package)?;

    let config = AppConfig::new(
        env!("CARGO_PKG_VERSION"),
        usb.package.user_id,
        platform,
        device_id,
        paths.db_path.clone(),
        paths.local_factor_path.clone(),
        usb.payload.password_derivation_algorithm,
    );
    write_config(paths, &config)?;

    let recovery = crypto_recovery::build_recovery_metadata(&master_key, generation)?;
    write_recovery_metadata(&paths.recovery_path, &recovery)?;
    let store = CdrStore::new(&paths.db_path);
    match verify_usb_cdr_backup(&request.usb_path, usb.package.user_id, &master_key) {
        Ok(backup) => store.replace_all(&backup.records)?,
        Err(KeylessPassError::MissingFactor(_)) => store.init()?,
        Err(error) => return Err(error),
    }
    remember_master_key(&config, &master_key)?;
    Ok(RecoveryResponse {
        generation,
        path: paths.local_factor_path.clone(),
    })
}

pub fn reset_mnemonic_with_provider(
    paths: &StoragePaths,
    provider: &dyn PlatformFactorProvider,
    request: ResetMnemonicRequest,
) -> Result<RecoveryResponse> {
    crate::service::license::require_license_feature_at(
        paths,
        provider,
        &crate::service::license::default_license_verifier(),
        "desktop-client",
    )?;
    if request.new_mnemonic.trim().is_empty() {
        return Err(KeylessPassError::MissingFactor(
            "new mnemonic phrase is required".to_string(),
        ));
    }

    let config = read_config(paths)?;
    let local = load_local_context(provider, &config.local_factor_path)?;
    ensure_local_matches_config(&config, &local.package)?;
    let usb = load_usb_context(&request.usb_path)?;
    if usb.package.user_id != config.user_id || usb.package.device_id != config.device_id {
        return Err(KeylessPassError::Integrity(
            "USB factor package does not match this managed computer".to_string(),
        ));
    }
    let master_key = master_key_from_local_usb(&local, &usb)?;
    let generation = local
        .payload
        .recovery_generation
        .max(usb.payload.recovery_generation)
        + 1;
    let mnemonic_salt = crate::crypto::random_bytes(16);
    let mnemonic_salt_b64 = b64_encode(&mnemonic_salt);
    let f_m = kdf::derive_mnemonic_factor(&request.new_mnemonic, &mnemonic_salt)?;
    let mnemonic_verifier = kdf::derive_mnemonic_verifier(&f_m)?;
    let w_mc = wrap_master_key(
        &master_key,
        &f_m,
        &local.f_c,
        WRAP_LABEL_MC,
        &mc_wrap_aad(
            config.user_id,
            &config.device_id,
            &mnemonic_salt_b64,
            &local.payload.salt_c,
        ),
    )?;

    let updated_local_payload = LocalFactorPayload {
        schema_version: FACTOR_PAYLOAD_SCHEMA_VERSION,
        user_id: local.payload.user_id,
        device_id: local.payload.device_id.clone(),
        salt_c: local.payload.salt_c.clone(),
        mnemonic_salt: mnemonic_salt_b64.clone(),
        password_derivation_algorithm: local.payload.password_derivation_algorithm,
        mnemonic_verifier: Some(mnemonic_verifier.clone()),
        recovery_generation: generation,
        w_mc: w_mc.clone(),
        w_cu: None,
    };
    let w_mu = wrap_master_key(
        &master_key,
        &f_m,
        &usb.f_u,
        WRAP_LABEL_MU,
        &mu_wrap_aad(
            config.user_id,
            &usb.payload.usb_id,
            &mnemonic_salt_b64,
            &usb.payload.salt_u,
        ),
    )?;
    let w_cu = wrap_master_key(
        &master_key,
        &local.f_c,
        &usb.f_u,
        WRAP_LABEL_CU,
        &cu_wrap_aad(
            config.user_id,
            &config.device_id,
            &usb.payload.usb_id,
            &local.payload.salt_c,
            &usb.payload.salt_u,
        ),
    )?;
    let updated_local_payload = LocalFactorPayload {
        w_cu: Some(w_cu.clone()),
        ..updated_local_payload
    };
    let local_package = create_local_factor_package(
        provider,
        config.user_id,
        &config.device_id,
        &config.platform,
        &updated_local_payload,
    )?;
    write_local_factor_package(&config.local_factor_path, &local_package)?;
    let updated_usb_payload = UsbFactorPayload {
        schema_version: FACTOR_PAYLOAD_SCHEMA_VERSION,
        user_id: usb.payload.user_id,
        usb_id: usb.payload.usb_id.clone(),
        usb_secret: usb.payload.usb_secret.clone(),
        salt_u: usb.payload.salt_u.clone(),
        mnemonic_salt: mnemonic_salt_b64,
        password_derivation_algorithm: local.payload.password_derivation_algorithm,
        mnemonic_verifier: Some(mnemonic_verifier),
        recovery_generation: generation,
        w_mu,
        w_cu,
    };
    let usb_package = create_usb_factor_package(
        config.user_id,
        &config.device_id,
        &config.platform,
        &updated_usb_payload,
    )?;
    let path = write_usb_factor_package(&request.usb_path, &usb_package)?;
    let store = CdrStore::new(&config.cdr_store_path);
    let records = store.list_all()?;
    write_usb_cdr_backup(&request.usb_path, config.user_id, &master_key, &records)?;

    let recovery = crypto_recovery::build_recovery_metadata(&master_key, generation)?;
    write_recovery_metadata(&paths.recovery_path, &recovery)?;
    remember_master_key(&config, &master_key)?;
    Ok(RecoveryResponse { generation, path })
}

fn ensure_local_matches_config(
    config: &AppConfig,
    package: &crate::domain::FactorPackage,
) -> Result<()> {
    if package.user_id != config.user_id || package.device_id != config.device_id {
        return Err(KeylessPassError::Integrity(
            "local factor package does not match this device".to_string(),
        ));
    }
    Ok(())
}
