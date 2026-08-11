//! Exact Policy-Space Credential Derivation (EPSCD).

use crate::error::{KeylessPassError, Result};
use crate::permutation::DomainPermutation;
use crate::policy::CompiledPolicy;
use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use hkdf::Hkdf;
use num_bigint::BigUint;
use rand::RngCore;
use serde::Serialize;
use sha2::{Digest, Sha256};
use uuid::Uuid;

pub const SCHEME_VERSION_V1: u32 = 1;
pub const SCHEME_VERSION_V2: u32 = 2;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CredentialContext {
    pub scheme_version: u32,
    pub vault_id: Uuid,
    pub service_id: Uuid,
    pub account_id: Uuid,
    /// Public random identifier delimiting a no-repeat credential lineage.
    /// Scheme v1 ignores this field; scheme v2 requires it to be non-nil.
    pub lineage_id: Uuid,
    pub credential_salt: [u8; 16],
    pub root_generation: u64,
    pub policy_id: Uuid,
    pub policy_version: u32,
    pub policy_epoch: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DerivedCredential {
    pub password: String,
    pub rank: BigUint,
    pub domain_size: BigUint,
    pub policy_hash: [u8; 32],
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct CredentialKeyInfo {
    domain: &'static str,
    scheme_version: u32,
    vault_id: String,
    service_id: String,
    account_id: String,
    root_generation: u64,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct CredentialKeyInfoV2 {
    domain: &'static str,
    scheme_version: u32,
    vault_id: String,
    service_id: String,
    account_id: String,
    lineage_id: String,
    credential_salt: String,
    root_generation: u64,
    policy_version: u32,
    policy_epoch: u64,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct PermutationTweak {
    domain: &'static str,
    scheme_version: u32,
    vault_id: String,
    service_id: String,
    account_id: String,
    credential_salt: String,
    root_generation: u64,
    policy_id: String,
    policy_version: u32,
    policy_hash: String,
    policy_epoch: u64,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct PermutationTweakV2 {
    domain: &'static str,
    scheme_version: u32,
    vault_id: String,
    service_id: String,
    account_id: String,
    lineage_id: String,
    credential_salt: String,
    root_generation: u64,
    policy_id: String,
    policy_version: u32,
    policy_hash: String,
    policy_epoch: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CollisionUnionBound {
    pub numerator: BigUint,
    pub denominator: BigUint,
}

impl CollisionUnionBound {
    pub fn is_certain(&self) -> bool {
        self.numerator == self.denominator
    }
}

pub fn derive_password(
    root_key: &[u8; 32],
    context: &CredentialContext,
    generation: u64,
    policy: &CompiledPolicy,
    permutation: &dyn DomainPermutation,
) -> Result<DerivedCredential> {
    validate_context(context)?;
    let generation = BigUint::from(generation);
    if &generation >= policy.total_count() {
        return Err(validation(
            "credential generation exhausted the policy domain",
        ));
    }
    let policy_hash = policy.spec().policy_hash()?;
    let key = derive_credential_key(root_key, context)?;
    let tweak = permutation_tweak(context, &policy_hash)?;
    let rank = permutation.permute(&key, &tweak, policy.total_count(), &generation)?;
    Ok(DerivedCredential {
        password: policy.unrank(&rank)?,
        rank,
        domain_size: policy.total_count().clone(),
        policy_hash,
    })
}

pub fn derive_credential_key(root_key: &[u8; 32], context: &CredentialContext) -> Result<[u8; 32]> {
    validate_context(context)?;
    let info = match context.scheme_version {
        SCHEME_VERSION_V1 => serde_json_canonicalizer::to_vec(&CredentialKeyInfo {
            domain: "EPSCD/credential-key/scheme-v1",
            scheme_version: context.scheme_version,
            vault_id: context.vault_id.to_string(),
            service_id: context.service_id.to_string(),
            account_id: context.account_id.to_string(),
            root_generation: context.root_generation,
        })?,
        SCHEME_VERSION_V2 => serde_json_canonicalizer::to_vec(&CredentialKeyInfoV2 {
            domain: "EPSCD/credential-key/scheme-v2",
            scheme_version: context.scheme_version,
            vault_id: context.vault_id.to_string(),
            service_id: context.service_id.to_string(),
            account_id: context.account_id.to_string(),
            lineage_id: context.lineage_id.to_string(),
            credential_salt: BASE64.encode(context.credential_salt),
            root_generation: context.root_generation,
            policy_version: context.policy_version,
            policy_epoch: context.policy_epoch,
        })?,
        _ => unreachable!("validated scheme version"),
    };
    let hkdf = Hkdf::<Sha256>::new(Some(&context.credential_salt), root_key);
    let mut key = [0_u8; 32];
    hkdf.expand(&info, &mut key)
        .map_err(|_| KeylessPassError::Crypto("EPSCD HKDF expand failed".to_string()))?;
    Ok(key)
}

/// Starts a fresh credential-key lineage after suspected `Kcred` exposure.
///
/// Rekeying starts a new lineage. Same-lineage non-repetition does not extend
/// across this boundary; cross-lineage reuse is reported as a probability
/// bound rather than prevented with a persisted password-history database.
pub fn rekey_credential_context(context: &CredentialContext) -> Result<CredentialContext> {
    validate_context(context)?;
    let mut rekeyed = context.clone();
    if context.scheme_version == SCHEME_VERSION_V2 {
        rekeyed.lineage_id = Uuid::new_v4();
    }
    loop {
        rand::rngs::OsRng.fill_bytes(&mut rekeyed.credential_salt);
        if rekeyed.credential_salt != context.credential_salt {
            return Ok(rekeyed);
        }
    }
}

/// Public identifier for freshness checkpoints; it is not a derivation secret.
pub fn credential_lineage_id(context: &CredentialContext) -> Result<String> {
    validate_context(context)?;
    if context.scheme_version == SCHEME_VERSION_V2 {
        return Ok(context.lineage_id.to_string());
    }
    let mut bytes = serde_json_canonicalizer::to_vec(&CredentialKeyInfo {
        domain: "EPSCD/credential-key/scheme-v1",
        scheme_version: context.scheme_version,
        vault_id: context.vault_id.to_string(),
        service_id: context.service_id.to_string(),
        account_id: context.account_id.to_string(),
        root_generation: context.root_generation,
    })?;
    bytes.extend_from_slice(&context.credential_salt);
    Ok(BASE64.encode(Sha256::digest(bytes)))
}

pub fn permutation_tweak(context: &CredentialContext, policy_hash: &[u8; 32]) -> Result<Vec<u8>> {
    validate_context(context)?;
    match context.scheme_version {
        SCHEME_VERSION_V1 => Ok(serde_json_canonicalizer::to_vec(&PermutationTweak {
            domain: "EPSCD/policy-space-permutation/scheme-v1",
            scheme_version: context.scheme_version,
            vault_id: context.vault_id.to_string(),
            service_id: context.service_id.to_string(),
            account_id: context.account_id.to_string(),
            credential_salt: BASE64.encode(context.credential_salt),
            root_generation: context.root_generation,
            policy_id: context.policy_id.to_string(),
            policy_version: context.policy_version,
            policy_hash: BASE64.encode(policy_hash),
            policy_epoch: context.policy_epoch,
        })?),
        SCHEME_VERSION_V2 => Ok(serde_json_canonicalizer::to_vec(&PermutationTweakV2 {
            domain: "EPSCD/policy-space-permutation/scheme-v2",
            scheme_version: context.scheme_version,
            vault_id: context.vault_id.to_string(),
            service_id: context.service_id.to_string(),
            account_id: context.account_id.to_string(),
            lineage_id: context.lineage_id.to_string(),
            credential_salt: BASE64.encode(context.credential_salt),
            root_generation: context.root_generation,
            policy_id: context.policy_id.to_string(),
            policy_version: context.policy_version,
            policy_hash: BASE64.encode(policy_hash),
            policy_epoch: context.policy_epoch,
        })?),
        _ => unreachable!("validated scheme version"),
    }
}

/// Union bound for `future_outputs` draws from a new independent lineage
/// colliding with `history_overlap` values that belong to the new domain.
/// The returned exact rational is `min(N, h*m) / N`.
pub fn cross_lineage_collision_union_bound(
    domain_size: &BigUint,
    history_overlap: &BigUint,
    future_outputs: &BigUint,
) -> Result<CollisionUnionBound> {
    if domain_size == &BigUint::from(0_u8) {
        return Err(validation("collision bound requires a non-empty domain"));
    }
    if history_overlap > domain_size {
        return Err(validation("history overlap exceeds the new policy domain"));
    }
    let product = history_overlap * future_outputs;
    Ok(CollisionUnionBound {
        numerator: product.min(domain_size.clone()),
        denominator: domain_size.clone(),
    })
}

pub fn policy_space_warning(policy: &CompiledPolicy, minimum_bits: f64) -> Option<String> {
    let bits = policy.entropy_bits();
    (bits < minimum_bits).then(|| {
        format!(
            "legacy policy exposes {bits:.2} effective bits, below the configured {minimum_bits:.2}-bit minimum"
        )
    })
}

fn validate_context(context: &CredentialContext) -> Result<()> {
    if !matches!(
        context.scheme_version,
        SCHEME_VERSION_V1 | SCHEME_VERSION_V2
    ) {
        return Err(validation("unsupported EPSCD scheme version"));
    }
    if context.vault_id.is_nil()
        || context.service_id.is_nil()
        || context.account_id.is_nil()
        || context.policy_id.is_nil()
        || (context.scheme_version == SCHEME_VERSION_V2 && context.lineage_id.is_nil())
    {
        return Err(validation("EPSCD identifiers must be non-nil"));
    }
    Ok(())
}

fn validation(message: &str) -> KeylessPassError {
    KeylessPassError::Validation(message.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::permutation::Ff1CycleWalking;
    use crate::policy::{CharacterClassConstraint, PolicySpec};
    use std::collections::HashSet;

    fn context() -> CredentialContext {
        CredentialContext {
            scheme_version: SCHEME_VERSION_V1,
            vault_id: Uuid::from_u128(1),
            service_id: Uuid::from_u128(2),
            account_id: Uuid::from_u128(3),
            lineage_id: Uuid::nil(),
            credential_salt: [0x11; 16],
            root_generation: 1,
            policy_id: Uuid::from_u128(4),
            policy_version: 1,
            policy_epoch: 1,
        }
    }

    fn policy() -> CompiledPolicy {
        CompiledPolicy::compile(PolicySpec {
            policy_ir_version: 1,
            min_length: 12,
            max_length: 12,
            alphabet: "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789!@#$"
                .to_string(),
            forbidden_characters: String::new(),
            classes: vec![
                CharacterClassConstraint {
                    name: "upper".to_string(),
                    alphabet: "ABCDEFGHIJKLMNOPQRSTUVWXYZ".to_string(),
                    min_count: 1,
                    max_count: None,
                },
                CharacterClassConstraint {
                    name: "digit".to_string(),
                    alphabet: "0123456789".to_string(),
                    min_count: 1,
                    max_count: None,
                },
            ],
            fixed_characters: Vec::new(),
            fixed_prefix: String::new(),
            fixed_suffix: String::new(),
            forbidden_first_characters: "!@#$".to_string(),
            forbidden_last_characters: "!@#$".to_string(),
            max_total_per_character: None,
            max_identical_run: Some(2),
            max_sequential_run: None,
            forbidden_substrings: Vec::new(),
        })
        .unwrap()
    }

    #[test]
    fn scheme_v1_is_policy_compliant_and_non_repeating() {
        let policy = policy();
        let backend = Ff1CycleWalking::default();
        let mut passwords = HashSet::new();
        for generation in 0..128 {
            let derived =
                derive_password(&[0x22; 32], &context(), generation, &policy, &backend).unwrap();
            assert!(policy.accepts(&derived.password));
            assert!(passwords.insert(derived.password));
        }
    }

    #[test]
    fn generation_is_not_part_of_the_tweak() {
        let policy_hash = policy().spec().policy_hash().unwrap();
        assert_eq!(
            permutation_tweak(&context(), &policy_hash).unwrap(),
            permutation_tweak(&context(), &policy_hash).unwrap()
        );
    }

    #[test]
    fn scheme_v1_fixed_vector_is_stable() {
        let vector: serde_json::Value =
            serde_json::from_str(include_str!("../../test-vectors/epscd-scheme-v1.json")).unwrap();
        let root_key: [u8; 32] = BASE64
            .decode(vector["rootKey"].as_str().unwrap())
            .unwrap()
            .try_into()
            .unwrap();
        let credential_salt: [u8; 16] = BASE64
            .decode(vector["context"]["credentialSalt"].as_str().unwrap())
            .unwrap()
            .try_into()
            .unwrap();
        let context = CredentialContext {
            scheme_version: vector["schemeVersion"].as_u64().unwrap() as u32,
            vault_id: Uuid::parse_str(vector["context"]["vaultID"].as_str().unwrap()).unwrap(),
            service_id: Uuid::parse_str(vector["context"]["serviceID"].as_str().unwrap()).unwrap(),
            account_id: Uuid::parse_str(vector["context"]["accountID"].as_str().unwrap()).unwrap(),
            lineage_id: Uuid::nil(),
            credential_salt,
            root_generation: vector["context"]["rootGeneration"].as_u64().unwrap(),
            policy_id: Uuid::parse_str(vector["context"]["policyID"].as_str().unwrap()).unwrap(),
            policy_version: vector["context"]["policyVersion"].as_u64().unwrap() as u32,
            policy_epoch: vector["context"]["policyEpoch"].as_u64().unwrap(),
        };
        let policy_spec: PolicySpec = serde_json::from_value(vector["policy"].clone()).unwrap();
        let policy_hash = policy_spec.policy_hash().unwrap();
        let policy = CompiledPolicy::compile(policy_spec).unwrap();
        let key = derive_credential_key(&root_key, &context).unwrap();
        let tweak = permutation_tweak(&context, &policy_hash).unwrap();
        let derived = derive_password(
            &root_key,
            &context,
            vector["generation"].as_u64().unwrap(),
            &policy,
            &Ff1CycleWalking::default(),
        )
        .unwrap();
        assert_eq!(BASE64.encode(policy_hash), vector["policyHash"]);
        assert_eq!(BASE64.encode(key), vector["credentialKey"]);
        assert_eq!(
            serde_json::from_slice::<serde_json::Value>(&tweak).unwrap(),
            vector["tweak"]
        );
        assert_eq!(derived.domain_size.to_string(), vector["domainSize"]);
        assert_eq!(derived.rank.to_string(), vector["rank"]);
        assert_eq!(derived.password, vector["password"]);
    }

    #[test]
    fn credential_rekey_changes_kcred_without_relying_on_policy_epoch() {
        let original = context();
        let rekeyed = rekey_credential_context(&original).unwrap();
        assert_eq!(rekeyed.policy_epoch, original.policy_epoch);
        assert_ne!(rekeyed.credential_salt, original.credential_salt);
        assert_ne!(
            derive_credential_key(&[0x22; 32], &original).unwrap(),
            derive_credential_key(&[0x22; 32], &rekeyed).unwrap()
        );
        assert_ne!(
            credential_lineage_id(&original).unwrap(),
            credential_lineage_id(&rekeyed).unwrap()
        );
    }

    #[test]
    fn scheme_v2_binds_lineage_policy_epoch_and_version_into_kcred() {
        let mut left = context();
        left.scheme_version = SCHEME_VERSION_V2;
        left.lineage_id = Uuid::from_u128(5);
        let key = derive_credential_key(&[0x22; 32], &left).unwrap();

        let mut right = left.clone();
        right.lineage_id = Uuid::from_u128(6);
        assert_ne!(key, derive_credential_key(&[0x22; 32], &right).unwrap());
        right = left.clone();
        right.policy_epoch += 1;
        assert_ne!(key, derive_credential_key(&[0x22; 32], &right).unwrap());
        right = left.clone();
        right.policy_version += 1;
        assert_ne!(key, derive_credential_key(&[0x22; 32], &right).unwrap());
    }

    #[test]
    fn collision_bound_is_exact_and_capped() {
        let bound = cross_lineage_collision_union_bound(
            &BigUint::from(1_000_u16),
            &BigUint::from(5_u8),
            &BigUint::from(10_u8),
        )
        .unwrap();
        assert_eq!(bound.numerator, BigUint::from(50_u8));
        assert_eq!(bound.denominator, BigUint::from(1_000_u16));

        let certain = cross_lineage_collision_union_bound(
            &BigUint::from(100_u8),
            &BigUint::from(20_u8),
            &BigUint::from(10_u8),
        )
        .unwrap();
        assert!(certain.is_certain());
    }
}
