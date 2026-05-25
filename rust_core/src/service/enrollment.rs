use crate::crypto::{b64_encode, kdf, recovery as crypto_recovery, SecretBytes};
use crate::domain::{AppConfig, LocalFactorPayload, PasswordDerivationAlgorithm, UsbFactorPayload};
use crate::error::{KeylessPassError, Result};
use crate::platform::{
    current_platform_provider, current_security_status, PlatformFactorProvider,
    PlatformSecurityStatus,
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
    let paths = StoragePaths::default().map_err(String::from)?;
    let provider = current_platform_provider(&paths.app_dir);
    enroll_with_provider(&paths, provider.as_ref(), request).map_err(String::from)
}

pub fn enroll_with_provider(
    paths: &StoragePaths,
    provider: &dyn PlatformFactorProvider,
    request: EnrollmentRequest,
) -> Result<EnrollmentResponse> {
    paths.ensure()?;
    if paths.config_path.exists() || paths.local_factor_path.exists() || paths.db_path.exists() {
        return Err(KeylessPassError::Validation(
            "KeylessPass is already enrolled on this device; ordinary re-enrollment is blocked to avoid overwriting master-key-dependent recovery material. Use recovery to rebuild a missing factor package, or perform an explicit factory reset outside the normal enrollment flow.".to_string(),
        ));
    }

    let user_id = Uuid::new_v4();
    let device_id = provider.get_or_create_device_id()?;
    let device_secret: SecretBytes = provider.get_or_create_device_secret()?;
    let platform = provider.platform_name();
    let k_master = crate::crypto::random_bytes(32);
    let usb_secret = crate::crypto::random_bytes(32);
    let mnemonic_salt = crate::crypto::random_bytes(16);
    let mnemonic_salt_b64 = b64_encode(&mnemonic_salt);
    let f_m = kdf::derive_mnemonic_factor(&request.mnemonic, &user_id, &mnemonic_salt)?;
    let mnemonic_verifier = kdf::derive_mnemonic_verifier(&f_m)?;
    let password_derivation_algorithm = request.password_derivation_algorithm;

    let local_payload = LocalFactorPayload {
        k_master: b64_encode(&k_master),
        device_secret: b64_encode(device_secret.expose()),
        usb_secret: b64_encode(&usb_secret),
        mnemonic_salt: mnemonic_salt_b64.clone(),
        password_derivation_algorithm,
        mnemonic_verifier: Some(mnemonic_verifier.clone()),
        recovery_generation: 1,
    };
    let usb_payload = UsbFactorPayload {
        k_master: b64_encode(&k_master),
        usb_secret: b64_encode(&usb_secret),
        device_secret: b64_encode(device_secret.expose()),
        mnemonic_salt: mnemonic_salt_b64,
        password_derivation_algorithm,
        mnemonic_verifier: Some(mnemonic_verifier),
        recovery_generation: 1,
    };

    let local_package =
        create_local_factor_package(provider, user_id, &device_id, &platform, &local_payload)?;
    write_local_factor_package(&paths.local_factor_path, &local_package)?;

    let usb_package = create_usb_factor_package(
        &request.mnemonic,
        user_id,
        &device_id,
        &platform,
        &usb_payload,
    )?;
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

    let recovery = crypto_recovery::build_recovery_metadata(&k_master, 1)?;
    write_recovery_metadata(&paths.recovery_path, &recovery)?;

    Ok(EnrollmentResponse {
        config,
        usb_package_path,
        security_status: current_security_status(provider),
    })
}
