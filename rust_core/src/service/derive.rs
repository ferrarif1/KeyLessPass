use crate::crypto::{encoder, kdf};
use crate::derivation::{derive_password_v3, Ff1CycleWalking};
use crate::domain::CredentialDescriptionRecord;
use crate::error::{KeylessPassError, Result};
use crate::platform::{current_platform_provider, PlatformFactorProvider};
use crate::service::factor_keys::{
    load_local_context, master_key_from_mnemonic_local, remember_master_key,
};
use crate::service::migration::unlock_v3_with_recovery_phrase;
use crate::storage::{read_config, recovery_manifest_v3_file, CdrStore, StoragePaths};
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
            "recovery share phrase is required".to_string(),
        ));
    }

    let config = read_config(paths)?;
    let (master_key, legacy_algorithm) = if recovery_manifest_v3_file(paths).exists() {
        (
            unlock_v3_with_recovery_phrase(paths, provider, &request.mnemonic)?,
            None,
        )
    } else {
        let local = load_local_context(provider, &config.local_factor_path)?;
        if local.package.user_id != config.user_id || local.package.device_id != config.device_id {
            return Err(KeylessPassError::Integrity(
                "local factor package does not match this device".to_string(),
            ));
        }
        let algorithm = local.payload.password_derivation_algorithm;
        (
            master_key_from_mnemonic_local(&request.mnemonic, &local)?,
            Some(algorithm),
        )
    };
    remember_master_key(&config, &master_key)?;
    let store = CdrStore::new(&config.cdr_store_path);
    let record = store.get(request.record_id, request.version)?;
    record.verify_mac(&master_key)?;

    let algorithm = config.password_derivation_algorithm;
    if legacy_algorithm.is_some_and(|legacy| algorithm != legacy) {
        return Err(KeylessPassError::Integrity(
            "derivation algorithm metadata mismatch".to_string(),
        ));
    }
    let root_key = || -> Result<[u8; 32]> {
        master_key
            .as_slice()
            .try_into()
            .map_err(|_| KeylessPassError::Crypto("Root Key must contain 256 bits".to_string()))
    };
    let password = match (record.derivation_version, record.encoder_version) {
        (crate::domain::CDR_DERIVATION_VERSION_V3, crate::domain::CDR_ENCODER_VERSION_V3) => {
            derive_password_v3(&root_key()?, &record, &Ff1CycleWalking::default())?.password
        }
        (crate::domain::CDR_DERIVATION_VERSION, crate::domain::CDR_ENCODER_VERSION)
            if record.schema_version >= crate::domain::CDR_SCHEMA_VERSION =>
        {
            let service_secret = kdf::derive_service_secret_v3(&root_key()?, &record)?;
            encoder::encode_password(&service_secret, &record.encoding_descriptor)?
        }
        (1, 1 | 2) if record.schema_version < crate::domain::CDR_SCHEMA_VERSION => {
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
            encoder::encode_password(&service_secret, &record.encoding_descriptor)?
        }
        versions => {
            return Err(KeylessPassError::Validation(format!(
                "unsupported derivation/encoder version pair: {versions:?}"
            )))
        }
    };

    Ok(DerivedPasswordResponse {
        password,
        expires_at: Utc::now() + Duration::seconds(30),
        record,
    })
}
