use crate::crypto::mac;
use crate::error::{KeylessPassError, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CredentialState {
    Active,
    Retired,
    PendingRotation,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RequiredClass {
    pub name: String,
    pub alphabet: String,
    pub position: Option<usize>,
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
    pub rule_version: u32,
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
                    position: Some(1),
                },
                RequiredClass {
                    name: "lower".to_string(),
                    alphabet: "abcdefghijkmnopqrstuvwxyz".to_string(),
                    position: Some(5),
                },
                RequiredClass {
                    name: "digit".to_string(),
                    alphabet: "23456789".to_string(),
                    position: Some(9),
                },
                RequiredClass {
                    name: "symbol".to_string(),
                    alphabet: "!@#$%*-_=+".to_string(),
                    position: Some(13),
                },
            ],
            fixed_positions: Vec::new(),
            normalization: "none".to_string(),
            forbidden_chars: "\"'`\\/:;?&<>{}[]()|, ".to_string(),
            rule_version: 1,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CredentialDescriptionRecord {
    pub record_id: Uuid,
    pub record_seq: u64,
    pub display_name: String,
    pub service_hint: String,
    pub account_hint: String,
    #[serde(default)]
    pub notes: String,
    pub version: u32,
    pub salt: String,
    pub encoding_descriptor: EncodingDescriptor,
    pub state: CredentialState,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub retired_at: Option<DateTime<Utc>>,
    pub mac_tag: String,
}

impl CredentialDescriptionRecord {
    pub fn new(
        record_seq: u64,
        display_name: impl Into<String>,
        service_hint: impl Into<String>,
        account_hint: impl Into<String>,
        notes: impl Into<String>,
        encoding_descriptor: EncodingDescriptor,
    ) -> Self {
        let now = Utc::now();
        Self {
            record_id: Uuid::new_v4(),
            record_seq,
            display_name: display_name.into(),
            service_hint: service_hint.into(),
            account_hint: account_hint.into(),
            notes: notes.into(),
            version: 1,
            salt: crate::crypto::random_base64(16),
            encoding_descriptor,
            state: CredentialState::Active,
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
        let now = Utc::now();
        Self {
            record_id: previous.record_id,
            record_seq: previous.record_seq,
            display_name: previous.display_name.clone(),
            service_hint: previous.service_hint.clone(),
            account_hint: previous.account_hint.clone(),
            notes: previous.notes.clone(),
            version: previous.version + 1,
            salt: crate::crypto::random_base64(16),
            encoding_descriptor,
            state: CredentialState::PendingRotation,
            created_at: now,
            updated_at: now,
            retired_at: None,
            mac_tag: String::new(),
        }
    }

    pub fn set_mac(&mut self, master_key: &[u8]) -> Result<()> {
        let payload = self.mac_payload()?;
        self.mac_tag = mac::hmac_sha256_base64(&mac::cdr_mac_key(master_key), &payload)?;
        Ok(())
    }

    pub fn verify_mac(&self, master_key: &[u8]) -> Result<()> {
        let payload = self.mac_payload()?;
        let expected = mac::hmac_sha256_base64(&mac::cdr_mac_key(master_key), &payload)?;
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
        self.retired_at = None;
        self.updated_at = Utc::now();
        self.set_mac(master_key)
    }

    fn mac_payload(&self) -> Result<Vec<u8>> {
        let mut copy = self.clone();
        copy.mac_tag.clear();
        Ok(serde_json::to_vec(&copy)?)
    }
}
