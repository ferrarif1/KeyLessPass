use crate::crypto::{b64_decode, b64_encode, kdf, mac};
use crate::domain::{
    NetworkRecoveryShareSet, RecoveryAttemptReport, RecoveryFactorType, RecoveryManifest,
    RecoveryMetadata, RecoveryShareSet, ShareEnvelope, SuccessfulRecoveryPair,
    RECOVERY_CRYPTO_SUITE_VERSION, RECOVERY_PHRASE_ENCODING_VERSION, RECOVERY_SCHEME_VERSION,
    RECOVERY_SHARE_COUNT, RECOVERY_THRESHOLD, SHARE_ENVELOPE_SCHEMA_VERSION,
};
use crate::error::{KeylessPassError, Result};
use bip39::Language;
use chrono::{DateTime, Utc};
use rand::rngs::OsRng;
use sha2::{Digest, Sha256};
use uuid::Uuid;
use vsss_rs::Gf256;
use zeroize::Zeroize;

const SHARE_AUTH_LABEL: &[u8] = b"recovery-share-authentication";
const CONFIRMATION_LABEL: &[u8] = b"root-key-confirmation";
const PHRASE_MAGIC: &[u8; 4] = b"KLRP";
const PHRASE_BINARY_LEN: usize = 148;

/// Builds the schema-v1 metadata used by the pairwise-wrapper migration reader.
pub fn build_recovery_metadata(master_key: &[u8], generation: u64) -> Result<RecoveryMetadata> {
    let fragment = mac::hmac_sha256(master_key, b"KeylessPass recovery fragment")?;
    let encrypted_fragment = b64_encode(&fragment);
    let fragment_mac = mac::hmac_sha256_base64(master_key, encrypted_fragment.as_bytes())?;
    Ok(RecoveryMetadata::new(
        1,
        encrypted_fragment,
        fragment_mac,
        generation,
    ))
}

#[allow(clippy::too_many_arguments)]
pub fn create_share_set(
    root_key: &[u8; 32],
    vault_id: Uuid,
    root_generation: u64,
    share_set_generation: u64,
    recovery_factor_generation: u64,
    managed_factor_id: &str,
    managed_factor_generation: u64,
    usb_factor_id: &str,
    usb_factor_generation: u64,
) -> Result<RecoveryShareSet> {
    let (mut envelopes, manifest) = create_bound_share_set(
        root_key,
        vault_id,
        root_generation,
        share_set_generation,
        RecoveryFactorType::Recovery,
        recovery_factor_generation,
        managed_factor_id,
        managed_factor_generation,
        usb_factor_id,
        usb_factor_generation,
    )?;
    let recovery = envelopes.remove(0);
    let managed_computer = envelopes.remove(0);
    let usb = envelopes.remove(0);
    Ok(RecoveryShareSet {
        recovery_phrase: encode_recovery_phrase(&recovery)?,
        managed_computer,
        usb,
        manifest,
    })
}

#[allow(clippy::too_many_arguments)]
pub fn create_network_share_set(
    root_key: &[u8; 32],
    vault_id: Uuid,
    root_generation: u64,
    share_set_generation: u64,
    network_factor_generation: u64,
    managed_factor_id: &str,
    managed_factor_generation: u64,
    usb_factor_id: &str,
    usb_factor_generation: u64,
) -> Result<NetworkRecoveryShareSet> {
    let (mut envelopes, manifest) = create_bound_share_set(
        root_key,
        vault_id,
        root_generation,
        share_set_generation,
        RecoveryFactorType::Network,
        network_factor_generation,
        managed_factor_id,
        managed_factor_generation,
        usb_factor_id,
        usb_factor_generation,
    )?;
    let network = envelopes.remove(0);
    let managed_computer = envelopes.remove(0);
    let usb = envelopes.remove(0);
    Ok(NetworkRecoveryShareSet {
        managed_computer,
        usb,
        network,
        manifest,
    })
}

