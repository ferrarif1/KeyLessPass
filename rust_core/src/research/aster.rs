//! ASTER research-only authorization and evaluator boundary.
//!
//! This module implements canonical request binding, signed capabilities,
//! durable use accounting, endpoint secret-inventory instrumentation, and the
//! semantic exact-domain evaluator interface. It is not an MPC implementation:
//! the process-local evaluator holds Root-Epoch keys so that protocol and
//! lifecycle invariants can be tested independently of an MPC backend.

use crate::aster_exact_domain::{self, CredentialContext, DerivedCredential, SCHEME_VERSION_V2};
use crate::permutation::Ff1CycleWalking;
use crate::policy::CompiledPolicy;
use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use rand::RngCore;
use rusqlite::{params, Connection, OptionalExtension, TransactionBehavior};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;
use thiserror::Error;
use uuid::Uuid;
use zeroize::Zeroize;

pub const PROTOCOL_VERSION: u32 = 1;

#[derive(Debug, Error)]
pub enum AsterError {
    #[error("authorization failed: {0}")]
    Authorization(String),
    #[error("root epoch unavailable: {0}")]
    EpochUnavailable(u64),
    #[error("root epoch is still referenced: {0}")]
    EpochReferenced(u64),
    #[error("migration state error: {0}")]
    Migration(String),
    #[error("policy error: {0}")]
    Policy(String),
    #[error("storage error: {0}")]
    Storage(#[from] rusqlite::Error),
    #[error("exact-domain error: {0}")]
    Core(#[from] crate::error::KeylessPassError),
}

pub type Result<T> = std::result::Result<T, AsterError>;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct AsterRequest {
    pub protocol_version: u32,
    pub operation: String,
    pub vault_id: Uuid,
    pub service_id: Uuid,
    pub account_id: Uuid,
    pub lineage_id: Uuid,
    pub credential_salt: [u8; 16],
    pub policy_id: Uuid,
    pub policy_hash: [u8; 32],
    pub policy_epoch: u64,
    pub root_epoch: u64,
    pub generation: u64,
    pub freshness_generation: u64,
    pub expiry_unix_seconds: i64,
    pub nonce: [u8; 16],
    pub use_budget: u32,
}

impl AsterRequest {
    /// Stable, length-delimited binary encoding. Every field carries an
    /// explicit numeric tag and length, so concatenation ambiguity is absent.
    pub fn canonical_bytes(&self) -> Vec<u8> {
        let mut out = b"ASTER-REQUEST\0".to_vec();
        field(&mut out, 1, &self.protocol_version.to_be_bytes());
        field(&mut out, 2, self.operation.as_bytes());
        field(&mut out, 3, self.vault_id.as_bytes());
        field(&mut out, 4, self.service_id.as_bytes());
        field(&mut out, 5, self.account_id.as_bytes());
        field(&mut out, 6, self.lineage_id.as_bytes());
        field(&mut out, 7, &self.credential_salt);
        field(&mut out, 8, self.policy_id.as_bytes());
        field(&mut out, 9, &self.policy_hash);
        field(&mut out, 10, &self.policy_epoch.to_be_bytes());
        field(&mut out, 11, &self.root_epoch.to_be_bytes());
        field(&mut out, 12, &self.generation.to_be_bytes());
        field(&mut out, 13, &self.freshness_generation.to_be_bytes());
        field(&mut out, 14, &self.expiry_unix_seconds.to_be_bytes());
        field(&mut out, 15, &self.nonce);
        field(&mut out, 16, &self.use_budget.to_be_bytes());
        out
    }

    pub fn digest(&self) -> [u8; 32] {
        Sha256::digest(self.canonical_bytes()).into()
    }

    pub fn credential_context(&self) -> CredentialContext {
        CredentialContext {
            scheme_version: SCHEME_VERSION_V2,
            vault_id: self.vault_id,
            service_id: self.service_id,
            account_id: self.account_id,
            lineage_id: self.lineage_id,
            credential_salt: self.credential_salt,
            root_generation: self.root_epoch,
            policy_id: self.policy_id,
            policy_version: 2,
            policy_epoch: self.policy_epoch,
        }
    }
}

fn field(out: &mut Vec<u8>, tag: u16, value: &[u8]) {
    out.extend_from_slice(&tag.to_be_bytes());
    out.extend_from_slice(&(value.len() as u32).to_be_bytes());
    out.extend_from_slice(value);
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Capability {
    pub request: AsterRequest,
    pub signature: Vec<u8>,
}

impl Capability {
    pub fn fingerprint(&self) -> [u8; 32] {
        let mut h = Sha256::new();
        h.update(self.request.canonical_bytes());
        h.update(&self.signature);
        h.finalize().into()
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ScopeMode {
    Exact,
    ProjectedServiceAccount,
    Wildcard,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ValidationChecks {
    pub expiry: bool,
    pub revocation: bool,
    pub nonce_budget: bool,
    pub freshness_generation: bool,
    pub root_epoch: bool,
    pub generation: bool,
    pub lineage: bool,
    pub policy_hash_and_epoch: bool,
}

impl Default for ValidationChecks {
    fn default() -> Self {
        Self {
            expiry: true,
            revocation: true,
            nonce_budget: true,
            freshness_generation: true,
            root_epoch: true,
            generation: true,
            lineage: true,
            policy_hash_and_epoch: true,
        }
    }
}

#[derive(Debug)]
pub struct ApprovalAuthority {
    signing_key: SigningKey,
}

impl ApprovalAuthority {
    pub fn from_seed(seed: [u8; 32]) -> Self {
        Self {
            signing_key: SigningKey::from_bytes(&seed),
        }
    }

    pub fn generate() -> Self {
        let mut seed = [0_u8; 32];
        rand::rngs::OsRng.fill_bytes(&mut seed);
        let authority = Self::from_seed(seed);
        seed.zeroize();
        authority
    }

    pub fn verifying_key(&self) -> VerifyingKey {
        self.signing_key.verifying_key()
    }

    pub fn issue(&self, request: AsterRequest) -> Capability {
        let signature = self.signing_key.sign(&request.canonical_bytes());
        Capability {
            request,
            signature: signature.to_bytes().to_vec(),
        }
    }
}

#[derive(Debug)]
pub struct CapabilityLedger {
    connection: Connection,
}

impl CapabilityLedger {
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        let connection = Connection::open(path)?;
        connection.execute_batch(
            "PRAGMA journal_mode=WAL;
             PRAGMA synchronous=FULL;
             CREATE TABLE IF NOT EXISTS capability_use (
               fingerprint BLOB PRIMARY KEY,
               nonce BLOB NOT NULL,
               used INTEGER NOT NULL,
               budget INTEGER NOT NULL
             );
             CREATE TABLE IF NOT EXISTS revoked_capability (
               fingerprint BLOB PRIMARY KEY
             );",
        )?;
        Ok(Self { connection })
    }

    pub fn in_memory() -> Result<Self> {
        Self::open(":memory:")
    }

    pub fn revoke(&self, capability: &Capability) -> Result<()> {
        self.connection.execute(
            "INSERT OR IGNORE INTO revoked_capability(fingerprint) VALUES (?1)",
            params![capability.fingerprint().as_slice()],
        )?;
        Ok(())
    }

    fn is_revoked(&self, fingerprint: &[u8; 32]) -> Result<bool> {
        Ok(self
            .connection
            .query_row(
                "SELECT 1 FROM revoked_capability WHERE fingerprint=?1",
                params![fingerprint.as_slice()],
                |_| Ok(()),
            )
            .optional()?
            .is_some())
    }

    fn consume(&mut self, capability: &Capability) -> Result<()> {
        let fingerprint = capability.fingerprint();
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let used: Option<u32> = tx
            .query_row(
                "SELECT used FROM capability_use WHERE fingerprint=?1",
                params![fingerprint.as_slice()],
                |row| row.get(0),
            )
            .optional()?;
        let used = used.unwrap_or(0);
        if used >= capability.request.use_budget {
            return Err(AsterError::Authorization(
                "nonce/use budget exhausted".to_string(),
            ));
        }
        tx.execute(
            "INSERT INTO capability_use(fingerprint,nonce,used,budget)
             VALUES (?1,?2,1,?3)
             ON CONFLICT(fingerprint) DO UPDATE SET used=used+1",
            params![
                fingerprint.as_slice(),
                capability.request.nonce.as_slice(),
                capability.request.use_budget
            ],
        )?;
        tx.commit()?;
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SecretInventory {
    pub root_epoch_key_present: bool,
    pub reusable_lineage_key_present: bool,
    pub capability_present: bool,
    pub plaintext_password_present: bool,
}

impl SecretInventory {
    pub fn endpoint_before_request() -> Self {
        Self {
            root_epoch_key_present: false,
            reusable_lineage_key_present: false,
            capability_present: false,
            plaintext_password_present: false,
        }
    }

    pub fn endpoint_during_output() -> Self {
        Self {
            root_epoch_key_present: false,
            reusable_lineage_key_present: false,
            capability_present: true,
            plaintext_password_present: true,
        }
    }

    pub fn endpoint_after_use() -> Self {
        Self {
            root_epoch_key_present: false,
            reusable_lineage_key_present: false,
            capability_present: false,
            plaintext_password_present: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct EpochMetadata {
    pub root_epoch: u64,
    pub sharing_generation: u64,
    pub threshold: usize,
    pub parties: usize,
    pub backend: String,
}

#[derive(Debug)]
struct EpochState {
    root: [u8; 32],
    sharing_generation: u64,
}

impl Drop for EpochState {
    fn drop(&mut self) {
        self.root.zeroize();
    }
}

/// Process-local semantic evaluator. Root material is confined to this service
/// object but is not secret-shared. Do not report it as a threshold/MPC result.
#[derive(Debug)]
pub struct SemanticEvaluator {
    authority_key: VerifyingKey,
    ledger: CapabilityLedger,
    scope_mode: ScopeMode,
    checks: ValidationChecks,
    threshold: usize,
    parties: usize,
    epochs: BTreeMap<u64, EpochState>,
    retired: BTreeSet<u64>,
    permutation: Ff1CycleWalking,
}

impl SemanticEvaluator {
    pub fn new(
        authority_key: VerifyingKey,
        ledger: CapabilityLedger,
        threshold: usize,
        parties: usize,
    ) -> Result<Self> {
        if threshold == 0 || threshold > parties {
            return Err(AsterError::Migration("invalid evaluator threshold".into()));
        }
        Ok(Self {
            authority_key,
            ledger,
            scope_mode: ScopeMode::Exact,
            checks: ValidationChecks::default(),
            threshold,
            parties,
            epochs: BTreeMap::new(),
            retired: BTreeSet::new(),
            permutation: Ff1CycleWalking::default(),
        })
    }

    pub fn set_experiment_validation(&mut self, mode: ScopeMode, checks: ValidationChecks) {
        self.scope_mode = mode;
        self.checks = checks;
    }

    pub fn install_epoch_for_test(&mut self, epoch: u64, root: [u8; 32]) -> Result<EpochMetadata> {
        if self.epochs.contains_key(&epoch) || self.retired.contains(&epoch) {
            return Err(AsterError::Migration("epoch already exists".into()));
        }
        self.epochs.insert(
            epoch,
            EpochState {
                root,
                sharing_generation: 0,
            },
        );
        self.metadata(epoch)
    }

    pub fn create_root_epoch(&mut self, epoch: u64) -> Result<EpochMetadata> {
        let mut root = [0_u8; 32];
        rand::rngs::OsRng.fill_bytes(&mut root);
        self.install_epoch_for_test(epoch, root)
    }

    pub fn refresh_shares(&mut self, epoch: u64) -> Result<EpochMetadata> {
        let state = self
            .epochs
            .get_mut(&epoch)
            .ok_or(AsterError::EpochUnavailable(epoch))?;
        state.sharing_generation += 1;
        self.metadata(epoch)
    }

    pub fn metadata(&self, epoch: u64) -> Result<EpochMetadata> {
        let state = self
            .epochs
            .get(&epoch)
            .ok_or(AsterError::EpochUnavailable(epoch))?;
        Ok(EpochMetadata {
            root_epoch: epoch,
            sharing_generation: state.sharing_generation,
            threshold: self.threshold,
            parties: self.parties,
            backend: "semantic-process-local-not-mpc".to_string(),
        })
    }

    pub fn retire_epoch_unchecked(&mut self, epoch: u64) -> Result<()> {
        self.epochs
            .remove(&epoch)
            .ok_or(AsterError::EpochUnavailable(epoch))?;
        self.retired.insert(epoch);
        Ok(())
    }

    pub fn revoke(&self, capability: &Capability) -> Result<()> {
        self.ledger.revoke(capability)
    }

    pub fn derive(
        &mut self,
        candidate: &AsterRequest,
        capability: &Capability,
        policy: &CompiledPolicy,
        now: i64,
    ) -> Result<DerivedCredential> {
        self.authorize(candidate, capability, now)?;
        if policy.spec().policy_hash()? != candidate.policy_hash {
            return Err(AsterError::Policy(
                "request policy hash does not match compiler".into(),
            ));
        }
        let root = &self
            .epochs
            .get(&candidate.root_epoch)
            .ok_or(AsterError::EpochUnavailable(candidate.root_epoch))?
            .root;
        aster_exact_domain::derive_password(
            root,
            &candidate.credential_context(),
            candidate.generation,
            policy,
            &self.permutation,
        )
        .map_err(Into::into)
    }

    pub fn internal_password(
        &self,
        request: &AsterRequest,
        policy: &CompiledPolicy,
    ) -> Result<String> {
        if policy.spec().policy_hash()? != request.policy_hash {
            return Err(AsterError::Policy(
                "request policy hash does not match compiler".into(),
            ));
        }
        let root = &self
            .epochs
            .get(&request.root_epoch)
            .ok_or(AsterError::EpochUnavailable(request.root_epoch))?
            .root;
        Ok(aster_exact_domain::derive_password(
            root,
            &request.credential_context(),
            request.generation,
            policy,
            &self.permutation,
        )?
        .password)
    }

    pub fn select_migration_candidate(
        &mut self,
        request: &AsterRequest,
        capability: &Capability,
        policy: &CompiledPolicy,
        history: &[(u64, u64)],
        max_candidates: u64,
        now: i64,
    ) -> Result<(u64, String)> {
        self.authorize(request, capability, now)?;
        let mut excluded = BTreeSet::new();
        for (epoch, generation) in history {
            let mut historical = request.clone();
            historical.root_epoch = *epoch;
            historical.generation = *generation;
            excluded.insert(self.internal_password(&historical, policy)?);
        }
        for generation in request.generation..request.generation.saturating_add(max_candidates) {
            let mut candidate = request.clone();
            candidate.generation = generation;
            let password = self.internal_password(&candidate, policy)?;
            if !excluded.contains(&password) {
                return Ok((generation, password));
            }
        }
        Err(AsterError::Policy(
            "no candidate outside authenticated history within budget".into(),
        ))
    }

    fn authorize(
        &mut self,
        candidate: &AsterRequest,
        capability: &Capability,
        now: i64,
    ) -> Result<()> {
        if candidate.protocol_version != PROTOCOL_VERSION {
            return Err(AsterError::Authorization(
                "unsupported protocol version".into(),
            ));
        }
        let signature = Signature::from_slice(&capability.signature)
            .map_err(|_| AsterError::Authorization("malformed signature".into()))?;
        self.authority_key
            .verify(&capability.request.canonical_bytes(), &signature)
            .map_err(|_| AsterError::Authorization("invalid signature".into()))?;
        if self.checks.expiry && now > capability.request.expiry_unix_seconds {
            return Err(AsterError::Authorization("capability expired".into()));
        }
        if self.checks.revocation && self.ledger.is_revoked(&capability.fingerprint())? {
            return Err(AsterError::Authorization("capability revoked".into()));
        }
        if !scope_matches(&capability.request, candidate, self.scope_mode, self.checks) {
            return Err(AsterError::Authorization(
                "capability scope mismatch".into(),
            ));
        }
        if self.checks.nonce_budget {
            self.ledger.consume(capability)?;
        }
        Ok(())
    }
}

fn scope_matches(
    issued: &AsterRequest,
    candidate: &AsterRequest,
    mode: ScopeMode,
    checks: ValidationChecks,
) -> bool {
    if issued.protocol_version != candidate.protocol_version
        || issued.operation != candidate.operation
    {
        return false;
    }
    if mode == ScopeMode::Wildcard {
        return true;
    }
    if issued.vault_id != candidate.vault_id
        || issued.service_id != candidate.service_id
        || issued.account_id != candidate.account_id
        || issued.credential_salt != candidate.credential_salt
        || issued.policy_id != candidate.policy_id
    {
        return false;
    }
    if mode == ScopeMode::ProjectedServiceAccount {
        return true;
    }
    (!checks.lineage || issued.lineage_id == candidate.lineage_id)
        && (!checks.policy_hash_and_epoch
            || (issued.policy_hash == candidate.policy_hash
                && issued.policy_epoch == candidate.policy_epoch))
        && (!checks.root_epoch || issued.root_epoch == candidate.root_epoch)
        && (!checks.generation || issued.generation == candidate.generation)
        && (!checks.freshness_generation
            || issued.freshness_generation == candidate.freshness_generation)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::policy::{CharacterClassConstraint, PolicySpec};

    fn policy() -> CompiledPolicy {
        CompiledPolicy::compile(PolicySpec {
            policy_ir_version: 1,
            min_length: 8,
            max_length: 8,
            alphabet: "ABCDEFGHJKLMNPQRSTUVWXYZ23456789".into(),
            forbidden_characters: String::new(),
            classes: vec![CharacterClassConstraint {
                name: "digit".into(),
                alphabet: "23456789".into(),
                min_count: 1,
                max_count: None,
            }],
            fixed_characters: vec![],
            fixed_prefix: String::new(),
            fixed_suffix: String::new(),
            forbidden_first_characters: String::new(),
            forbidden_last_characters: String::new(),
            max_total_per_character: None,
            max_identical_run: Some(2),
            max_sequential_run: None,
            forbidden_substrings: vec![],
        })
        .unwrap()
    }

    fn request(policy: &CompiledPolicy) -> AsterRequest {
        AsterRequest {
            protocol_version: PROTOCOL_VERSION,
            operation: "derive".into(),
            vault_id: Uuid::from_u128(1),
            service_id: Uuid::from_u128(2),
            account_id: Uuid::from_u128(3),
            lineage_id: Uuid::from_u128(4),
            credential_salt: [5; 16],
            policy_id: Uuid::from_u128(6),
            policy_hash: policy.spec().policy_hash().unwrap(),
            policy_epoch: 7,
            root_epoch: 8,
            generation: 9,
            freshness_generation: 10,
            expiry_unix_seconds: 2_000,
            nonce: [11; 16],
            use_budget: 1,
        }
    }

    fn evaluator(authority: &ApprovalAuthority) -> SemanticEvaluator {
        let mut evaluator = SemanticEvaluator::new(
            authority.verifying_key(),
            CapabilityLedger::in_memory().unwrap(),
            2,
            3,
        )
        .unwrap();
        evaluator.install_epoch_for_test(8, [0x42; 32]).unwrap();
        evaluator
    }

    #[test]
    fn canonical_encoding_and_hash_are_stable() {
        let request = request(&policy());
        assert_eq!(request.canonical_bytes().len(), 308);
        assert_eq!(
            hex(&request.digest()),
            "6f8e6fab7bfc9321be8fd9f529874379c0ad54a554e8196b9b74c15bca0fa10b"
        );
    }

    #[test]
    fn exact_scope_signature_and_replay_checks_hold() {
        let policy = policy();
        let authority = ApprovalAuthority::from_seed([0xA5; 32]);
        let mut evaluator = evaluator(&authority);
        let request = request(&policy);
        let capability = authority.issue(request.clone());
        let first = evaluator
            .derive(&request, &capability, &policy, 1_000)
            .unwrap();
        assert!(policy.rank(&first.password).is_ok());
        assert!(evaluator
            .derive(&request, &capability, &policy, 1_000)
            .is_err());

        let mut cross_service = request.clone();
        cross_service.service_id = Uuid::from_u128(99);
        assert!(evaluator
            .derive(
                &cross_service,
                &authority.issue(request.clone()),
                &policy,
                1_000
            )
            .is_err());

        let mut mutated = authority.issue(request.clone());
        mutated.signature[0] ^= 1;
        assert!(evaluator
            .derive(&request, &mutated, &policy, 1_000)
            .is_err());
    }

    #[test]
    fn all_exact_scope_dimensions_are_bound() {
        let p = policy();
        let authority = ApprovalAuthority::from_seed([0xA5; 32]);
        let base = request(&p);
        let mutations: [fn(&mut AsterRequest); 7] = [
            |r| r.account_id = Uuid::from_u128(100),
            |r| r.lineage_id = Uuid::from_u128(101),
            |r| r.policy_hash[0] ^= 1,
            |r| r.policy_epoch += 1,
            |r| r.root_epoch += 1,
            |r| r.generation += 1,
            |r| r.freshness_generation += 1,
        ];
        for mutate in mutations {
            let mut evaluator = evaluator(&authority);
            let capability = authority.issue(base.clone());
            let mut candidate = base.clone();
            mutate(&mut candidate);
            assert!(evaluator
                .derive(&candidate, &capability, &p, 1_000)
                .is_err());
        }
    }

    #[test]
    fn expiry_and_revocation_hold() {
        let p = policy();
        let authority = ApprovalAuthority::from_seed([0xA5; 32]);
        let request = request(&p);
        let capability = authority.issue(request.clone());
        let mut expired = evaluator(&authority);
        assert!(expired.derive(&request, &capability, &p, 2_001).is_err());
        let mut revoked = evaluator(&authority);
        revoked.revoke(&capability).unwrap();
        assert!(revoked.derive(&request, &capability, &p, 1_000).is_err());
    }

    #[test]
    fn refresh_preserves_family_but_independent_epoch_changes_it() {
        let p = policy();
        let authority = ApprovalAuthority::from_seed([0xA5; 32]);
        let mut evaluator = evaluator(&authority);
        let request = request(&p);
        let before = evaluator.internal_password(&request, &p).unwrap();
        evaluator.refresh_shares(8).unwrap();
        let after = evaluator.internal_password(&request, &p).unwrap();
        assert_eq!(before, after);
        evaluator.install_epoch_for_test(9, [0x24; 32]).unwrap();
        let mut new_epoch = request.clone();
        new_epoch.root_epoch = 9;
        assert_ne!(before, evaluator.internal_password(&new_epoch, &p).unwrap());
    }

    #[test]
    fn history_selection_does_not_release_rejected_values() {
        let p = policy();
        let authority = ApprovalAuthority::from_seed([0xA5; 32]);
        let mut evaluator = evaluator(&authority);
        evaluator.install_epoch_for_test(9, [0x24; 32]).unwrap();
        let mut request = request(&p);
        request.operation = "migrate-select".into();
        request.root_epoch = 9;
        request.generation = 0;
        let history = vec![(8, 0), (8, 1), (8, 2), (8, 3), (8, 4)];
        let capability = authority.issue(request.clone());
        let (generation, candidate) = evaluator
            .select_migration_candidate(&request, &capability, &p, &history, 32, 1_000)
            .unwrap();
        let old: BTreeSet<_> = history
            .iter()
            .map(|(epoch, generation)| {
                let mut r = request.clone();
                r.root_epoch = *epoch;
                r.generation = *generation;
                evaluator.internal_password(&r, &p).unwrap()
            })
            .collect();
        assert!(!old.contains(&candidate));
        assert!(generation < 32);
    }

    fn hex(bytes: &[u8]) -> String {
        bytes.iter().map(|byte| format!("{byte:02x}")).collect()
    }
}
