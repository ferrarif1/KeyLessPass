//! Factor-preserving heterogeneous Root-Key recovery research prototype.
//!
//! The network stores only a 3-of-5 sharing of the top-level network share
//! `S_N`. A managed endpoint may request recovery, but release requires two
//! independent Ed25519 approvals bound to one fresh session and its ephemeral
//! X25519 public key. This module deliberately contains no view key, data key,
//! opaque-object scan, threshold OPRF, or service-password value.

use crate::crypto::{aead, recovery::recover_root_key};
use crate::domain::{RecoveryFactorType, RecoveryManifest, ShareEnvelope, SuccessfulRecoveryPair};
use crate::error::{KeylessPassError, Result};
use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use hkdf::Hkdf;
use rand::rngs::OsRng;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use uuid::Uuid;
use vsss_rs::Gf256;
use x25519_dalek::{EphemeralSecret, PublicKey, StaticSecret};
use zeroize::Zeroize;

pub const NETWORK_THRESHOLD: usize = 3;
pub const NETWORK_NODE_COUNT: usize = 5;
pub const REQUIRED_APPROVALS: usize = 2;
pub const MAX_TICKET_LIFETIME_SECONDS: i64 = 15 * 60;
pub const RECOVERY_PURPOSE: &str = "recover_network_root_share";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct NetworkFragmentRecord {
    pub schema_version: u32,
    pub node_id: String,
    pub fragment_index: u8,
    pub vault_id: Uuid,
    pub root_generation: u64,
    pub share_set_id: Uuid,
    pub share_set_generation: u64,
    pub fragment: Vec<u8>,
}