#[allow(clippy::too_many_arguments)]
fn create_bound_share_set(
    root_key: &[u8; 32],
    vault_id: Uuid,
    root_generation: u64,
    share_set_generation: u64,
    first_factor_type: RecoveryFactorType,
    first_factor_generation: u64,
    managed_factor_id: &str,
    managed_factor_generation: u64,
    usb_factor_id: &str,
    usb_factor_generation: u64,
) -> Result<(Vec<ShareEnvelope>, RecoveryManifest)> {
    let shares = Gf256::split_array(
        RECOVERY_THRESHOLD as usize,
        RECOVERY_SHARE_COUNT as usize,
        root_key,
        OsRng,
    )
    .map_err(|error| KeylessPassError::Crypto(format!("Shamir split failed: {error}")))?;
    let share_set_id = Uuid::new_v4();
    let created_at = DateTime::from_timestamp(Utc::now().timestamp(), 0)
        .expect("current UTC timestamp must be representable");
    let first_factor_id = match first_factor_type {
        RecoveryFactorType::Recovery => format!("recovery:{share_set_id}"),
        RecoveryFactorType::Network => format!("network:{share_set_id}"),
        _ => {
            return Err(KeylessPassError::Validation(
                "first recovery factor must be paper recovery or network".to_string(),
            ))
        }
    };
    let factors = [
        (
            first_factor_type,
            first_factor_id.as_str(),
            first_factor_generation,
        ),
        (
            RecoveryFactorType::ManagedComputer,
            managed_factor_id,
            managed_factor_generation,
        ),
        (
            RecoveryFactorType::Usb,
            usb_factor_id,
            usb_factor_generation,
        ),
    ];
    let mut envelopes = Vec::with_capacity(3);
    for (share, (factor_type, factor_id, factor_generation)) in shares.into_iter().zip(factors) {
        let share_index = *share.first().ok_or_else(|| {
            KeylessPassError::Crypto("Shamir library returned an empty share".to_string())
        })?;
        let mut envelope = ShareEnvelope {
            schema_version: SHARE_ENVELOPE_SCHEMA_VERSION,
            scheme_version: RECOVERY_SCHEME_VERSION,
            crypto_suite_version: RECOVERY_CRYPTO_SUITE_VERSION,
            vault_id,
            root_generation,
            share_set_id,
            share_index,
            threshold: RECOVERY_THRESHOLD,
            share_count: RECOVERY_SHARE_COUNT,
            factor_type,
            factor_id: factor_id.to_string(),
            factor_generation,
            created_at,
            share_data: b64_encode(&share),
            encoding_version: RECOVERY_PHRASE_ENCODING_VERSION,
            metadata_mac: String::new(),
        };
        sign_envelope(root_key, &mut envelope)?;
        envelopes.push(envelope);
    }

    let manifest = RecoveryManifest {
        schema_version: SHARE_ENVELOPE_SCHEMA_VERSION,
        scheme_version: RECOVERY_SCHEME_VERSION,
        crypto_suite_version: RECOVERY_CRYPTO_SUITE_VERSION,
        vault_id,
        root_generation,
        share_set_id,
        share_set_generation,
        threshold: RECOVERY_THRESHOLD,
        share_count: RECOVERY_SHARE_COUNT,
        committed_at: created_at,
        key_confirmation_value: key_confirmation_value(root_key, vault_id, root_generation)?,
    };
    Ok((envelopes, manifest))
}

pub fn recover_root_key(
    left: &ShareEnvelope,
    right: &ShareEnvelope,
    manifest: &RecoveryManifest,
) -> Result<[u8; 32]> {
    validate_pair(left, right, manifest)?;
    let mut recovered = Gf256::combine_array(vec![
        validated_share_data(left)?,
        validated_share_data(right)?,
    ])
    .map_err(|error| KeylessPassError::Crypto(format!("Shamir recovery failed: {error}")))?;
    if recovered.len() != 32 {
        recovered.zeroize();
        return Err(KeylessPassError::Integrity(
            "recovered Root Key length is not 256 bits".to_string(),
        ));
    }
    let mut root_key = [0_u8; 32];
    root_key.copy_from_slice(&recovered);
    recovered.zeroize();

    let actual_kcv =
        key_confirmation_value(&root_key, manifest.vault_id, manifest.root_generation)?;
    if !mac::constant_time_eq_b64(&actual_kcv, &manifest.key_confirmation_value)? {
        root_key.zeroize();
        return Err(KeylessPassError::Integrity(
            "Root Key confirmation failed".to_string(),
        ));
    }
    verify_envelope(&root_key, left)?;
    verify_envelope(&root_key, right)?;
    Ok(root_key)
}

