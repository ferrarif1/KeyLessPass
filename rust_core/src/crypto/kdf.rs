use crate::error::{KeylessPassError, Result};
use argon2::{Algorithm, Argon2, Params as Argon2Params, Version};
use hkdf::Hkdf;
use pbkdf2::pbkdf2_hmac;
use scrypt::{scrypt, Params as ScryptParams};
use sha2::Sha256;
use sha2::{Digest, Sha256 as Sha256Digest};
use unicode_normalization::UnicodeNormalization;
use uuid::Uuid;
use zeroize::Zeroize;

use crate::domain::{CredentialDescriptionRecord, PasswordDerivationAlgorithm};
use serde::Serialize;

const ARGON2ID_MEMORY_KIB: u32 = 19 * 1024;
const ARGON2ID_ITERATIONS: u32 = 2;
const ARGON2ID_PARALLELISM: u32 = 1;
const SCRYPT_LOG_N: u8 = 15;
const SCRYPT_R: u32 = 8;
const SCRYPT_P: u32 = 1;
const PBKDF2_ITERATIONS: u32 = 210_000;

pub fn hkdf_sha256(
    input_key_material: &[u8],
    salt: &[u8],
    info: &[u8],
    len: usize,
) -> Result<Vec<u8>> {
    let hk = Hkdf::<Sha256>::new(Some(salt), input_key_material);
    let mut out = vec![0_u8; len];
    hk.expand(info, &mut out)
        .map_err(|_| KeylessPassError::Crypto("HKDF expand failed".to_string()))?;
    Ok(out)
}

pub fn hkdf_32(input_key_material: &[u8], salt: &[u8], info: &[u8]) -> Result<[u8; 32]> {
    let bytes = hkdf_sha256(input_key_material, salt, info, 32)?;
    let mut out = [0_u8; 32];
    out.copy_from_slice(&bytes);
    Ok(out)
}

pub fn derive_vault_subkey(
    root_key: &[u8],
    vault_id: &Uuid,
    root_generation: u64,
    crypto_suite_version: u32,
    purpose: &[u8],
) -> Result<[u8; 32]> {
    if root_key.len() != 32 {
        return Err(KeylessPassError::Crypto(
            "root key must contain 256 bits".to_string(),
        ));
    }
    let mut salt_context = Vec::with_capacity(28);
    salt_context.extend_from_slice(vault_id.as_bytes());
    salt_context.extend_from_slice(&root_generation.to_be_bytes());
    salt_context.extend_from_slice(&crypto_suite_version.to_be_bytes());
    let salt = Sha256Digest::digest(
        [
            b"KeyLessPass/vault-subkey/salt/v1".as_slice(),
            &salt_context,
        ]
        .concat(),
    );
    let info = [b"KeyLessPass/vault-subkey/v1/".as_slice(), purpose].concat();
    hkdf_32(root_key, &salt, &info)
}

pub fn normalize_mnemonic(mnemonic: &str) -> String {
    mnemonic
        .nfkc()
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase()
}

pub fn derive_mnemonic_factor(mnemonic: &str, salt: &[u8]) -> Result<[u8; 32]> {
    let normalized = normalize_mnemonic(mnemonic);
    if normalized.is_empty() {
        return Err(KeylessPassError::MissingFactor(
            "mnemonic phrase is empty".to_string(),
        ));
    }
    let params = Argon2Params::new(
        ARGON2ID_MEMORY_KIB,
        ARGON2ID_ITERATIONS,
        ARGON2ID_PARALLELISM,
        Some(32),
    )
    .map_err(|_| KeylessPassError::Crypto("Argon2id parameter error".to_string()))?;
    let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);
    let mut stretched = [0_u8; 32];
    argon2
        .hash_password_into(
            normalized.as_bytes(),
            &mnemonic_factor_kdf_salt(salt),
            &mut stretched,
        )
        .map_err(|_| KeylessPassError::Crypto("mnemonic Argon2id derivation failed".to_string()))?;
    let factor = hkdf_32(&stretched, salt, b"KeyLessPass mnemonic factor v2");
    stretched.zeroize();
    factor
}

