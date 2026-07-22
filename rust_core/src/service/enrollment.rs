use crate::crypto::{b64_encode, kdf, recovery as crypto_recovery};
use crate::domain::{AppConfig, LocalFactorPayload, PasswordDerivationAlgorithm, UsbFactorPayload};
use crate::domain::{WRAP_LABEL_CU, WRAP_LABEL_MC, WRAP_LABEL_MU};
use crate::error::{KeylessPassError, Result};
use crate::platform::{
    current_platform_provider, current_security_status, PlatformFactorProvider,
    PlatformSecurityStatus,
};
use crate::service::factor_keys::{
    cu_wrap_aad, mc_wrap_aad, mu_wrap_aad, remember_master_key, wrap_master_key,
};
use crate::storage::{
    create_local_factor_package, create_usb_factor_package, write_config,
    write_local_factor_package, write_recovery_metadata, write_usb_cdr_backup,
    write_usb_factor_package, CdrStore, StoragePaths,
};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use uuid::Uuid;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EnrollmentRequest {
    pub mnemonic: String,
    pub usb_path: String,
    #[serde(default)]
    pub password_derivation_algorithm: PasswordDerivationAlgorithm,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EnrollmentResponse {
    pub config: AppConfig,
    pub usb_package_path: PathBuf,
    pub security_status: PlatformSecurityStatus,
}

pub fn enroll(request: EnrollmentRequest) -> std::result::Result<EnrollmentResponse, String> {
    crate::service::license::require_license_feature("desktop-client")?;
    let paths = StoragePaths::default().map_err(String::from)?;
    let provider = current_platform_provider(&paths.app_dir);
    enroll_with_provider(&paths, provider.as_ref(), request).map_err(String::from)
}

pub fn enroll_with_provider(
    paths: &StoragePaths,
    provider: &dyn PlatformFactorProvider,
    request: EnrollmentRequest,
) -> Result<EnrollmentResponse> {
    crate::service::license::require_license_feature_at(
        paths,
        provider,
        &crate::service::license::default_license_verifier(),
        "desktop-client",
    )?;
    paths.ensure()?;
    if paths.config_path.exists() || paths.local_factor_path.exists() || paths.db_path.exists() {
        return Err(KeylessPassError::Validation(
            "KeylessPass is already enrolled on this device; ordinary re-enrollment is blocked to avoid overwriting master-key-dependent recovery material. Use recovery to rebuild a missing factor package, or perform an explicit factory reset outside the normal enrollment flow.".to_string(),
        ));
    }

    let user_id = Uuid::new_v4();
    let device_id = provider.get_or_create_device_id()?;
    let device_secret = provider.get_or_create_device_secret()?;
    let platform = provider.platform_name();
    let k_master_bytes = crate::crypto::random_bytes(32);
    let mut k_master = [0_u8; 32];
    k_master.copy_from_slice(&k_master_bytes);
    let usb_secret = crate::crypto::random_bytes(32);
    let usb_id = Uuid::new_v4().to_string();
    let device_salt = crate::crypto::random_bytes(16);
    let device_salt_b64 = b64_encode(&device_salt);
    let usb_salt = crate::crypto::random_bytes(16);
    let usb_salt_b64 = b64_encode(&usb_salt);
    let mnemonic_salt = crate::crypto::random_bytes(16);
    let mnemonic_salt_b64 = b64_encode(&mnemonic_salt);
    let f_m = kdf::derive_mnemonic_factor(&request.mnemonic, &mnemonic_salt)?;
    let f_c =
        kdf::derive_platform_factor(device_secret.expose(), &device_id, &user_id, &device_salt)?;
    let f_u = kdf::derive_usb_factor(&usb_secret, &usb_id, &user_id, &usb_salt)?;
    let mnemonic_verifier = kdf::derive_mnemonic_verifier(&f_m)?;
    let password_derivation_algorithm = request.password_derivation_algorithm;
    let w_mc = wrap_master_key(
        &k_master,
        &f_m,
        &f_c,
        WRAP_LABEL_MC,
        &mc_wrap_aad(user_id, &device_id, &mnemonic_salt_b64, &device_salt_b64),
    )?;
    let w_mu = wrap_master_key(
        &k_master,
        &f_m,
        &f_u,
        WRAP_LABEL_MU,
        &mu_wrap_aad(user_id, &usb_id, &mnemonic_salt_b64, &usb_salt_b64),
    )?;
    let w_cu = wrap_master_key(
        &k_master,
        &f_c,
        &f_u,
        WRAP_LABEL_CU,
        &cu_wrap_aad(
            user_id,
            &device_id,
            &usb_id,
            &device_salt_b64,
            &usb_salt_b64,
        ),
    )?;

    let local_payload = LocalFactorPayload {
        schema_version: crate::domain::FACTOR_PAYLOAD_SCHEMA_VERSION,
        user_id,
        device_id: device_id.clone(),
        salt_c: device_salt_b64.clone(),
        mnemonic_salt: mnemonic_salt_b64.clone(),
        password_derivation_algorithm,
        mnemonic_verifier: Some(mnemonic_verifier.clone()),
        recovery_generation: 1,
        w_mc,
        w_cu: Some(w_cu.clone()),
    };
    let usb_payload = UsbFactorPayload {
        schema_version: crate::domain::FACTOR_PAYLOAD_SCHEMA_VERSION,
        user_id,
        usb_id: usb_id.clone(),
        usb_secret: b64_encode(&usb_secret),
        salt_u: usb_salt_b64,
        mnemonic_salt: mnemonic_salt_b64,
        password_derivation_algorithm,
        mnemonic_verifier: Some(mnemonic_verifier),
        recovery_generation: 1,
        w_mu,
        w_cu,
    };

    let local_package =
        create_local_factor_package(provider, user_id, &device_id, &platform, &local_payload)?;
    write_local_factor_package(&paths.local_factor_path, &local_package)?;

    let usb_package = create_usb_factor_package(user_id, &device_id, &platform, &usb_payload)?;
    let usb_package_path = write_usb_factor_package(&request.usb_path, &usb_package)?;

    CdrStore::new(&paths.db_path).init()?;
    write_usb_cdr_backup(&request.usb_path, user_id, &k_master, &[])?;

    let config = AppConfig::new(
        env!("CARGO_PKG_VERSION"),
        user_id,
        platform,
        device_id,
        paths.db_path.clone(),
        paths.local_factor_path.clone(),
        password_derivation_algorithm,
    );
    write_config(paths, &config)?;
    remember_master_key(&config, &k_master)?;

    let recovery = crypto_recovery::build_recovery_metadata(&k_master, 1)?;
    write_recovery_metadata(&paths.recovery_path, &recovery)?;

    Ok(EnrollmentResponse {
        config,
        usb_package_path,
        security_status: current_security_status(provider),
    })
}