pub fn recover_root_key_with_phrase(
    recovery_phrase: &str,
    other: &ShareEnvelope,
    manifest: &RecoveryManifest,
) -> Result<[u8; 32]> {
    let recovery = decode_recovery_phrase(recovery_phrase)?;
    recover_root_key(&recovery, other, manifest)
}

/// Tries every pair when three factors are available. A single successful pair
/// identifies the excluded factor as damaged; two factors cannot be diagnosed.
pub fn recover_root_key_from_available(
    shares: &[ShareEnvelope],
    manifest: &RecoveryManifest,
) -> Result<RecoveryAttemptReport> {
    if !(2..=3).contains(&shares.len()) {
        return Err(KeylessPassError::Validation(
            "recovery requires two or three available shares".to_string(),
        ));
    }
    for (index, share) in shares.iter().enumerate() {
        if shares[index + 1..]
            .iter()
            .any(|other| other.factor_type == share.factor_type)
        {
            return Err(KeylessPassError::Validation(
                "available recovery shares must have distinct factor roles".to_string(),
            ));
        }
    }

    let mut recovered_root = None;
    let mut successful_pairs = Vec::new();
    for left in 0..shares.len() - 1 {
        for right in left + 1..shares.len() {
            if let Ok(candidate) = recover_root_key(&shares[left], &shares[right], manifest) {
                if recovered_root.is_some_and(|root| root != candidate) {
                    return Err(KeylessPassError::Integrity(
                        "different recovery pairs reconstructed different Root Keys".to_string(),
                    ));
                }
                recovered_root = Some(candidate);
                successful_pairs.push(SuccessfulRecoveryPair {
                    left: shares[left].factor_type,
                    right: shares[right].factor_type,
                });
            }
        }
    }
    let root_key = recovered_root.ok_or_else(|| {
        KeylessPassError::Integrity(
            "no available recovery pair passed KCV and MAC checks".to_string(),
        )
    })?;
    let suspected_damaged_factor = if shares.len() == 3 && successful_pairs.len() == 1 {
        shares
            .iter()
            .find(|share| {
                share.factor_type != successful_pairs[0].left
                    && share.factor_type != successful_pairs[0].right
            })
            .map(|share| share.factor_type)
    } else {
        None
    };
    Ok(RecoveryAttemptReport {
        root_key,
        successful_pairs,
        suspected_damaged_factor,
    })
}

pub fn encode_recovery_phrase(envelope: &ShareEnvelope) -> Result<String> {
    if envelope.factor_type != RecoveryFactorType::Recovery {
        return Err(KeylessPassError::Validation(
            "only a recovery-factor share can be encoded as a recovery phrase".to_string(),
        ));
    }
    validate_envelope_shape(envelope)?;
    let share_data = validated_share_data(envelope)?;
    let metadata_mac = b64_decode(&envelope.metadata_mac)?;
    if metadata_mac.len() != 32 {
        return Err(KeylessPassError::Integrity(
            "share metadata MAC must be 256 bits".to_string(),
        ));
    }
    let timestamp = envelope.created_at.timestamp();
    let mut bytes = Vec::with_capacity(PHRASE_BINARY_LEN);
    bytes.extend_from_slice(PHRASE_MAGIC);
    bytes.extend_from_slice(&envelope.schema_version.to_be_bytes());
    bytes.extend_from_slice(&envelope.scheme_version.to_be_bytes());
    bytes.extend_from_slice(&envelope.crypto_suite_version.to_be_bytes());
    bytes.extend_from_slice(&envelope.encoding_version.to_be_bytes());
    bytes.extend_from_slice(envelope.vault_id.as_bytes());
    bytes.extend_from_slice(&envelope.root_generation.to_be_bytes());
    bytes.extend_from_slice(envelope.share_set_id.as_bytes());
    bytes.push(envelope.share_index);
    bytes.push(envelope.threshold);
    bytes.push(envelope.share_count);
    bytes.extend_from_slice(&envelope.factor_generation.to_be_bytes());
    bytes.extend_from_slice(&timestamp.to_be_bytes());
    bytes.extend_from_slice(&share_data);
    bytes.extend_from_slice(&metadata_mac);
    let checksum = Sha256::digest(&bytes);
    bytes.extend_from_slice(&checksum[..4]);
    debug_assert_eq!(bytes.len(), PHRASE_BINARY_LEN);
    Ok(bytes_to_words(&bytes))
}