pub fn split_network_share(
    network_share: &ShareEnvelope,
    manifest: &RecoveryManifest,
) -> Result<Vec<NetworkFragmentRecord>> {
    validate_network_share_binding(network_share, manifest)?;
    let payload = serde_json_canonicalizer::to_vec(network_share)?;
    let fragments = Gf256::split_array(NETWORK_THRESHOLD, NETWORK_NODE_COUNT, &payload, OsRng)
        .map_err(|error| {
            KeylessPassError::Crypto(format!("network share split failed: {error}"))
        })?;
    Ok(fragments
        .into_iter()
        .enumerate()
        .map(|(index, fragment)| NetworkFragmentRecord {
            schema_version: 1,
            node_id: format!("recovery-node-{}", index + 1),
            fragment_index: fragment[0],
            vault_id: manifest.vault_id,
            root_generation: manifest.root_generation,
            share_set_id: manifest.share_set_id,
            share_set_generation: manifest.share_set_generation,
            fragment,
        })
        .collect())
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RecoveryApproval {
    pub approver_id: String,
    pub signature: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RecoveryAuthorizationTicket {
    pub schema_version: u32,
    pub vault_id: Uuid,
    pub root_generation: u64,
    pub share_set_id: Uuid,
    pub share_set_generation: u64,
    pub op_id: Uuid,
    pub ephemeral_recovery_public_key: [u8; 32],
    pub expires_at: i64,
    pub issued_at: i64,
    pub purpose: String,
    pub authorized_node_ids: Vec<String>,
    pub approvals: Vec<RecoveryApproval>,
}

impl RecoveryAuthorizationTicket {
    pub fn new(
        manifest: &RecoveryManifest,
        ephemeral_recovery_public_key: [u8; 32],
        authorized_node_ids: Vec<String>,
        now: i64,
        lifetime_seconds: i64,
    ) -> Result<Self> {
        if !(1..=MAX_TICKET_LIFETIME_SECONDS).contains(&lifetime_seconds) {
            return Err(validation(
                "recovery ticket lifetime is outside the allowed range",
            ));
        }
        let unique: BTreeSet<_> = authorized_node_ids.iter().collect();
        if unique.len() < NETWORK_THRESHOLD || unique.len() != authorized_node_ids.len() {
            return Err(validation(
                "recovery ticket requires at least three distinct authorized nodes",
            ));
        }
        Ok(Self {
            schema_version: 1,
            vault_id: manifest.vault_id,
            root_generation: manifest.root_generation,
            share_set_id: manifest.share_set_id,
            share_set_generation: manifest.share_set_generation,
            op_id: Uuid::new_v4(),
            ephemeral_recovery_public_key,
            expires_at: now + lifetime_seconds,
            issued_at: now,
            purpose: RECOVERY_PURPOSE.to_string(),
            authorized_node_ids,
            approvals: Vec::new(),
        })
    }

    fn approval_payload(&self) -> Result<Vec<u8>> {
        let mut copy = self.clone();
        copy.approvals.clear();
        Ok(serde_json_canonicalizer::to_vec(&copy)?)
    }

    fn digest(&self) -> Result<[u8; 32]> {
        Ok(Sha256::digest(serde_json_canonicalizer::to_vec(self)?).into())
    }
}

pub struct RecoveryApprover {
    pub approver_id: String,
    signing_key: SigningKey,
}

impl RecoveryApprover {
    pub fn from_seed(approver_id: impl Into<String>, seed: [u8; 32]) -> Self {
        Self {
            approver_id: approver_id.into(),
            signing_key: SigningKey::from_bytes(&seed),
        }
    }

    pub fn verifying_key(&self) -> [u8; 32] {
        self.signing_key.verifying_key().to_bytes()
    }

    pub fn approve(&self, ticket: &mut RecoveryAuthorizationTicket) -> Result<()> {
        if ticket
            .approvals
            .iter()
            .any(|approval| approval.approver_id == self.approver_id)
        {
            return Err(validation("an approver may sign a ticket only once"));
        }
        let signature = self.signing_key.sign(&ticket.approval_payload()?);
        ticket.approvals.push(RecoveryApproval {
            approver_id: self.approver_id.clone(),
            signature: signature.to_bytes().to_vec(),
        });
        Ok(())
    }
}

pub struct RecoveryClientSession {
    secret: StaticSecret,
    pub ticket: RecoveryAuthorizationTicket,
}

impl RecoveryClientSession {
    pub fn begin(
        manifest: &RecoveryManifest,
        authorized_node_ids: Vec<String>,
        now: i64,
        lifetime_seconds: i64,
    ) -> Result<Self> {
        let secret = StaticSecret::random_from_rng(OsRng);
        let public_key = PublicKey::from(&secret).to_bytes();
        let ticket = RecoveryAuthorizationTicket::new(
            manifest,
            public_key,
            authorized_node_ids,
            now,
            lifetime_seconds,
        )?;
        Ok(Self { secret, ticket })
    }

    pub fn open_fragment(&self, response: &SealedNetworkFragment) -> Result<Vec<u8>> {
        if response.op_id != self.ticket.op_id
            || response.vault_id != self.ticket.vault_id
            || response.root_generation != self.ticket.root_generation
            || response.share_set_id != self.ticket.share_set_id
            || response.share_set_generation != self.ticket.share_set_generation
        {
            return Err(integrity(
                "network response is outside the recovery session",
            ));
        }
        let peer = PublicKey::from(response.sender_public_key);
        let shared = self.secret.diffie_hellman(&peer);
        let mut key = release_key(shared.as_bytes(), &self.ticket.digest()?)?;
        let plaintext = aead::decrypt(
            &key,
            &response.nonce,
            &response.ciphertext,
            &response.tag,
            &fragment_aad(&self.ticket, &response.node_id, response.fragment_index)?,
        );
        key.zeroize();
        plaintext
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SealedNetworkFragment {
    pub schema_version: u32,
    pub node_id: String,
    pub fragment_index: u8,
    pub vault_id: Uuid,
    pub root_generation: u64,
    pub share_set_id: Uuid,
    pub share_set_generation: u64,
    pub op_id: Uuid,
    pub sender_public_key: [u8; 32],
    pub nonce: Vec<u8>,
    pub ciphertext: Vec<u8>,
    pub tag: Vec<u8>,
}

#[derive(Default)]
pub struct ReleaseLedger {
    entries: BTreeMap<(String, Uuid), ([u8; 32], SealedNetworkFragment)>,
}

pub struct NetworkRecoveryNode {
    fragment: NetworkFragmentRecord,
    trusted_approvers: BTreeMap<String, VerifyingKey>,
}

impl NetworkRecoveryNode {
    pub fn new(
        fragment: NetworkFragmentRecord,
        trusted_approvers: &[(String, [u8; 32])],
    ) -> Result<Self> {
        let mut keys = BTreeMap::new();
        for (id, bytes) in trusted_approvers {
            let key = VerifyingKey::from_bytes(bytes)
                .map_err(|_| validation("invalid Ed25519 approval public key"))?;
            if keys.insert(id.clone(), key).is_some() {
                return Err(validation("duplicate trusted approver identifier"));
            }
        }
        Ok(Self {
            fragment,
            trusted_approvers: keys,
        })
    }

    pub fn release(
        &self,
        ticket: &RecoveryAuthorizationTicket,
        ledger: &mut ReleaseLedger,
        now: i64,
    ) -> Result<SealedNetworkFragment> {
        self.verify_ticket(ticket, now)?;
        let digest = ticket.digest()?;
        let ledger_key = (self.fragment.node_id.clone(), ticket.op_id);
        if let Some((previous_digest, response)) = ledger.entries.get(&ledger_key) {
            if previous_digest == &digest {
                return Ok(response.clone());
            }
            return Err(integrity(
                "opID was reused for a different recovery session",
            ));
        }

        let sender_secret = EphemeralSecret::random_from_rng(OsRng);
        let sender_public = PublicKey::from(&sender_secret);
        let recipient_public = PublicKey::from(ticket.ephemeral_recovery_public_key);
        let shared = sender_secret.diffie_hellman(&recipient_public);
        if shared.as_bytes() == &[0_u8; 32] {
            return Err(integrity(
                "recovery public key produced an invalid shared secret",
            ));
        }
        let mut key = release_key(shared.as_bytes(), &digest)?;
        let aad = fragment_aad(ticket, &self.fragment.node_id, self.fragment.fragment_index)?;
        let (nonce, ciphertext, tag) = aead::encrypt(&key, &self.fragment.fragment, &aad)?;
        key.zeroize();
        let response = SealedNetworkFragment {
            schema_version: 1,
            node_id: self.fragment.node_id.clone(),
            fragment_index: self.fragment.fragment_index,
            vault_id: self.fragment.vault_id,
            root_generation: self.fragment.root_generation,
            share_set_id: self.fragment.share_set_id,
            share_set_generation: self.fragment.share_set_generation,
            op_id: ticket.op_id,
            sender_public_key: sender_public.to_bytes(),
            nonce,
            ciphertext,
            tag,
        };
        ledger
            .entries
            .insert(ledger_key, (digest, response.clone()));
        Ok(response)
    }

    fn verify_ticket(&self, ticket: &RecoveryAuthorizationTicket, now: i64) -> Result<()> {
        if ticket.schema_version != 1
            || ticket.purpose != RECOVERY_PURPOSE
            || now < ticket.issued_at
            || now > ticket.expires_at
            || ticket.expires_at - ticket.issued_at > MAX_TICKET_LIFETIME_SECONDS
        {
            return Err(validation(
                "recovery ticket is unsupported, premature, or expired",
            ));
        }
        if !ticket
            .authorized_node_ids
            .iter()
            .any(|node| node == &self.fragment.node_id)
        {
            return Err(validation("ticket does not authorize this recovery node"));
        }
        if ticket.vault_id != self.fragment.vault_id
            || ticket.root_generation != self.fragment.root_generation
            || ticket.share_set_id != self.fragment.share_set_id
            || ticket.share_set_generation != self.fragment.share_set_generation
        {
            return Err(integrity(
                "ticket is stale or outside the node's active share set",
            ));
        }

        let payload = ticket.approval_payload()?;
        let mut valid = BTreeSet::new();
        for approval in &ticket.approvals {
            let Some(key) = self.trusted_approvers.get(&approval.approver_id) else {
                continue;
            };
            let Ok(signature) = Signature::from_slice(&approval.signature) else {
                continue;
            };
            if key.verify(&payload, &signature).is_ok() {
                valid.insert(approval.approver_id.as_str());
            }
        }
        if valid.len() < REQUIRED_APPROVALS {
            return Err(KeylessPassError::MissingFactor(
                "network share release requires two independent approvals".to_string(),
            ));
        }
        Ok(())
    }
}

pub fn reconstruct_network_share(
    session: &RecoveryClientSession,
    responses: &[SealedNetworkFragment],
    manifest: &RecoveryManifest,
) -> Result<ShareEnvelope> {
    if responses.len() < NETWORK_THRESHOLD {
        return Err(KeylessPassError::MissingFactor(
            "three network node responses are required".to_string(),
        ));
    }
    let mut indices = BTreeSet::new();
    let mut fragments = Vec::with_capacity(responses.len());
    for response in responses {
        if !indices.insert(response.fragment_index) {
            return Err(validation(
                "network responses require distinct fragment indices",
            ));
        }
        if response.vault_id != manifest.vault_id
            || response.root_generation != manifest.root_generation
            || response.share_set_id != manifest.share_set_id
            || response.share_set_generation != manifest.share_set_generation
        {
            return Err(integrity("network responses mix recovery share sets"));
        }
        fragments.push(session.open_fragment(response)?);
    }
    let mut payload = Gf256::combine_array(&fragments).map_err(|error| {
        KeylessPassError::Crypto(format!("network share combine failed: {error}"))
    })?;
    let share: ShareEnvelope = serde_json::from_slice(&payload)?;
    payload.zeroize();
    validate_network_share_binding(&share, manifest)?;
    Ok(share)
}

pub fn recover_root_with_network_share(
    network_share: &ShareEnvelope,
    local_share: &ShareEnvelope,
    manifest: &RecoveryManifest,
) -> Result<([u8; 32], SuccessfulRecoveryPair)> {
    let root = recover_root_key(network_share, local_share, manifest)?;
    Ok((
        root,
        SuccessfulRecoveryPair {
            left: network_share.factor_type,
            right: local_share.factor_type,
        },
    ))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompromiseDomain {
    ManagedEndpoint,
    OfflineMedium,
    NetworkInfrastructure,
    ApprovalAuthority,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct CompromiseClosure {
    pub has_device_share: bool,
    pub has_offline_share: bool,
    pub can_request_network_release: bool,
    pub compromised_network_nodes: usize,
    pub valid_independent_approvals: usize,
    pub fresh_session: bool,
}

impl CompromiseClosure {
    pub fn network_share_obtainable(self) -> bool {
        self.can_request_network_release
            && self.compromised_network_nodes >= NETWORK_THRESHOLD
            && self.valid_independent_approvals >= REQUIRED_APPROVALS
            && self.fresh_session
    }

    pub fn obtainable_top_level_share_count(self) -> usize {
        usize::from(self.has_device_share)
            + usize::from(self.has_offline_share)
            + usize::from(self.network_share_obtainable())
    }

    pub fn factor_preserved(self) -> bool {
        self.obtainable_top_level_share_count() < 2
    }
}

fn validate_network_share_binding(
    network_share: &ShareEnvelope,
    manifest: &RecoveryManifest,
) -> Result<()> {
    if network_share.factor_type != RecoveryFactorType::Network
        || network_share.vault_id != manifest.vault_id
        || network_share.root_generation != manifest.root_generation
        || network_share.share_set_id != manifest.share_set_id
        || network_share.threshold != manifest.threshold
        || network_share.share_count != manifest.share_count
    {
        return Err(integrity(
            "network share is outside the committed recovery manifest",
        ));
    }
    Ok(())
}

fn release_key(shared_secret: &[u8; 32], ticket_digest: &[u8; 32]) -> Result<[u8; 32]> {
    let hkdf = Hkdf::<Sha256>::new(Some(ticket_digest), shared_secret);
    let mut key = [0_u8; 32];
    hkdf.expand(b"KeyLessPass/network-fragment-release/v1", &mut key)
        .map_err(|_| KeylessPassError::Crypto("network release HKDF failed".to_string()))?;
    Ok(key)
}

fn fragment_aad(
    ticket: &RecoveryAuthorizationTicket,
    node_id: &str,
    fragment_index: u8,
) -> Result<Vec<u8>> {
    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    struct Binding<'a> {
        ticket_digest: [u8; 32],
        node_id: &'a str,
        fragment_index: u8,
    }
    Ok(serde_json_canonicalizer::to_vec(&Binding {
        ticket_digest: ticket.digest()?,
        node_id,
        fragment_index,
    })?)
}

fn validation(message: &str) -> KeylessPassError {
    KeylessPassError::Validation(message.to_string())
}

fn integrity(message: &str) -> KeylessPassError {
    KeylessPassError::Integrity(message.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crypto::recovery::create_network_share_set;

    struct Fixture {
        root: [u8; 32],
        set: crate::domain::NetworkRecoveryShareSet,
        approvers: Vec<RecoveryApprover>,
        trusted: Vec<(String, [u8; 32])>,
        records: Vec<NetworkFragmentRecord>,
    }

    impl Fixture {
        fn new(share_set_generation: u64) -> Self {
            let root = [0x42_u8; 32];
            let set = create_network_share_set(
                &root,
                Uuid::from_u128(7),
                1,
                share_set_generation,
                share_set_generation,
                "managed-a",
                share_set_generation,
                "usb-a",
                share_set_generation,
            )
            .unwrap();
            let approvers = vec![
                RecoveryApprover::from_seed("approver-a", [1_u8; 32]),
                RecoveryApprover::from_seed("approver-b", [2_u8; 32]),
                RecoveryApprover::from_seed("approver-c", [3_u8; 32]),
            ];
            let trusted = approvers
                .iter()
                .map(|approver| (approver.approver_id.clone(), approver.verifying_key()))
                .collect();
            let records = split_network_share(&set.network, &set.manifest).unwrap();
            Self {
                root,
                set,
                approvers,
                trusted,
                records,
            }
        }

        fn session(&self, now: i64) -> RecoveryClientSession {
            RecoveryClientSession::begin(
                &self.set.manifest,
                self.records
                    .iter()
                    .map(|record| record.node_id.clone())
                    .collect(),
                now,
                600,
            )
            .unwrap()
        }

        fn nodes(&self) -> Vec<NetworkRecoveryNode> {
            self.records
                .iter()
                .cloned()
                .map(|record| NetworkRecoveryNode::new(record, &self.trusted).unwrap())
                .collect()
        }
    }

    fn approve(fixture: &Fixture, ticket: &mut RecoveryAuthorizationTicket, count: usize) {
        for approver in fixture.approvers.iter().take(count) {
            approver.approve(ticket).unwrap();
        }
    }

    #[test]
    fn authorized_three_of_five_release_recovers_network_share_and_root() {
        let fixture = Fixture::new(1);
        let now = 10_000;
        let mut session = fixture.session(now);
        approve(&fixture, &mut session.ticket, 2);
        let mut ledger = ReleaseLedger::default();
        let responses: Vec<_> = fixture
            .nodes()
            .iter()
            .take(3)
            .map(|node| node.release(&session.ticket, &mut ledger, now).unwrap())
            .collect();
        let network =
            reconstruct_network_share(&session, &responses, &fixture.set.manifest).unwrap();
        let (root, _) = recover_root_with_network_share(
            &network,
            &fixture.set.managed_computer,
            &fixture.set.manifest,
        )
        .unwrap();
        assert_eq!(root, fixture.root);
    }

    #[test]
    fn endpoint_or_copied_usb_without_approval_does_not_obtain_network_share() {
        assert!(CompromiseClosure {
            has_device_share: true,
            can_request_network_release: true,
            ..Default::default()
        }
        .factor_preserved());
        assert!(CompromiseClosure {
            has_offline_share: true,
            can_request_network_release: true,
            ..Default::default()
        }
        .factor_preserved());
    }

    #[test]
    fn one_approval_fewer_than_three_nodes_and_expired_ticket_are_rejected() {
        let fixture = Fixture::new(1);
        let now = 10_000;
        let mut session = fixture.session(now);
        fixture.approvers[1].approve(&mut session.ticket).unwrap();
        let nodes = fixture.nodes();
        let mut ledger = ReleaseLedger::default();
        assert!(nodes[0].release(&session.ticket, &mut ledger, now).is_err());
        approve(&fixture, &mut session.ticket, 1);
        let responses: Vec<_> = nodes
            .iter()
            .take(2)
            .map(|node| node.release(&session.ticket, &mut ledger, now).unwrap())
            .collect();
        assert!(reconstruct_network_share(&session, &responses, &fixture.set.manifest).is_err());
        assert!(nodes[2]
            .release(&session.ticket, &mut ledger, session.ticket.expires_at + 1)
            .is_err());
    }

    #[test]
    fn ticket_is_bound_to_ephemeral_key_and_op_id_is_idempotent_only_for_same_ticket() {
        let fixture = Fixture::new(1);
        let now = 10_000;
        let mut session = fixture.session(now);
        approve(&fixture, &mut session.ticket, 2);
        let node = fixture.nodes().remove(0);
        let mut ledger = ReleaseLedger::default();
        let first = node.release(&session.ticket, &mut ledger, now).unwrap();
        assert_eq!(
            node.release(&session.ticket, &mut ledger, now).unwrap(),
            first
        );

        let mut rebound = session.ticket.clone();
        rebound.ephemeral_recovery_public_key[0] ^= 1;
        assert!(node.release(&rebound, &mut ledger, now).is_err());

        let mut changed = session.ticket.clone();
        changed.approvals.clear();
        changed.expires_at -= 1;
        approve(&fixture, &mut changed, 2);
        assert!(node.release(&changed, &mut ledger, now).is_err());
    }

    #[test]
    fn stale_share_set_and_mixed_generation_responses_are_rejected() {
        let old = Fixture::new(1);
        let new = Fixture::new(2);
        let now = 10_000;
        let mut old_session = old.session(now);
        approve(&old, &mut old_session.ticket, 2);
        let mut ledger = ReleaseLedger::default();
        let current_node = NetworkRecoveryNode::new(new.records[0].clone(), &old.trusted).unwrap();
        assert!(current_node
            .release(&old_session.ticket, &mut ledger, now)
            .is_err());

        let mut new_ticket = old_session.ticket.clone();
        new_ticket.share_set_id = new.set.manifest.share_set_id;
        new_ticket.share_set_generation = 2;
        new_ticket.approvals.clear();
        approve(&old, &mut new_ticket, 2);
        let old_nodes = old.nodes();
        let old_response = old_nodes[0]
            .release(&old_session.ticket, &mut ledger, now)
            .unwrap();
        let new_nodes: Vec<_> = new
            .records
            .iter()
            .cloned()
            .map(|record| NetworkRecoveryNode::new(record, &old.trusted).unwrap())
            .collect();
        let new_response = new_nodes[1].release(&new_ticket, &mut ledger, now).unwrap();
        assert!(reconstruct_network_share(
            &old_session,
            &[old_response.clone(), new_response, old_response],
            &old.set.manifest,
        )
        .is_err());
    }
}
