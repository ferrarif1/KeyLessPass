use crate::crypto::{encoder, kdf};
use crate::domain::CredentialDescriptionRecord;
use crate::error::{KeylessPassError, Result};
use crate::platform::{current_platform_provider, PlatformFactorProvider};
use crate::service::factor_keys::{
    load_local_context, master_key_from_mnemonic_local, remember_master_key,
};
use crate::storage::{read_config, CdrStore, StoragePaths};
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DerivePasswordRequest {
    pub record_id: Uuid,
    pub version: Option<u32>,
    pub mnemonic: String,
    #[serde(default)]
    pub usb_path: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DerivedPasswordResponse {
    pub password: String,
    pub expires_at: DateTime<Utc>,
    pub record: CredentialDescriptionRecord,
}

pub fn derive_password(
    request: DerivePasswordRequest,
) -> std::result::Result<DerivedPasswordResponse, String> {
    crate::service::license::require_license_feature("desktop-client")?;
    let paths = StoragePaths::default().map_err(String::from)?;
    let provider = current_platform_provider(&paths.app_dir);
    derive_password_with_provider(&paths, provider.as_ref(), request).map_err(String::from)
}

pub fn derive_password_with_provider(
    paths: &StoragePaths,
    provider: &dyn PlatformFactorProvider,
    request: DerivePasswordRequest,
) -> Result<DerivedPasswordResponse> {
    crate::service::license::require_license_feature_at(
        paths,
        provider,
        &crate::service::license::default_license_verifier(),
        "desktop-client",
    )?;
    if request.mnemonic.trim().is_empty() {
        return Err(KeylessPassError::MissingFactor(
            "mnemonic phrase is required".to_string(),
        ));
    }

    let config = read_config(paths)?;
    let local = load_local_context(provider, &config.local_factor_path)?;
    if local.package.user_id != config.user_id || local.package.device_id != config.device_id {
        return Err(KeylessPassError::Integrity(
            "local factor package does not match this device".to_string(),
        ));
    }

    let master_key = master_key_from_mnemonic_local(&request.mnemonic, &local)?;
    remember_master_key(&config, &master_key)?;
    let store = CdrStore::new(&config.cdr_store_path);
    let record = store.get(request.record_id, request.version)?;
    record.verify_mac(&master_key)?;

    let algorithm = config.password_derivation_algorithm;
    if algorithm != local.payload.password_derivation_algorithm {
        return Err(KeylessPassError::Integrity(
            "derivation algorithm metadata mismatch".to_string(),
        ));
    }
    let derivation_key = kdf::derive_password_root_from_master(&master_key)?;
    let service_secret = kdf::derive_service_secret_with_algorithm(
        algorithm,
        &derivation_key,
        &config.user_id,
        record.record_seq,
        &record.record_id,
        record.version,
        &crate::crypto::b64_decode(&record.salt)?,
    )?;
    let password = encoder::encode_password(&service_secret, &record.encoding_descriptor)?;

    Ok(DerivedPasswordResponse {
        password,
        expires_at: Utc::now() + Duration::seconds(30),
        record,
    })
}
