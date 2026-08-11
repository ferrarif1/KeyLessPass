use crate::crypto::mac;
use crate::domain::rotation::{RotationContract, RotationEvidence};
use crate::error::{KeylessPassError, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use uuid::Uuid;

pub const CDR_SCHEMA_VERSION: u32 = 3;
pub const CDR_CRYPTO_SUITE_VERSION: u32 = 1;
pub const CDR_ENCODER_VERSION: u32 = 2;
pub const CDR_DERIVATION_VERSION: u32 = 2;
pub const CDR_ENCODER_VERSION_V3: u32 = 3;
pub const CDR_DERIVATION_VERSION_V3: u32 = 3;

fn default_one() -> u32 {
    1
}

fn default_generation() -> u64 {
    1
}

fn nil_uuid() -> Uuid {
    Uuid::nil()
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CredentialState {
    Active,
    Retired,
    PendingRotation,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum RotationState {
    #[default]
    Stable,
    Prepared,
    UpdateSent,
    RemoteConfirmed,
    LocalCommitted,
    UnknownOutcome,
    ReconciliationRequired,
    EvidenceInsufficient,
    AmbiguousRemoteState,
    OverlapEstablished,
    OldRevocationSent,
    OldRevocationUnknown,
    RollbackRequired,
    Aborted,
    Superseded,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ReplicaMetadata {
    pub replica_id: Uuid,
    pub lamport_clock: u64,
    pub epoch: u64,
}

impl Default for ReplicaMetadata {
    fn default() -> Self {
        Self {
            replica_id: Uuid::nil(),
            lamport_clock: 0,
            epoch: 0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RequiredClass {
    pub name: String,
    pub alphabet: String,
    pub position: Option<usize>,
    #[serde(default = "default_min_count")]
    pub min_count: usize,
    #[serde(default)]
    pub max_count: Option<usize>,
}

fn default_min_count() -> usize {
    1
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct FixedPosition {
    pub index: usize,
    pub character: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct EncodingDescriptor {
    pub length: usize,
    pub alphabet_profile: String,
    pub allowed_alphabet: String,
    pub required_classes: Vec<RequiredClass>,
    pub fixed_positions: Vec<FixedPosition>,
    pub normalization: String,
    pub forbidden_chars: String,
    #[serde(default)]
    pub forbidden_first_chars: String,
    #[serde(default)]
    pub forbidden_last_chars: String,
    #[serde(default)]
    pub forbid_repeated_characters: bool,
    #[serde(default)]
    pub forbid_sequential_characters: bool,
    #[serde(default = "default_max_attempts")]
    pub max_attempts: u32,
    pub rule_version: u32,
}

fn default_max_attempts() -> u32 {
    1024
}

impl Default for EncodingDescriptor {
    fn default() -> Self {
        Self {
            length: 18,
            alphabet_profile: "enterprise-balanced".to_string(),
            allowed_alphabet: "ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz23456789!@#$%*-_=+"
                .to_string(),
            required_classes: vec![
                RequiredClass {
                    name: "upper".to_string(),
                    alphabet: "ABCDEFGHJKLMNPQRSTUVWXYZ".to_string(),
                    position: None,
                    min_count: 1,
                    max_count: None,
                },
                RequiredClass {
                    name: "lower".to_string(),
                    alphabet: "abcdefghijkmnopqrstuvwxyz".to_string(),
                    position: None,
                    min_count: 1,
                    max_count: None,
                },
                RequiredClass {
                    name: "digit".to_string(),
                    alphabet: "23456789".to_string(),
                    position: None,
                    min_count: 1,
                    max_count: None,
                },
                RequiredClass {
                    name: "symbol".to_string(),
                    alphabet: "!@#$%*-_=+".to_string(),
                    position: None,
                    min_count: 1,
                    max_count: None,
                },
            ],
            fixed_positions: Vec::new(),
            normalization: "none".to_string(),
            forbidden_chars: "\"'`\\/:;?&<>{}[]()|, ".to_string(),
            forbidden_first_chars: String::new(),
            forbidden_last_chars: String::new(),
            forbid_repeated_characters: false,
            forbid_sequential_characters: false,
            max_attempts: default_max_attempts(),
            rule_version: 2,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CredentialDescriptionRecord {
    #[serde(default = "default_one")]
    pub schema_version: u32,
    #[serde(default = "default_one")]
    pub crypto_suite_version: u32,
    #[serde(default = "nil_uuid")]
    pub vault_id: Uuid,
    pub record_id: Uuid,
    pub record_seq: u64,
    #[serde(default = "nil_uuid")]
    pub service_id: Uuid,
    #[serde(default = "nil_uuid")]
    pub account_id: Uuid,
    #[serde(default = "default_generation")]
    pub credential_generation: u64,
    #[serde(default = "default_generation")]
    pub root_generation: u64,
    #[serde(default = "nil_uuid")]
    pub policy_id: Uuid,
    #[serde(default = "default_one")]
    pub policy_version: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub policy_epoch: Option<u64>,
    #[serde(default = "default_one")]
    pub encoder_version: u32,
    #[serde(default = "default_one")]
    pub derivation_version: u32,
    pub display_name: String,
    pub service_hint: String,
    pub account_hint: String,
    #[serde(default)]
    pub notes: String,
    pub version: u32,
    pub salt: String,
    pub encoding_descriptor: EncodingDescriptor,
    pub state: CredentialState,
    #[serde(default)]
    pub rotation_state: RotationState,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rotation_contract: Option<RotationContract>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rotation_evidence: Option<RotationEvidence>,
    #[serde(default)]
    pub operation_id: Option<Uuid>,
    #[serde(default)]
    pub parent_record_hash: String,
    #[serde(default)]
    pub replica: ReplicaMetadata,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub retired_at: Option<DateTime<Utc>>,
    pub mac_tag: String,
}

impl CredentialDescriptionRecord {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        vault_id: Uuid,
        root_generation: u64,
        record_seq: u64,
        display_name: impl Into<String>,
        service_hint: impl Into<String>,
        account_hint: impl Into<String>,
        notes: impl Into<String>,
        encoding_descriptor: EncodingDescriptor,
    ) -> Self {
        let now = Utc::now();
        Self {
            schema_version: CDR_SCHEMA_VERSION,
            crypto_suite_version: CDR_CRYPTO_SUITE_VERSION,
            vault_id,
            record_id: Uuid::new_v4(),
            record_seq,
            service_id: Uuid::new_v4(),
            account_id: Uuid::new_v4(),
            credential_generation: 1,
            root_generation,
            policy_id: Uuid::new_v4(),
            policy_version: encoding_descriptor.rule_version,
            policy_epoch: None,
            encoder_version: CDR_ENCODER_VERSION,
            derivation_version: CDR_DERIVATION_VERSION,
            display_name: display_name.into(),
            service_hint: service_hint.into(),
            account_hint: account_hint.into(),
            notes: notes.into(),
            version: 1,
            salt: crate::crypto::random_base64(16),
            encoding_descriptor,
            state: CredentialState::Active,
            rotation_state: RotationState::Stable,
            rotation_contract: None,
            rotation_evidence: None,
            operation_id: None,
            parent_record_hash: String::new(),
            replica: ReplicaMetadata {
                replica_id: Uuid::new_v4(),
                lamport_clock: 1,
                epoch: 1,
            },
            created_at: now,
            updated_at: now,
            retired_at: None,
            mac_tag: String::new(),
        }
    }

    pub fn rotation_from(
        previous: &Self,
        encoding_descriptor: EncodingDescriptor,
    ) -> CredentialDescriptionRecord {
        Self::rotation_from_with_contract(
            previous,
            encoding_descriptor,
            RotationContract::OpaqueReplacement,
        )
    }

    pub fn rotation_from_with_contract(
        previous: &Self,
        encoding_descriptor: EncodingDescriptor,
        rotation_contract: RotationContract,
    ) -> CredentialDescriptionRecord {
        let now = Utc::now();
        let parent_record_hash = previous.record_hash().unwrap_or_default();
        Self {
            schema_version: CDR_SCHEMA_VERSION,
            crypto_suite_version: previous.crypto_suite_version,
            vault_id: previous.vault_id,
            record_id: previous.record_id,
            record_seq: previous.record_seq,
            service_id: previous.service_id,
            account_id: previous.account_id,
            credential_generation: previous.credential_generation + 1,
            root_generation: previous.root_generation,
            policy_id: previous.policy_id,
            policy_version: encoding_descriptor.rule_version,
            policy_epoch: previous.policy_epoch,
            encoder_version: CDR_ENCODER_VERSION,
            derivation_version: previous.derivation_version,
            display_name: previous.display_name.clone(),
            service_hint: previous.service_hint.clone(),
            account_hint: previous.account_hint.clone(),
            notes: previous.notes.clone(),
            version: previous.version + 1,
            salt: crate::crypto::random_base64(16),
            encoding_descriptor,
            state: CredentialState::PendingRotation,
            rotation_state: RotationState::Prepared,
            rotation_contract: Some(rotation_contract),
            rotation_evidence: Some(RotationEvidence::default()),
            operation_id: Some(Uuid::new_v4()),
            parent_record_hash,
            replica: ReplicaMetadata {
                replica_id: previous.replica.replica_id,
                lamport_clock: previous.replica.lamport_clock + 1,
                epoch: previous.replica.epoch + 1,
            },
            created_at: now,
            updated_at: now,
            retired_at: None,
            mac_tag: String::new(),
        }
    }

    /// Creates an explicit encoder/derivation-v3 candidate without changing v2 semantics.
    /// The credential salt is stable inside the v3 permutation domain.
    pub fn rotation_to_v3_with_contract(
        previous: &Self,
        encoding_descriptor: EncodingDescriptor,
        rotation_contract: RotationContract,
    ) -> CredentialDescriptionRecord {
        let now = Utc::now();
        let parent_record_hash = previous.record_hash().unwrap_or_default();
        let previous_is_v3 = previous.derivation_version == CDR_DERIVATION_VERSION_V3
            && previous.encoder_version == CDR_ENCODER_VERSION_V3;
        let policy_changed = previous.encoding_descriptor != encoding_descriptor;
        let policy_epoch = if previous_is_v3 {
            previous.policy_epoch.unwrap_or(1) + u64::from(policy_changed)
        } else {
            1
        };
        let credential_generation = if previous_is_v3 && !policy_changed {
            previous.credential_generation + 1
        } else {
            0
        };
        Self {
            schema_version: CDR_SCHEMA_VERSION,
            crypto_suite_version: previous.crypto_suite_version,
            vault_id: previous.vault_id,
            record_id: previous.record_id,
            record_seq: previous.record_seq,
            service_id: previous.service_id,
            account_id: previous.account_id,
            credential_generation,
            root_generation: previous.root_generation,
            policy_id: previous.policy_id,
            policy_version: encoding_descriptor.rule_version,
            policy_epoch: Some(policy_epoch),
            encoder_version: CDR_ENCODER_VERSION_V3,
            derivation_version: CDR_DERIVATION_VERSION_V3,
            display_name: previous.display_name.clone(),
            service_hint: previous.service_hint.clone(),
            account_hint: previous.account_hint.clone(),
            notes: previous.notes.clone(),
            version: previous.version + 1,
            salt: previous.salt.clone(),
            encoding_descriptor,
            state: CredentialState::PendingRotation,
            rotation_state: RotationState::Prepared,
            rotation_contract: Some(rotation_contract),
            rotation_evidence: Some(RotationEvidence::default()),
            operation_id: Some(Uuid::new_v4()),
            parent_record_hash,
            replica: ReplicaMetadata {
                replica_id: previous.replica.replica_id,
                lamport_clock: previous.replica.lamport_clock + 1,
                epoch: previous.replica.epoch + 1,
            },
            created_at: now,
            updated_at: now,
            retired_at: None,
            mac_tag: String::new(),
        }
    }

    pub fn set_mac(&mut self, master_key: &[u8]) -> Result<()> {
        let payload = self.mac_payload()?;
        self.mac_tag = mac::hmac_sha256_base64(&self.authentication_key(master_key)?, &payload)?;
        Ok(())
    }

    pub fn verify_mac(&self, master_key: &[u8]) -> Result<()> {
        let payload = if self.schema_version < CDR_SCHEMA_VERSION {
            self.legacy_mac_payload()?
        } else {
            self.mac_payload()?
        };
        let expected = mac::hmac_sha256_base64(&self.authentication_key(master_key)?, &payload)?;
        if mac::constant_time_eq_b64(&expected, &self.mac_tag)? {
            Ok(())
        } else {
            Err(KeylessPassError::Integrity("CDR MAC mismatch".to_string()))
        }
    }

    pub fn update_display_fields(
        &mut self,
        display_name: String,
        service_hint: String,
        account_hint: String,
        notes: String,
        master_key: &[u8],
    ) -> Result<()> {
        self.display_name = display_name;
        self.service_hint = service_hint;
        self.account_hint = account_hint;
        self.notes = notes;
        self.updated_at = Utc::now();
        self.set_mac(master_key)
    }

    pub fn mark_retired(&mut self, master_key: &[u8]) -> Result<()> {
        self.state = CredentialState::Retired;
        self.retired_at = Some(Utc::now());
        self.updated_at = Utc::now();
        self.set_mac(master_key)
    }

    pub fn mark_active(&mut self, master_key: &[u8]) -> Result<()> {
        self.state = CredentialState::Active;
        self.rotation_state = RotationState::Stable;
        self.rotation_contract = None;
        self.rotation_evidence = None;
        self.retired_at = None;
        self.updated_at = Utc::now();
        self.set_mac(master_key)
    }

    pub fn canonical_bytes(&self) -> Result<Vec<u8>> {
        Ok(serde_json_canonicalizer::to_vec(self)?)
    }

    pub fn record_hash(&self) -> Result<String> {
        Ok(crate::crypto::b64_encode(&Sha256::digest(
            self.canonical_bytes()?,
        )))
    }

    fn authentication_key(&self, master_key: &[u8]) -> Result<Vec<u8>> {
        if self.schema_version < CDR_SCHEMA_VERSION || self.vault_id.is_nil() {
            return Ok(mac::cdr_mac_key(master_key));
        }
        Ok(crate::crypto::kdf::derive_vault_subkey(
            master_key,
            &self.vault_id,
            self.root_generation,
            self.crypto_suite_version,
            b"cdr-authentication",
        )?
        .to_vec())
    }

    fn mac_payload(&self) -> Result<Vec<u8>> {
        let mut copy = self.clone();
        copy.mac_tag.clear();
        Ok(serde_json_canonicalizer::to_vec(&copy)?)
    }

    fn legacy_mac_payload(&self) -> Result<Vec<u8>> {
        #[derive(Serialize)]
        #[serde(rename_all = "camelCase")]
        struct LegacyRequiredClass<'a> {
            name: &'a str,
            alphabet: &'a str,
            position: Option<usize>,
        }
        #[derive(Serialize)]
        #[serde(rename_all = "camelCase")]
        struct LegacyEncodingDescriptor<'a> {
            length: usize,
            alphabet_profile: &'a str,
            allowed_alphabet: &'a str,
            required_classes: Vec<LegacyRequiredClass<'a>>,
            fixed_positions: &'a [FixedPosition],
            normalization: &'a str,
            forbidden_chars: &'a str,
            rule_version: u32,
        }
        #[derive(Serialize)]
        #[serde(rename_all = "camelCase")]
        struct LegacyCdr<'a> {
            record_id: &'a Uuid,
            record_seq: u64,
            display_name: &'a str,
            service_hint: &'a str,
            account_hint: &'a str,
            notes: &'a str,
            version: u32,
            salt: &'a str,
            encoding_descriptor: LegacyEncodingDescriptor<'a>,
            state: &'a CredentialState,
            created_at: &'a DateTime<Utc>,
            updated_at: &'a DateTime<Utc>,
            retired_at: &'a Option<DateTime<Utc>>,
            mac_tag: &'a str,
        }
        let descriptor = LegacyEncodingDescriptor {
            length: self.encoding_descriptor.length,
            alphabet_profile: &self.encoding_descriptor.alphabet_profile,
            allowed_alphabet: &self.encoding_descriptor.allowed_alphabet,
            required_classes: self
                .encoding_descriptor
                .required_classes
                .iter()
                .map(|class| LegacyRequiredClass {
                    name: &class.name,
                    alphabet: &class.alphabet,
                    position: class.position,
                })
                .collect(),
            fixed_positions: &self.encoding_descriptor.fixed_positions,
            normalization: &self.encoding_descriptor.normalization,
            forbidden_chars: &self.encoding_descriptor.forbidden_chars,
            rule_version: self.encoding_descriptor.rule_version,
        };
        let legacy = LegacyCdr {
            record_id: &self.record_id,
            record_seq: self.record_seq,
            display_name: &self.display_name,
            service_hint: &self.service_hint,
            account_hint: &self.account_hint,
            notes: &self.notes,
            version: self.version,
            salt: &self.salt,
            encoding_descriptor: descriptor,
            state: &self.state,
            created_at: &self.created_at,
            updated_at: &self.updated_at,
            retired_at: &self.retired_at,
            mac_tag: "",
        };
        Ok(serde_json::to_vec(&legacy)?)
    }
}

#[cfg(test)]
mod vector_tests {
    use super::*;

    #[test]
    fn canonical_vector_is_cross_platform_stable() {
        let timestamp = DateTime::parse_from_rfc3339("2026-08-06T00:00:00Z")
            .unwrap()
            .with_timezone(&Utc);
        let mut record = CredentialDescriptionRecord {
            schema_version: 3,
            crypto_suite_version: 1,
            vault_id: Uuid::from_u128(0x00112233445566778899aabbccddeeff),
            record_id: Uuid::from_u128(0x11111111222233334444555555555555),
            record_seq: 42,
            service_id: Uuid::from_u128(0xaaaaaaaa111122223333444444444444),
            account_id: Uuid::from_u128(0xbbbbbbbb111122223333444444444444),
            credential_generation: 3,
            root_generation: 2,
            policy_id: Uuid::from_u128(0xcccccccc111122223333444444444444),
            policy_version: 2,
            policy_epoch: None,
            encoder_version: 2,
            derivation_version: 1,
            display_name: "Example".to_string(),
            service_hint: "legacy.example".to_string(),
            account_hint: "operator".to_string(),
            notes: "vector".to_string(),
            version: 3,
            salt: "EREREREREREREREREREREQ==".to_string(),
            encoding_descriptor: EncodingDescriptor::default(),
            state: CredentialState::Active,
            rotation_state: RotationState::Stable,
            rotation_contract: None,
            rotation_evidence: None,
            operation_id: None,
            parent_record_hash: "parent".to_string(),
            replica: ReplicaMetadata {
                replica_id: Uuid::from_u128(0xdddddddd111122223333444444444444),
                lamport_clock: 8,
                epoch: 5,
            },
            created_at: timestamp,
            updated_at: timestamp,
            retired_at: None,
            mac_tag: String::new(),
        };
        record.set_mac(&[0x5a_u8; 32]).unwrap();
        let expected = include_str!("../../test-vectors/cdr-v3-rfc8785.json").trim();
        assert_eq!(record.canonical_bytes().unwrap(), expected.as_bytes());
        let decoded: CredentialDescriptionRecord = serde_json::from_str(expected).unwrap();
        decoded.verify_mac(&[0x5a_u8; 32]).unwrap();
    }
}