pub fn decode_recovery_phrase(phrase: &str) -> Result<ShareEnvelope> {
    let bytes = words_to_bytes(phrase)?;
    if bytes.len() != PHRASE_BINARY_LEN || &bytes[..4] != PHRASE_MAGIC {
        return Err(KeylessPassError::Validation(
            "unsupported recovery phrase format".to_string(),
        ));
    }
    let checksum = Sha256::digest(&bytes[..PHRASE_BINARY_LEN - 4]);
    if checksum[..4] != bytes[PHRASE_BINARY_LEN - 4..] {
        return Err(KeylessPassError::Integrity(
            "recovery phrase checksum mismatch".to_string(),
        ));
    }
    let mut offset = 4;
    let schema_version = take_u32(&bytes, &mut offset);
    let scheme_version = take_u32(&bytes, &mut offset);
    let crypto_suite_version = take_u32(&bytes, &mut offset);
    let encoding_version = take_u32(&bytes, &mut offset);
    let vault_id = take_uuid(&bytes, &mut offset)?;
    let root_generation = take_u64(&bytes, &mut offset);
    let share_set_id = take_uuid(&bytes, &mut offset)?;
    let share_index = bytes[offset];
    let threshold = bytes[offset + 1];
    let share_count = bytes[offset + 2];
    offset += 3;
    let factor_generation = take_u64(&bytes, &mut offset);
    let timestamp = take_i64(&bytes, &mut offset);
    let created_at = DateTime::from_timestamp(timestamp, 0).ok_or_else(|| {
        KeylessPassError::Validation("recovery phrase timestamp is invalid".to_string())
    })?;
    let share_data = bytes[offset..offset + 33].to_vec();
    offset += 33;
    let metadata_mac = bytes[offset..offset + 32].to_vec();
    let envelope = ShareEnvelope {
        schema_version,
        scheme_version,
        crypto_suite_version,
        vault_id,
        root_generation,
        share_set_id,
        share_index,
        threshold,
        share_count,
        factor_type: RecoveryFactorType::Recovery,
        factor_id: format!("recovery:{share_set_id}"),
        factor_generation,
        created_at,
        share_data: b64_encode(&share_data),
        encoding_version,
        metadata_mac: b64_encode(&metadata_mac),
    };
    validate_envelope_shape(&envelope)?;
    Ok(envelope)
}

fn sign_envelope(root_key: &[u8; 32], envelope: &mut ShareEnvelope) -> Result<()> {
    let key = recovery_subkey(root_key, envelope, SHARE_AUTH_LABEL)?;
    envelope.metadata_mac = mac::hmac_sha256_base64(&key, &envelope.canonical_mac_payload()?)?;
    Ok(())
}

fn verify_envelope(root_key: &[u8; 32], envelope: &ShareEnvelope) -> Result<()> {
    let key = recovery_subkey(root_key, envelope, SHARE_AUTH_LABEL)?;
    let actual = mac::hmac_sha256_base64(&key, &envelope.canonical_mac_payload()?)?;
    if mac::constant_time_eq_b64(&actual, &envelope.metadata_mac)? {
        Ok(())
    } else {
        Err(KeylessPassError::Integrity(
            "share envelope metadata MAC mismatch".to_string(),
        ))
    }
}

fn recovery_subkey(
    root_key: &[u8; 32],
    envelope: &ShareEnvelope,
    label: &[u8],
) -> Result<[u8; 32]> {
    kdf::derive_vault_subkey(
        root_key,
        &envelope.vault_id,
        envelope.root_generation,
        envelope.crypto_suite_version,
        label,
    )
}

