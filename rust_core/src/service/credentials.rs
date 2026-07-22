use crate::domain::{CredentialDescriptionRecord, EncodingDescriptor};
use crate::error::{KeylessPassError, Result};
use crate::platform::{current_platform_provider, PlatformFactorProvider};
use crate::service::factor_keys::cached_master_key_with_local_factor;
use crate::storage::{read_config, CdrStore, StoragePaths};
use serde::Deserialize;
use uuid::Uuid;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AddCredentialRequest {
    pub display_name: String,
    pub service_hint: String,
    pub account_hint: String,
    #[serde(default)]
    pub notes: String,
    pub encoding_descriptor: Option<EncodingDescriptor>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateCredentialDisplayRequest {
    pub record_id: Uuid,
    pub version: u32,
    pub display_name: String,
    pub service_hint: String,
    pub account_hint: String,
    #[serde(default)]
    pub notes: String,
    pub encoding_descriptor: Option<EncodingDescriptor>,
}

pub fn add_credential(
    request: AddCredentialRequest,
) -> std::result::Result<CredentialDescriptionRecord, String> {
    crate::service::license::require_license_feature("desktop-client")?;
    let paths = StoragePaths::default().map_err(String::from)?;
    let provider = current_platform_provider(&paths.app_dir);
    add_credential_with_provider(&paths, provider.as_ref(), request).map_err(String::from)
}

pub fn list_credentials() -> std::result::Result<Vec<CredentialDescriptionRecord>, String> {
    crate::service::license::require_license_feature("desktop-client")?;
    let paths = StoragePaths::default().map_err(String::from)?;
    list_credentials_at(&paths).map_err(String::from)
}

pub fn update_credential_display(
    request: UpdateCredentialDisplayRequest,
) -> std::result::Result<CredentialDescriptionRecord, String> {
    crate::service::license::require_license_feature("desktop-client")?;
    let paths = StoragePaths::default().map_err(String::from)?;
    let provider = current_platform_provider(&paths.app_dir);
    update_credential_display_with_provider(&paths, provider.as_ref(), request)
        .map_err(String::from)
}

pub fn add_credential_with_provider(
    paths: &StoragePaths,
    provider: &dyn PlatformFactorProvider,
    request: AddCredentialRequest,
) -> Result<CredentialDescriptionRecord> {
    crate::service::license::require_license_feature_at(
        paths,
        provider,
        &crate::service::license::default_license_verifier(),
        "desktop-client",
    )?;
    let (config, master_key) = cached_master_key_with_local_factor(paths, provider)?;
    let store = CdrStore::new(&config.cdr_store_path);
    store.init()?;
    let seq = store.max_record_seq()? + 1;
    let descriptor = request.encoding_descriptor.unwrap_or_default();
    let mut record = CredentialDescriptionRecord::new(
        seq,
        request.display_name,
        request.service_hint,
        request.account_hint,
        request.notes,
        descriptor,
    );
    record.set_mac(&master_key)?;
    store.insert(&record)?;
    Ok(record)
}

pub fn list_credentials_at(paths: &StoragePaths) -> Result<Vec<CredentialDescriptionRecord>> {
    let config = read_config(paths)?;
    let store = CdrStore::new(&config.cdr_store_path);
    store.init()?;
    store.list_all()
}

pub fn update_credential_display_with_provider(
    paths: &StoragePaths,
    provider: &dyn PlatformFactorProvider,
    request: UpdateCredentialDisplayRequest,
) -> Result<CredentialDescriptionRecord> {
    crate::service::license::require_license_feature_at(
        paths,
        provider,
        &crate::service::license::default_license_verifier(),
        "desktop-client",
    )?;
    let (config, master_key) = cached_master_key_with_local_factor(paths, provider)?;
    let store = CdrStore::new(&config.cdr_store_path);
    let mut record = store.get(request.record_id, Some(request.version))?;

    if let Some(new_descriptor) = request.encoding_descriptor {
        if new_descriptor != record.encoding_descriptor {
            return Err(KeylessPassError::Validation(
                "encodingDescriptor is immutable within a CDR version; use rotation".to_string(),
            ));
        }
    }

    record.update_display_fields(
        request.display_name,
        request.service_hint,
        request.account_hint,
        request.notes,
        &master_key,
    )?;
    store.update(&record)?;
    Ok(record)
}