pub fn derive_mnemonic_verifier(f_m: &[u8]) -> Result<String> {
    let verifier = hkdf_sha256(
        f_m,
        b"KeylessPass mnemonic verifier salt",
        b"KeylessPass mnemonic verifier",
        32,
    )?;
    Ok(crate::crypto::b64_encode(&verifier))
}

pub fn derive_platform_factor(
    device_secret: &[u8],
    device_id: &str,
    user_id: &Uuid,
    salt: &[u8],
) -> Result<[u8; 32]> {
    let mut ikm = Vec::new();
    ikm.extend_from_slice(device_secret);
    ikm.extend_from_slice(device_id.as_bytes());
    ikm.extend_from_slice(user_id.as_bytes());
    hkdf_32(&ikm, salt, b"KeyLessPass computer factor v2")
}

pub fn derive_usb_factor(
    usb_secret: &[u8],
    usb_id: &str,
    user_id: &Uuid,
    salt: &[u8],
) -> Result<[u8; 32]> {
    let mut ikm = Vec::new();
    ikm.extend_from_slice(usb_secret);
    ikm.extend_from_slice(usb_id.as_bytes());
    ikm.extend_from_slice(user_id.as_bytes());
    hkdf_32(&ikm, salt, b"KeyLessPass USB factor v2")
}

pub fn derive_pairwise_wrap_key(factor_a: &[u8], factor_b: &[u8], label: &str) -> Result<[u8; 32]> {
    let mut ikm = Vec::with_capacity(factor_a.len() + factor_b.len());
    ikm.extend_from_slice(factor_a);
    ikm.extend_from_slice(factor_b);
    hkdf_32(
        &ikm,
        b"KeyLessPass pairwise wrapper salt v2",
        label.as_bytes(),
    )
}

pub fn derive_password_root(f_m: &[u8], f_c: &[u8], f_u: &[u8]) -> Result<[u8; 32]> {
    let mut ikm = Vec::with_capacity(f_m.len() + f_c.len() + f_u.len());
    ikm.extend_from_slice(f_m);
    ikm.extend_from_slice(f_c);
    ikm.extend_from_slice(f_u);
    hkdf_32(&ikm, b"", b"KeylessPass derivation key")
}

pub fn derive_password_root_from_master(master_key: &[u8]) -> Result<[u8; 32]> {
    hkdf_32(master_key, b"", b"KeyLessPass v2 derivation key")
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct PasswordPolicyBinding<'a> {
    policy_id: String,
    policy_version: u32,
    encoding_descriptor: &'a crate::domain::EncodingDescriptor,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct PasswordDerivationInput {
    domain: &'static str,
    service_id: String,
    account_id: String,
    credential_generation: u64,
    credential_salt: String,
    derivation_version: u32,
    encoder_version: u32,
    policy_hash: String,
}

/// Returns the RFC 8785 input authenticated by the v3 password-derivation HMAC.
/// Administrative `record_id` and `record_seq` fields are intentionally excluded.
pub fn password_derivation_input_v3(record: &CredentialDescriptionRecord) -> Result<Vec<u8>> {
    if record.schema_version != crate::domain::CDR_SCHEMA_VERSION
        || record.derivation_version != crate::domain::CDR_DERIVATION_VERSION
        || record.encoder_version != crate::domain::CDR_ENCODER_VERSION
    {
        return Err(KeylessPassError::Validation(
            "unsupported CDR password-derivation version".to_string(),
        ));
    }
    if record.service_id.is_nil() || record.account_id.is_nil() || record.policy_id.is_nil() {
        return Err(KeylessPassError::Validation(
            "v3 derivation requires non-nil service, account, and policy identifiers".to_string(),
        ));
    }
    let credential_salt = crate::crypto::b64_decode(&record.salt)?;
    if credential_salt.len() != 16 {
        return Err(KeylessPassError::Validation(
            "v3 credential salt must contain 128 bits".to_string(),
        ));
    }
    let policy = PasswordPolicyBinding {
        policy_id: record.policy_id.to_string(),
        policy_version: record.policy_version,
        encoding_descriptor: &record.encoding_descriptor,
    };
    let policy_jcs = serde_json_canonicalizer::to_vec(&policy)?;
    let input = PasswordDerivationInput {
        domain: "KeyLessPass/password-derivation-input/v1",
        service_id: record.service_id.to_string(),
        account_id: record.account_id.to_string(),
        credential_generation: record.credential_generation,
        credential_salt: crate::crypto::b64_encode(&credential_salt),
        derivation_version: record.derivation_version,
        encoder_version: record.encoder_version,
        policy_hash: crate::crypto::b64_encode(&Sha256Digest::digest(policy_jcs)),
    };
    Ok(serde_json_canonicalizer::to_vec(&input)?)
}