fn key_confirmation_value(
    root_key: &[u8; 32],
    vault_id: Uuid,
    root_generation: u64,
) -> Result<String> {
    let key = kdf::derive_vault_subkey(
        root_key,
        &vault_id,
        root_generation,
        RECOVERY_CRYPTO_SUITE_VERSION,
        CONFIRMATION_LABEL,
    )?;
    let mut context = b"KeyLessPass/root-key-confirmation/v1".to_vec();
    context.extend_from_slice(vault_id.as_bytes());
    context.extend_from_slice(&root_generation.to_be_bytes());
    mac::hmac_sha256_base64(&key, &context)
}

fn validate_pair(
    left: &ShareEnvelope,
    right: &ShareEnvelope,
    manifest: &RecoveryManifest,
) -> Result<()> {
    validate_envelope_shape(left)?;
    validate_envelope_shape(right)?;
    if left.share_index == right.share_index || left.factor_type == right.factor_type {
        return Err(KeylessPassError::Validation(
            "recovery requires two distinct factors and share indices".to_string(),
        ));
    }
    let same_set = left.vault_id == right.vault_id
        && left.root_generation == right.root_generation
        && left.share_set_id == right.share_set_id
        && left.scheme_version == right.scheme_version
        && left.crypto_suite_version == right.crypto_suite_version
        && left.threshold == right.threshold
        && left.share_count == right.share_count;
    if !same_set {
        return Err(KeylessPassError::Integrity(
            "shares belong to different vaults, generations, or share sets".to_string(),
        ));
    }
    let matches_manifest = manifest.schema_version == SHARE_ENVELOPE_SCHEMA_VERSION
        && manifest.scheme_version == RECOVERY_SCHEME_VERSION
        && manifest.crypto_suite_version == RECOVERY_CRYPTO_SUITE_VERSION
        && manifest.vault_id == left.vault_id
        && manifest.root_generation == left.root_generation
        && manifest.share_set_id == left.share_set_id
        && manifest.threshold == RECOVERY_THRESHOLD
        && manifest.share_count == RECOVERY_SHARE_COUNT;
    if !matches_manifest {
        return Err(KeylessPassError::Integrity(
            "shares do not match the committed recovery manifest".to_string(),
        ));
    }
    Ok(())
}

fn validate_envelope_shape(envelope: &ShareEnvelope) -> Result<()> {
    if envelope.schema_version != SHARE_ENVELOPE_SCHEMA_VERSION
        || envelope.scheme_version != RECOVERY_SCHEME_VERSION
        || envelope.crypto_suite_version != RECOVERY_CRYPTO_SUITE_VERSION
        || envelope.encoding_version != RECOVERY_PHRASE_ENCODING_VERSION
    {
        return Err(KeylessPassError::Validation(
            "unsupported share schema, scheme, crypto suite, or encoding version".to_string(),
        ));
    }
    if envelope.threshold != RECOVERY_THRESHOLD
        || envelope.share_count != RECOVERY_SHARE_COUNT
        || !(1..=RECOVERY_SHARE_COUNT).contains(&envelope.share_index)
    {
        return Err(KeylessPassError::Validation(
            "invalid 2-of-3 share parameters".to_string(),
        ));
    }
    if envelope.factor_id.is_empty() || envelope.metadata_mac.is_empty() {
        return Err(KeylessPassError::Validation(
            "share factor identity and metadata MAC are required".to_string(),
        ));
    }
    Ok(())
}

fn validated_share_data(envelope: &ShareEnvelope) -> Result<Vec<u8>> {
    let share = b64_decode(&envelope.share_data)?;
    if share.len() != 33 || share[0] != envelope.share_index {
        return Err(KeylessPassError::Integrity(
            "share payload length or index does not match its envelope".to_string(),
        ));
    }
    Ok(share)
}

fn bytes_to_words(bytes: &[u8]) -> String {
    let words = Language::English.word_list();
    let mut output = Vec::with_capacity((bytes.len() * 8 + 10) / 11);
    let mut accumulator = 0_u32;
    let mut bits = 0_u8;
    for byte in bytes {
        accumulator = (accumulator << 8) | u32::from(*byte);
        bits += 8;
        while bits >= 11 {
            bits -= 11;
            output.push(words[((accumulator >> bits) & 0x7ff) as usize]);
        }
    }
    if bits > 0 {
        output.push(words[((accumulator << (11 - bits)) & 0x7ff) as usize]);
    }
    output.join(" ")
}

