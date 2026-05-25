use crate::crypto::{encoder, kdf};
use crate::domain::CredentialDescriptionRecord;
use crate::error::{KeylessPassError, Result};
use crate::platform::{current_platform_provider, PlatformFactorProvider};
use crate::storage::{
    load_local_factor_payload, load_usb_factor_payload, read_config, CdrStore, StoragePaths,
};
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DerivePasswordRequest {
    pub record_id: Uuid,
    pub version: Option<u32>,
    pub mnemonic: String,
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
    let paths = StoragePaths::default().map_err(String::from)?;
    let provider = current_platform_provider(&paths.app_dir);
    derive_password_with_provider(&paths, provider.as_ref(), request).map_err(String::from)
}

pub fn derive_password_with_provider(
    paths: &StoragePaths,
    provider: &dyn PlatformFactorProvider,
    request: DerivePasswordRequest,
) -> Result<DerivedPasswordResponse> {
    if request.mnemonic.trim().is_empty() {
        return Err(KeylessPassError::MissingFactor(
            "mnemonic phrase is required".to_string(),
        ));
    }

    let config = read_config(paths)?;
    let (_, local_payload) = load_local_factor_payload(provider, &config.local_factor_path)?;
    let (_, usb_payload) = load_usb_factor_payload(&request.mnemonic, &request.usb_path)?;

    if local_payload.k_master != usb_payload.k_master {
        return Err(KeylessPassError::Integrity(
            "local and USB master key mismatch".to_string(),
        ));
    }
    if local_payload.mnemonic_salt != usb_payload.mnemonic_salt {
        return Err(KeylessPassError::Integrity(
            "local and USB mnemonic salt mismatch".to_string(),
        ));
    }

    let master_key = crate::crypto::b64_decode(&local_payload.k_master)?;
    let store = CdrStore::new(&config.cdr_store_path);
    let record = store.get(request.record_id, request.version)?;
    record.verify_mac(&master_key)?;

    let device_secret = crate::crypto::b64_decode(&local_payload.device_secret)?;
    let f_c = kdf::derive_platform_factor(
        &device_secret,
        &config.device_id,
        &config.user_id,
        &config.platform,
    )?;
    let f_u = crate::crypto::b64_decode(&usb_payload.usb_secret)?;
    let algorithm = config.password_derivation_algorithm;
    if algorithm != local_payload.password_derivation_algorithm
        || algorithm != usb_payload.password_derivation_algorithm
    {
        return Err(KeylessPassError::Integrity(
            "derivation algorithm metadata mismatch".to_string(),
        ));
    }
    let derivation_key = kdf::derive_password_root_from_master(&master_key, &f_c, &f_u)?;
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
