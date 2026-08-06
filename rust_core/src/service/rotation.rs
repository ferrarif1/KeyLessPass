use crate::crypto::encoder;
use crate::domain::{
    transition_rotation, CredentialDescriptionRecord, CredentialState, EncodingDescriptor,
    RotationEvent, RotationState,
};
use crate::error::{KeylessPassError, Result};
use crate::platform::{current_platform_provider, PlatformFactorProvider};
use crate::service::factor_keys::cached_master_key_with_local_factor;
use crate::storage::{CdrStore, StoragePaths};
use serde::Deserialize;
use uuid::Uuid;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RotateCredentialRequest {
    pub record_id: Uuid,
    pub encoding_descriptor: Option<EncodingDescriptor>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConfirmRotationRequest {
    pub record_id: Uuid,
    pub version: u32,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CancelRotationRequest {
    pub record_id: Uuid,
    pub version: u32,
}

pub fn rotate_credential(
    request: RotateCredentialRequest,
) -> std::result::Result<CredentialDescriptionRecord, String> {
    let paths = StoragePaths::default().map_err(String::from)?;
    let provider = current_platform_provider(&paths.app_dir);
    rotate_credential_with_provider(&paths, provider.as_ref(), request).map_err(String::from)
}

pub fn confirm_rotation(request: ConfirmRotationRequest) -> std::result::Result<(), String> {
    let paths = StoragePaths::default().map_err(String::from)?;
    let provider = current_platform_provider(&paths.app_dir);
    confirm_rotation_with_provider(&paths, provider.as_ref(), request).map_err(String::from)
}

pub fn cancel_rotation(request: CancelRotationRequest) -> std::result::Result<(), String> {
    let paths = StoragePaths::default().map_err(String::from)?;
    let provider = current_platform_provider(&paths.app_dir);
    cancel_rotation_with_provider(&paths, provider.as_ref(), request).map_err(String::from)
}

pub fn rotate_credential_with_provider(
    paths: &StoragePaths,
    provider: &dyn PlatformFactorProvider,
    request: RotateCredentialRequest,
) -> Result<CredentialDescriptionRecord> {
    let (config, master_key) = cached_master_key_with_local_factor(paths, provider)?;
    let store = CdrStore::new(&config.cdr_store_path);
    let previous = store.get(request.record_id, None)?;
    if previous.state == CredentialState::Retired {
        return Err(KeylessPassError::Validation(
            "cannot rotate a retired CDR version".to_string(),
        ));
    }
    if previous.state == CredentialState::PendingRotation {
        return Err(KeylessPassError::Validation(
            "complete or cancel the pending CDR version before rotating again".to_string(),
        ));
    }
    previous.verify_mac(&master_key)?;
    let descriptor = request
        .encoding_descriptor
        .unwrap_or_else(|| previous.encoding_descriptor.clone());
    encoder::ensure_rotation_required(
        &previous.encoding_descriptor,
        &descriptor,
        previous.version,
        previous.version + 1,
    )?;
    let mut next = CredentialDescriptionRecord::rotation_from(&previous, descriptor);
    next.set_mac(&master_key)?;
    store.insert(&next)?;
    Ok(next)
}

pub fn confirm_rotation_with_provider(
    paths: &StoragePaths,
    provider: &dyn PlatformFactorProvider,
    request: ConfirmRotationRequest,
) -> Result<()> {
    let (config, master_key) = cached_master_key_with_local_factor(paths, provider)?;
    let store = CdrStore::new(&config.cdr_store_path);
    let mut new_record = store.get(request.record_id, Some(request.version))?;
    if new_record.state != CredentialState::PendingRotation {
        return Err(KeylessPassError::Validation(
            "selected CDR version is not pending rotation".to_string(),
        ));
    }
    new_record.verify_mac(&master_key)?;

    if new_record.rotation_state == RotationState::Prepared {
        transition_rotation(&mut new_record, RotationEvent::RequestSent)?;
        new_record.set_mac(&master_key)?;
        store.update(&new_record)?;
    }
    if new_record.rotation_state == RotationState::UpdateSent {
        transition_rotation(&mut new_record, RotationEvent::RemoteNewPasswordVerified)?;
        new_record.set_mac(&master_key)?;
        store.update(&new_record)?;
    }
    if new_record.rotation_state != RotationState::RemoteConfirmed {
        return Err(KeylessPassError::Validation(
            "rotation must have remote-success evidence before local commit".to_string(),
        ));
    }

    for mut record in store.list_all()? {
        if record.record_id == request.record_id
            && record.version != request.version
            && record.state != CredentialState::Retired
        {
            record.verify_mac(&master_key)?;
            record.mark_retired(&master_key)?;
            store.update(&record)?;
        }
    }

    transition_rotation(&mut new_record, RotationEvent::CommitLocal)?;
    new_record.set_mac(&master_key)?;
    store.update(&new_record)?;
    transition_rotation(&mut new_record, RotationEvent::Finalize)?;
    new_record.set_mac(&master_key)?;
    store.update(&new_record)?;
    Ok(())
}

pub fn cancel_rotation_with_provider(
    paths: &StoragePaths,
    provider: &dyn PlatformFactorProvider,
    request: CancelRotationRequest,
) -> Result<()> {
    let (config, master_key) = cached_master_key_with_local_factor(paths, provider)?;
    let store = CdrStore::new(&config.cdr_store_path);
    let mut record = store.get(request.record_id, Some(request.version))?;
    record.verify_mac(&master_key)?;
    if record.state != CredentialState::PendingRotation {
        return Err(KeylessPassError::Validation(
            "selected CDR version is not pending rotation".to_string(),
        ));
    }
    transition_rotation(&mut record, RotationEvent::Abort)?;
    record.set_mac(&master_key)?;
    store.update(&record)?;
    Ok(())
}

pub fn mark_rotation_unknown_with_provider(
    paths: &StoragePaths,
    provider: &dyn PlatformFactorProvider,
    record_id: Uuid,
    version: u32,
) -> Result<CredentialDescriptionRecord> {
    let (config, master_key) = cached_master_key_with_local_factor(paths, provider)?;
    let store = CdrStore::new(&config.cdr_store_path);
    let mut record = store.get(record_id, Some(version))?;
    record.verify_mac(&master_key)?;
    if record.rotation_state == RotationState::Prepared {
        transition_rotation(&mut record, RotationEvent::RequestSent)?;
    }
    transition_rotation(&mut record, RotationEvent::TransportOutcomeUnknown)?;
    record.set_mac(&master_key)?;
    store.update(&record)?;
    Ok(record)
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReconciliationResult {
    NewPasswordWorks,
    OldPasswordWorks,
    BothPasswordsWork,
    NeitherWorks,
}

pub fn reconcile_rotation_with_provider(
    paths: &StoragePaths,
    provider: &dyn PlatformFactorProvider,
    record_id: Uuid,
    version: u32,
    result: ReconciliationResult,
) -> Result<CredentialDescriptionRecord> {
    let (config, master_key) = cached_master_key_with_local_factor(paths, provider)?;
    let store = CdrStore::new(&config.cdr_store_path);
    let mut record = store.get(record_id, Some(version))?;
    record.verify_mac(&master_key)?;
    if record.rotation_state == RotationState::UnknownOutcome {
        transition_rotation(&mut record, RotationEvent::BeginReconciliation)?;
    }
    let event = match result {
        ReconciliationResult::NewPasswordWorks => RotationEvent::NewPasswordAuthenticated,
        ReconciliationResult::OldPasswordWorks => RotationEvent::OldPasswordAuthenticated,
        ReconciliationResult::BothPasswordsWork => RotationEvent::BothPasswordsAuthenticated,
        ReconciliationResult::NeitherWorks => RotationEvent::NeitherPasswordAuthenticated,
    };
    transition_rotation(&mut record, event)?;
    record.set_mac(&master_key)?;
    store.update(&record)?;
    Ok(record)
}