fn words_to_bytes(phrase: &str) -> Result<Vec<u8>> {
    let language = Language::English;
    let mut output = Vec::with_capacity(PHRASE_BINARY_LEN);
    let mut accumulator = 0_u32;
    let mut bits = 0_u8;
    for word in phrase.split_whitespace().map(str::to_lowercase) {
        let index = language.find_word(&word).ok_or_else(|| {
            KeylessPassError::Validation(format!("unknown recovery phrase word: {word}"))
        })?;
        accumulator = (accumulator << 11) | u32::from(index);
        bits += 11;
        while bits >= 8 {
            bits -= 8;
            output.push(((accumulator >> bits) & 0xff) as u8);
        }
    }
    if output.len() != PHRASE_BINARY_LEN || (bits > 0 && accumulator & ((1 << bits) - 1) != 0) {
        return Err(KeylessPassError::Validation(
            "recovery phrase has an invalid length or padding".to_string(),
        ));
    }
    Ok(output)
}

fn take_u32(bytes: &[u8], offset: &mut usize) -> u32 {
    let value = u32::from_be_bytes(bytes[*offset..*offset + 4].try_into().unwrap());
    *offset += 4;
    value
}

fn take_u64(bytes: &[u8], offset: &mut usize) -> u64 {
    let value = u64::from_be_bytes(bytes[*offset..*offset + 8].try_into().unwrap());
    *offset += 8;
    value
}

fn take_i64(bytes: &[u8], offset: &mut usize) -> i64 {
    let value = i64::from_be_bytes(bytes[*offset..*offset + 8].try_into().unwrap());
    *offset += 8;
    value
}

