use crate::crypto::kdf::{derive_vault_subkey, hkdf_32};
use crate::domain::{CredentialDescriptionRecord, EncodingDescriptor};
use crate::error::{KeylessPassError, Result};
use crate::policy::{
    CharacterClassConstraint, CompiledPolicy, FixedCharacterConstraint, PolicySpec,
};
use num_bigint::BigUint;
use serde::Serialize;

pub use crate::permutation::{
    DomainPermutation, Ff1CycleWalking, DEFAULT_MAX_CYCLE_WALKS, MAX_FF1_DOMAIN_BITS,
    MIN_FF1_DOMAIN_SIZE,
};

pub const DERIVATION_VERSION_V3: u32 = 3;
pub const ENCODER_VERSION_V3: u32 = 3;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DerivedPolicyPassword {
    pub password: String,
    pub rank: BigUint,
    pub domain_size: BigUint,
    pub policy_hash: [u8; 32],
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct CredentialKeyContext {
    domain: &'static str,
    service_id: String,
    account_id: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct PermutationTweak {
    domain: &'static str,
    vault_id: String,
    service_id: String,
    account_id: String,
    credential_salt: String,
    root_generation: u64,
    policy_id: String,
    policy_version: u32,
    policy_hash: String,
    policy_epoch: u64,
    derivation_version: u32,
    encoder_version: u32,
}

pub fn derive_password_v3(
    root_key: &[u8; 32],
    record: &CredentialDescriptionRecord,
    permutation: &dyn DomainPermutation,
) -> Result<DerivedPolicyPassword> {
    validate_v3_record(record)?;
    let spec = policy_spec_from_encoding_descriptor(&record.encoding_descriptor)?;
    let policy_hash = spec.policy_hash()?;
    let policy = CompiledPolicy::compile(spec)?;
    derive_password_v3_with_policy(root_key, record, &policy, policy_hash, permutation)
}

pub(crate) fn policy_spec_from_encoding_descriptor(
    descriptor: &EncodingDescriptor,
) -> Result<PolicySpec> {
    if descriptor.normalization != "none" {
        return Err(validation(
            "ASTER exact-domain derivation supports normalization=none only",
        ));
    }
    let fixed_characters = descriptor
        .fixed_positions
        .iter()
        .map(|fixed| {
            let mut characters = fixed.character.chars();
            let character = characters
                .next()
                .ok_or_else(|| validation("fixed character must not be empty"))?;
            if characters.next().is_some() {
                return Err(validation(
                    "fixed character must contain one Unicode scalar value",
                ));
            }
            Ok(FixedCharacterConstraint {
                index: fixed.index,
                character,
            })
        })
        .collect::<Result<Vec<_>>>()?;
    let classes = descriptor
        .required_classes
        .iter()
        .map(|class| CharacterClassConstraint {
            name: class.name.clone(),
            alphabet: class.alphabet.clone(),
            min_count: class.min_count,
            max_count: class.max_count,
        })
        .collect();
    Ok(PolicySpec {
        policy_ir_version: 1,
        min_length: descriptor.length,
        max_length: descriptor.length,
        alphabet: descriptor.allowed_alphabet.clone(),
        forbidden_characters: descriptor.forbidden_chars.clone(),
        classes,
        fixed_characters,
        fixed_prefix: String::new(),
        fixed_suffix: String::new(),
        forbidden_first_characters: descriptor.forbidden_first_chars.clone(),
        forbidden_last_characters: descriptor.forbidden_last_chars.clone(),
        max_total_per_character: descriptor.forbid_repeated_characters.then_some(1),
        max_identical_run: None,
        max_sequential_run: descriptor.forbid_sequential_characters.then_some(1),
        forbidden_substrings: Vec::new(),
    })
}

pub fn derive_password_v3_with_policy(
    root_key: &[u8; 32],
    record: &CredentialDescriptionRecord,
    policy: &CompiledPolicy,
    policy_hash: [u8; 32],
    permutation: &dyn DomainPermutation,
) -> Result<DerivedPolicyPassword> {
    validate_v3_record(record)?;
    let generation = BigUint::from(record.credential_generation);
    if &generation >= policy.total_count() {
        return Err(validation(
            "credential generation exhausted the policy domain",
        ));
    }
    let key = derive_credential_key(root_key, record)?;
    let tweak = permutation_tweak(record, &policy_hash)?;
    let rank = permutation.permute(&key, &tweak, policy.total_count(), &generation)?;
    Ok(DerivedPolicyPassword {
        password: policy.unrank(&rank)?,
        rank,
        domain_size: policy.total_count().clone(),
        policy_hash,
    })
}

pub fn derive_credential_key(
    root_key: &[u8; 32],
    record: &CredentialDescriptionRecord,
) -> Result<[u8; 32]> {
    let vault_key = derive_vault_subkey(
        root_key,
        &record.vault_id,
        record.root_generation,
        record.crypto_suite_version,
        b"credential-permutation/v3",
    )?;
    let salt = crate::crypto::b64_decode(&record.salt)?;
    if salt.len() != 16 {
        return Err(validation("v3 credential salt must contain 128 bits"));
    }
    let context = CredentialKeyContext {
        domain: "KeyLessPass/credential-key/v3",
        service_id: record.service_id.to_string(),
        account_id: record.account_id.to_string(),
    };
    hkdf_32(
        &vault_key,
        &salt,
        &serde_json_canonicalizer::to_vec(&context)?,
    )
}

pub fn permutation_tweak(
    record: &CredentialDescriptionRecord,
    policy_hash: &[u8; 32],
) -> Result<Vec<u8>> {
    validate_v3_record(record)?;
    let salt = crate::crypto::b64_decode(&record.salt)?;
    let tweak = PermutationTweak {
        domain: "KeyLessPass/policy-space-permutation/v3",
        vault_id: record.vault_id.to_string(),
        service_id: record.service_id.to_string(),
        account_id: record.account_id.to_string(),
        credential_salt: crate::crypto::b64_encode(&salt),
        root_generation: record.root_generation,
        policy_id: record.policy_id.to_string(),
        policy_version: record.policy_version,
        policy_hash: crate::crypto::b64_encode(policy_hash),
        policy_epoch: record.policy_epoch.expect("validated above"),
        derivation_version: record.derivation_version,
        encoder_version: record.encoder_version,
    };
    Ok(serde_json_canonicalizer::to_vec(&tweak)?)
}

fn validate_v3_record(record: &CredentialDescriptionRecord) -> Result<()> {
    if record.derivation_version != DERIVATION_VERSION_V3
        || record.encoder_version != ENCODER_VERSION_V3
    {
        return Err(validation("record is not a derivation/encoder v3 record"));
    }
    if record.vault_id.is_nil()
        || record.service_id.is_nil()
        || record.account_id.is_nil()
        || record.policy_id.is_nil()
    {
        return Err(validation("v3 derivation identifiers must be non-nil"));
    }
    if record.policy_epoch.is_none() {
        return Err(validation("v3 derivation requires policyEpoch"));
    }
    Ok(())
}

fn validation(message: &str) -> KeylessPassError {
    KeylessPassError::Validation(message.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{CredentialDescriptionRecord, EncodingDescriptor};
    use std::collections::HashSet;
    use uuid::Uuid;

    fn record() -> CredentialDescriptionRecord {
        let mut record = CredentialDescriptionRecord::new(
            Uuid::from_u128(1),
            1,
            1,
            "test",
            "service",
            "account",
            "",
            EncodingDescriptor::default(),
        );
        record.derivation_version = DERIVATION_VERSION_V3;
        record.encoder_version = ENCODER_VERSION_V3;
        record.policy_epoch = Some(1);
        record.credential_generation = 0;
        record
    }

    #[test]
    fn prototype_permutation_inverts_and_has_no_sample_collisions() {
        let domain = BigUint::from(1_000_003_u64);
        let backend = Ff1CycleWalking::default();
        let key = [0x42_u8; 32];
        let mut outputs = HashSet::new();
        for value in 0_u32..2_000 {
            let input = BigUint::from(value);
            let output = backend
                .permute(&key, b"test-tweak", &domain, &input)
                .unwrap();
            assert!(outputs.insert(output.clone()));
            assert_eq!(
                backend
                    .invert(&key, b"test-tweak", &domain, &output)
                    .unwrap(),
                input
            );
        }
    }

    #[test]
    fn complete_derivation_is_policy_compliant_and_non_repeating() {
        let root_key = [0x11_u8; 32];
        let backend = Ff1CycleWalking::default();
        let mut record = record();
        let policy = CompiledPolicy::compile(
            policy_spec_from_encoding_descriptor(&record.encoding_descriptor).unwrap(),
        )
        .unwrap();
        let policy_hash = policy.spec().policy_hash().unwrap();
        let mut passwords = HashSet::new();
        for generation in 0..128 {
            record.credential_generation = generation;
            let derived =
                derive_password_v3_with_policy(&root_key, &record, &policy, policy_hash, &backend)
                    .unwrap();
            assert!(policy.accepts(&derived.password));
            assert!(passwords.insert(derived.password));
        }
    }

    #[test]
    fn generation_is_not_part_of_the_tweak() {
        let mut left = record();
        let policy_hash = policy_spec_from_encoding_descriptor(&left.encoding_descriptor)
            .unwrap()
            .policy_hash()
            .unwrap();
        let left_tweak = permutation_tweak(&left, &policy_hash).unwrap();
        left.credential_generation = 99;
        assert_eq!(left_tweak, permutation_tweak(&left, &policy_hash).unwrap());
    }

    #[test]
    fn v3_fixed_vector_is_cross_platform_stable() {
        let vector: serde_json::Value = serde_json::from_str(include_str!(
            "../../test-vectors/password-derivation-v3.json"
        ))
        .unwrap();
        let root_key: [u8; 32] = crate::crypto::b64_decode(vector["rootKey"].as_str().unwrap())
            .unwrap()
            .try_into()
            .unwrap();
        let mut record = record();
        record.vault_id = Uuid::from_u128(0x00112233445566778899aabbccddeeff);
        record.root_generation = 7;
        record.service_id = Uuid::from_u128(0xaaaaaaaa111122223333444444444444);
        record.account_id = Uuid::from_u128(0xbbbbbbbb111122223333444444444444);
        record.policy_id = Uuid::from_u128(0xcccccccc111122223333444444444444);
        record.policy_version = 2;
        record.salt = "EREREREREREREREREREREQ==".to_string();
        let policy = policy_spec_from_encoding_descriptor(&record.encoding_descriptor).unwrap();
        let policy_hash = policy.policy_hash().unwrap();
        let key = derive_credential_key(&root_key, &record).unwrap();
        let tweak = permutation_tweak(&record, &policy_hash).unwrap();
        let derived = derive_password_v3(&root_key, &record, &Ff1CycleWalking::default()).unwrap();
        assert_eq!(serde_json::to_value(policy).unwrap(), vector["policy"]);
        assert_eq!(
            crate::crypto::b64_encode(&policy_hash),
            vector["policyHash"]
        );
        assert_eq!(crate::crypto::b64_encode(&key), vector["credentialKey"]);
        assert_eq!(
            serde_json::from_slice::<serde_json::Value>(&tweak).unwrap(),
            vector["tweak"]
        );
        assert_eq!(derived.domain_size.to_string(), vector["domainSize"]);
        assert_eq!(derived.rank.to_string(), vector["rank"]);
        assert_eq!(derived.password, vector["password"]);
    }
}
