use crate::crypto::{encoder, kdf};
use crate::derivation::{
    derive_password_v3, derive_password_v3_with_policy, DomainPermutation, Ff1CycleWalking,
};
use crate::domain::{
    CredentialDescriptionRecord, CredentialState, EncodingDescriptor, RotationContract,
    CDR_DERIVATION_VERSION, CDR_DERIVATION_VERSION_V3, CDR_ENCODER_VERSION, CDR_ENCODER_VERSION_V3,
    CDR_SCHEMA_VERSION,
};
use crate::error::{KeylessPassError, Result};
use crate::platform::{current_platform_provider, PlatformFactorProvider};
use crate::policy::{CompiledPolicy, PolicySpec};
use crate::service::factor_keys::cached_master_key_with_local_factor;
use crate::storage::{CdrStore, StoragePaths};
use serde::Deserialize;
use uuid::Uuid;

pub const MAX_PASSWORD_HISTORY_WINDOW: usize = 64;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MigrateCredentialToV3Request {
    pub record_id: Uuid,
    pub encoding_descriptor: Option<EncodingDescriptor>,
    #[serde(default)]
    pub history_window: usize,
    #[serde(default)]
    pub rotation_contract: RotationContract,
}

pub fn migrate_credential_to_v3(
    request: MigrateCredentialToV3Request,
) -> std::result::Result<CredentialDescriptionRecord, String> {
    let paths = StoragePaths::default().map_err(String::from)?;
    let provider = current_platform_provider(&paths.app_dir);
    migrate_credential_to_v3_with_provider(&paths, provider.as_ref(), request).map_err(String::from)
}

pub fn migrate_credential_to_v3_with_provider(
    paths: &StoragePaths,
    provider: &dyn PlatformFactorProvider,
    request: MigrateCredentialToV3Request,
) -> Result<CredentialDescriptionRecord> {
    if request.history_window > MAX_PASSWORD_HISTORY_WINDOW {
        return Err(validation("password history window exceeds 64"));
    }
    let (config, root_key) = cached_master_key_with_local_factor(paths, provider)?;
    let store = CdrStore::new(&config.cdr_store_path);
    let previous = store.get(request.record_id, None)?;
    if previous.state != CredentialState::Active {
        return Err(validation(
            "v3 migration requires one active committed credential",
        ));
    }
    previous.verify_mac(&root_key)?;
    let descriptor = request
        .encoding_descriptor
        .unwrap_or_else(|| previous.encoding_descriptor.clone());
    let mut pending = CredentialDescriptionRecord::rotation_to_v3_with_contract(
        &previous,
        descriptor,
        request.rotation_contract,
    );

    let mut history = store
        .list_all()?
        .into_iter()
        .filter(|record| {
            record.record_id == previous.record_id
                && record.version <= previous.version
                && record.state != CredentialState::PendingRotation
        })
        .collect::<Vec<_>>();
    history.sort_by_key(|record| std::cmp::Reverse(record.version));
    history.truncate(request.history_window);
    for record in &history {
        record.verify_mac(&root_key)?;
        if record.root_generation != pending.root_generation {
            return Err(validation(
                "history regeneration across Root-Key generations is unavailable",
            ));
        }
    }
    select_v3_generation_avoiding_history(
        &root_key,
        &mut pending,
        &history,
        &Ff1CycleWalking::default(),
    )?;
    pending.set_mac(&root_key)?;
    store.insert(&pending)?;
    Ok(pending)
}

pub fn select_v3_generation_avoiding_history(
    root_key: &[u8; 32],
    pending: &mut CredentialDescriptionRecord,
    history: &[CredentialDescriptionRecord],
    permutation: &dyn DomainPermutation,
) -> Result<()> {
    if pending.derivation_version != CDR_DERIVATION_VERSION_V3
        || pending.encoder_version != CDR_ENCODER_VERSION_V3
        || pending.policy_epoch.is_none()
    {
        return Err(validation("candidate is not a complete v3 record"));
    }
    let policy = PolicySpec::from_encoding_descriptor(&pending.encoding_descriptor)?;
    let policy_hash = policy.policy_hash()?;
    let compiled = CompiledPolicy::compile(policy)?;
    let history_passwords = history
        .iter()
        .map(|record| derive_committed_password(root_key, record, permutation))
        .collect::<Result<Vec<_>>>()?;

    loop {
        let candidate =
            derive_password_v3_with_policy(root_key, pending, &compiled, policy_hash, permutation)?
                .password;
        if !history_passwords.iter().any(|old| old == &candidate) {
            return Ok(());
        }
        pending.credential_generation = pending
            .credential_generation
            .checked_add(1)
            .ok_or_else(|| validation("credential generation counter exhausted"))?;
    }
}

pub fn derive_committed_password(
    root_key: &[u8; 32],
    record: &CredentialDescriptionRecord,
    permutation: &dyn DomainPermutation,
) -> Result<String> {
    match (record.derivation_version, record.encoder_version) {
        (CDR_DERIVATION_VERSION_V3, CDR_ENCODER_VERSION_V3) => {
            Ok(derive_password_v3(root_key, record, permutation)?.password)
        }
        (CDR_DERIVATION_VERSION, CDR_ENCODER_VERSION)
            if record.schema_version >= CDR_SCHEMA_VERSION =>
        {
            let seed = kdf::derive_service_secret_v3(root_key, record)?;
            encoder::encode_password(&seed, &record.encoding_descriptor)
        }
        versions => Err(validation(&format!(
            "history contains unsupported derivation/encoder versions: {versions:?}"
        ))),
    }
}

fn validation(message: &str) -> KeylessPassError {
    KeylessPassError::Validation(message.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::EncodingDescriptor;

    #[test]
    fn v3_rotation_keeps_salt_and_uses_epoch_local_generations() {
        let previous = CredentialDescriptionRecord::new(
            Uuid::from_u128(1),
            1,
            1,
            "test",
            "service",
            "account",
            "",
            EncodingDescriptor::default(),
        );
        let migrated = CredentialDescriptionRecord::rotation_to_v3_with_contract(
            &previous,
            previous.encoding_descriptor.clone(),
            RotationContract::AtomicReplacement,
        );
        assert_eq!(migrated.salt, previous.salt);
        assert_eq!(migrated.policy_epoch, Some(1));
        assert_eq!(migrated.credential_generation, 0);

        let rotated = CredentialDescriptionRecord::rotation_to_v3_with_contract(
            &migrated,
            migrated.encoding_descriptor.clone(),
            RotationContract::AtomicReplacement,
        );
        assert_eq!(rotated.policy_epoch, Some(1));
        assert_eq!(rotated.credential_generation, 1);

        let mut changed_descriptor = rotated.encoding_descriptor.clone();
        changed_descriptor.rule_version += 1;
        let changed = CredentialDescriptionRecord::rotation_to_v3_with_contract(
            &rotated,
            changed_descriptor,
            RotationContract::AtomicReplacement,
        );
        assert_eq!(changed.policy_epoch, Some(2));
        assert_eq!(changed.credential_generation, 0);
    }
}