fn take_uuid(bytes: &[u8], offset: &mut usize) -> Result<Uuid> {
    let value = Uuid::from_slice(&bytes[*offset..*offset + 16])
        .map_err(|error| KeylessPassError::Validation(format!("invalid phrase UUID: {error}")))?;
    *offset += 16;
    Ok(value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fixed_phrase_vector_is_cross_platform_stable() {
        let envelope = ShareEnvelope {
            schema_version: 3,
            scheme_version: 1,
            crypto_suite_version: 1,
            vault_id: Uuid::from_u128(0x00112233445566778899aabbccddeeff),
            root_generation: 7,
            share_set_id: Uuid::from_u128(0xffeeddccbbaa99887766554433221100),
            share_index: 1,
            threshold: 2,
            share_count: 3,
            factor_type: RecoveryFactorType::Recovery,
            factor_id: "recovery:ffeeddcc-bbaa-9988-7766-554433221100".to_string(),
            factor_generation: 9,
            created_at: DateTime::from_timestamp(1_700_000_000, 0).unwrap(),
            share_data: b64_encode(&[[1_u8].as_slice(), &[0x11_u8; 32]].concat()),
            encoding_version: 1,
            metadata_mac: b64_encode(&[0x22_u8; 32]),
        };
        let expected = include_str!("../../test-vectors/recovery-phrase-v1.txt").trim();
        let actual = encode_recovery_phrase(&envelope).unwrap();
        assert_eq!(actual, expected);
        assert_eq!(decode_recovery_phrase(expected).unwrap(), envelope);
    }

    fn set(root: &[u8; 32], generation: u64) -> RecoveryShareSet {
        create_share_set(
            root,
            Uuid::from_u128(0x102030405060708090a0b0c0d0e0f000),
            generation,
            generation,
            generation,
            "computer-1",
            generation,
            "usb-1",
            generation,
        )
        .unwrap()
    }

    #[test]
    fn every_two_share_combination_recovers_the_same_root() {
        let root = [0x5a; 32];
        let set = set(&root, 1);
        let recovery = decode_recovery_phrase(&set.recovery_phrase).unwrap();
        assert_eq!(
            recover_root_key(&recovery, &set.managed_computer, &set.manifest).unwrap(),
            root
        );
        assert_eq!(
            recover_root_key(&recovery, &set.usb, &set.manifest).unwrap(),
            root
        );
        assert_eq!(
            recover_root_key(&set.managed_computer, &set.usb, &set.manifest).unwrap(),
            root
        );
    }

    #[test]
    fn phrase_round_trip_and_checksum_detect_errors() {
        let set = set(&[7_u8; 32], 1);
        let envelope = decode_recovery_phrase(&set.recovery_phrase).unwrap();
        assert_eq!(
            encode_recovery_phrase(&envelope).unwrap(),
            set.recovery_phrase
        );
        assert_eq!(set.recovery_phrase.split_whitespace().count(), 108);

        let mut words: Vec<_> = set.recovery_phrase.split_whitespace().collect();
        words[20] = if words[20] == "abandon" {
            "ability"
        } else {
            "abandon"
        };
        assert!(decode_recovery_phrase(&words.join(" ")).is_err());
    }

    #[test]
    fn rejects_cross_set_generation_vault_and_metadata_tampering() {
        let root = [9_u8; 32];
        let set_a = set(&root, 1);
        let set_b = set(&root, 1);
        assert!(recover_root_key(&set_a.managed_computer, &set_b.usb, &set_a.manifest).is_err());

        let mut generation = set_a.usb.clone();
        generation.root_generation += 1;
        assert!(recover_root_key(&set_a.managed_computer, &generation, &set_a.manifest).is_err());

        let mut vault = set_a.usb.clone();
        vault.vault_id = Uuid::new_v4();
        assert!(recover_root_key(&set_a.managed_computer, &vault, &set_a.manifest).is_err());

        let mut metadata = set_a.usb.clone();
        metadata.factor_id.push_str("-tampered");
        assert!(recover_root_key(&set_a.managed_computer, &metadata, &set_a.manifest).is_err());
    }

    #[test]
    fn old_threshold_shares_cannot_open_a_rotated_root_generation() {
        let old = set(&[1_u8; 32], 1);
        let new = set(&[2_u8; 32], 2);
        assert!(recover_root_key(&old.managed_computer, &old.usb, &new.manifest).is_err());
        assert_eq!(
            recover_root_key(&new.managed_computer, &new.usb, &new.manifest).unwrap(),
            [2_u8; 32]
        );
    }

    #[test]
    fn property_all_pairs_recover_for_many_root_keys() {
        for case in 0_u64..64 {
            let digest = Sha256::digest(case.to_be_bytes());
            let mut root = [0_u8; 32];
            root.copy_from_slice(&digest);
            let set = set(&root, case + 1);
            let recovery = decode_recovery_phrase(&set.recovery_phrase).unwrap();
            for (left, right) in [
                (&recovery, &set.managed_computer),
                (&recovery, &set.usb),
                (&set.managed_computer, &set.usb),
            ] {
                assert_eq!(recover_root_key(left, right, &set.manifest).unwrap(), root);
            }
        }
    }

    #[test]
    fn all_pair_recovery_identifies_one_damaged_factor() {
        let root = [0x39_u8; 32];
        let set = set(&root, 1);
        let recovery = decode_recovery_phrase(&set.recovery_phrase).unwrap();
        let complete = recover_root_key_from_available(
            &[
                recovery.clone(),
                set.managed_computer.clone(),
                set.usb.clone(),
            ],
            &set.manifest,
        )
        .unwrap();
        assert_eq!(complete.root_key, root);
        assert_eq!(complete.successful_pairs.len(), 3);
        assert_eq!(complete.suspected_damaged_factor, None);

        let mut damaged_usb = set.usb.clone();
        let mut share = b64_decode(&damaged_usb.share_data).unwrap();
        share[10] ^= 0x80;
        damaged_usb.share_data = b64_encode(&share);
        let diagnosed = recover_root_key_from_available(
            &[recovery, set.managed_computer, damaged_usb],
            &set.manifest,
        )
        .unwrap();
        assert_eq!(diagnosed.root_key, root);
        assert_eq!(diagnosed.successful_pairs.len(), 1);
        assert_eq!(
            diagnosed.suspected_damaged_factor,
            Some(RecoveryFactorType::Usb)
        );
    }
}