/// Derives the compatibility seed used only for pre-ASTER credential records.
pub fn derive_service_secret_v3(
    root_key: &[u8; 32],
    record: &CredentialDescriptionRecord,
) -> Result<[u8; 32]> {
    let password_key = derive_vault_subkey(
        root_key,
        &record.vault_id,
        record.root_generation,
        record.crypto_suite_version,
        b"password-derivation",
    )?;
    let seed =
        crate::crypto::mac::hmac_sha256(&password_key, &password_derivation_input_v3(record)?)?;
    seed.try_into()
        .map_err(|_| KeylessPassError::Crypto("password seed length is not 256 bits".to_string()))
}

pub fn derive_service_secret(
    derivation_key: &[u8],
    user_id: &Uuid,
    record_seq: u64,
    record_id: &Uuid,
    version: u32,
    salt: &[u8],
) -> Result<[u8; 32]> {
    let mut info = Vec::new();
    info.extend_from_slice(user_id.as_bytes());
    info.extend_from_slice(&record_seq.to_be_bytes());
    info.extend_from_slice(record_id.as_bytes());
    info.extend_from_slice(&version.to_be_bytes());
    info.extend_from_slice(salt);
    hkdf_32(derivation_key, b"KeylessPass service password salt", &info)
}

pub fn derive_service_secret_with_algorithm(
    algorithm: PasswordDerivationAlgorithm,
    derivation_key: &[u8],
    user_id: &Uuid,
    record_seq: u64,
    record_id: &Uuid,
    version: u32,
    salt: &[u8],
) -> Result<[u8; 32]> {
    let path = service_path_material(user_id, record_seq, record_id, version, salt);
    match algorithm {
        PasswordDerivationAlgorithm::HkdfSha256 => {
            hkdf_32(derivation_key, b"KeylessPass service password salt", &path)
        }
        PasswordDerivationAlgorithm::Argon2id => {
            let params = Argon2Params::new(
                ARGON2ID_MEMORY_KIB,
                ARGON2ID_ITERATIONS,
                ARGON2ID_PARALLELISM,
                Some(32),
            )
            .map_err(|_| KeylessPassError::Crypto("Argon2id parameter error".to_string()))?;
            let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);
            let mut out = [0_u8; 32];
            argon2
                .hash_password_into(derivation_key, &service_kdf_salt(&path), &mut out)
                .map_err(|_| KeylessPassError::Crypto("Argon2id derivation failed".to_string()))?;
            Ok(out)
        }
        PasswordDerivationAlgorithm::Scrypt => {
            let params = ScryptParams::new(SCRYPT_LOG_N, SCRYPT_R, SCRYPT_P, 32)
                .map_err(|_| KeylessPassError::Crypto("scrypt parameter error".to_string()))?;
            let mut out = [0_u8; 32];
            scrypt(derivation_key, &service_kdf_salt(&path), &params, &mut out)
                .map_err(|_| KeylessPassError::Crypto("scrypt derivation failed".to_string()))?;
            Ok(out)
        }
        PasswordDerivationAlgorithm::Pbkdf2HmacSha256 => {
            let mut out = [0_u8; 32];
            pbkdf2_hmac::<Sha256>(
                derivation_key,
                &service_kdf_salt(&path),
                PBKDF2_ITERATIONS,
                &mut out,
            );
            Ok(out)
        }
    }
}

fn service_path_material(
    user_id: &Uuid,
    record_seq: u64,
    record_id: &Uuid,
    version: u32,
    salt: &[u8],
) -> Vec<u8> {
    let mut info = Vec::new();
    info.extend_from_slice(user_id.as_bytes());
    info.extend_from_slice(&record_seq.to_be_bytes());
    info.extend_from_slice(record_id.as_bytes());
    info.extend_from_slice(&version.to_be_bytes());
    info.extend_from_slice(salt);
    info
}

fn service_kdf_salt(path_material: &[u8]) -> [u8; 32] {
    let mut hasher = Sha256Digest::new();
    hasher.update(b"KeyLessPass service derivation salt v1");
    hasher.update(path_material);
    hasher.finalize().into()
}

fn mnemonic_factor_kdf_salt(salt: &[u8]) -> [u8; 32] {
    let mut hasher = Sha256Digest::new();
    hasher.update(b"KeyLessPass mnemonic Argon2id salt v2");
    hasher.update(salt);
    hasher.finalize().into()
}

pub fn derive_fallback_package_key(secret: &[u8], label: &[u8]) -> Result<[u8; 32]> {
    hkdf_32(secret, b"KeylessPass fallback local package salt", label)
}

#[cfg(test)]
mod v3_vector_tests {
    use super::*;
    use crate::crypto::encoder;
    use crate::domain::EncodingDescriptor;

    fn vector_record() -> CredentialDescriptionRecord {
        let mut record = CredentialDescriptionRecord::new(
            Uuid::from_u128(0x00112233445566778899aabbccddeeff),
            7,
            42,
            "ignored display name",
            "ignored.example",
            "ignored account",
            "ignored note",
            EncodingDescriptor::default(),
        );
        record.record_id = Uuid::from_u128(0x11111111222233334444555555555555);
        record.service_id = Uuid::from_u128(0xaaaaaaaa111122223333444444444444);
        record.account_id = Uuid::from_u128(0xbbbbbbbb111122223333444444444444);
        record.credential_generation = 3;
        record.policy_id = Uuid::from_u128(0xcccccccc111122223333444444444444);
        record.policy_version = 2;
        record.salt = "EREREREREREREREREREREQ==".to_string();
        record
    }

    #[test]
    fn fixed_password_derivation_vector_is_cross_platform_stable() {
        let vector: serde_json::Value = serde_json::from_str(include_str!(
            "../../test-vectors/password-derivation-v2.json"
        ))
        .unwrap();
        let root_key: [u8; 32] = crate::crypto::b64_decode(vector["rootKey"].as_str().unwrap())
            .unwrap()
            .try_into()
            .unwrap();
        let record = vector_record();
        let input = password_derivation_input_v3(&record).unwrap();
        let password_key = derive_vault_subkey(
            &root_key,
            &record.vault_id,
            record.root_generation,
            record.crypto_suite_version,
            b"password-derivation",
        )
        .unwrap();
        let seed = derive_service_secret_v3(&root_key, &record).unwrap();
        let password = encoder::encode_password(&seed, &record.encoding_descriptor).unwrap();
        assert_eq!(
            input,
            serde_json_canonicalizer::to_vec(&vector["derivationInput"]).unwrap()
        );
        assert_eq!(
            crate::crypto::b64_encode(&password_key),
            vector["passwordKey"].as_str().unwrap()
        );
        assert_eq!(
            crate::crypto::b64_encode(&seed),
            vector["seed"].as_str().unwrap()
        );
        assert_eq!(password, vector["password"].as_str().unwrap());

        let mut administrative_copy = record.clone();
        administrative_copy.record_id = Uuid::new_v4();
        administrative_copy.record_seq += 1;
        assert_eq!(
            derive_service_secret_v3(&root_key, &administrative_copy).unwrap(),
            seed
        );
        let mut different_service = record;
        different_service.service_id = Uuid::new_v4();
        assert_ne!(
            derive_service_secret_v3(&root_key, &different_service).unwrap(),
            seed
        );
    }
}
