use crate::error::{KeylessPassError, Result};
use hkdf::Hkdf;
use sha2::Sha256;
use unicode_normalization::UnicodeNormalization;
use uuid::Uuid;

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

pub fn normalize_mnemonic(mnemonic: &str) -> String {
    mnemonic
        .nfkc()
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase()
}

pub fn derive_mnemonic_factor(mnemonic: &str, user_id: &Uuid, salt: &[u8]) -> Result<[u8; 32]> {
    let normalized = normalize_mnemonic(mnemonic);
    if normalized.is_empty() {
        return Err(KeylessPassError::MissingFactor(
            "mnemonic phrase is empty".to_string(),
        ));
    }
    let mut ikm = Vec::with_capacity(normalized.len() + 16);
    ikm.extend_from_slice(normalized.as_bytes());
    ikm.extend_from_slice(user_id.as_bytes());
    hkdf_32(&ikm, salt, b"KeylessPass mnemonic factor")
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
    platform: &str,
) -> Result<[u8; 32]> {
    let mut ikm = Vec::new();
    ikm.extend_from_slice(device_secret);
    ikm.extend_from_slice(device_id.as_bytes());
    ikm.extend_from_slice(user_id.as_bytes());
    ikm.extend_from_slice(platform.as_bytes());
    hkdf_32(
        &ikm,
        b"KeylessPass platform factor salt",
        b"KeylessPass platform factor",
    )
}

pub fn derive_password_root(f_m: &[u8], f_c: &[u8], f_u: &[u8]) -> Result<[u8; 32]> {
    let mut ikm = Vec::with_capacity(f_m.len() + f_c.len() + f_u.len());
    ikm.extend_from_slice(f_m);
    ikm.extend_from_slice(f_c);
    ikm.extend_from_slice(f_u);
    hkdf_32(&ikm, b"", b"KeylessPass derivation key")
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

pub fn derive_usb_package_key(f_m: &[u8]) -> Result<[u8; 32]> {
    hkdf_32(
        f_m,
        b"KeylessPass USB package salt",
        b"USB factor package AEAD key",
    )
}

pub fn derive_fallback_package_key(secret: &[u8], label: &[u8]) -> Result<[u8; 32]> {
    hkdf_32(secret, b"KeylessPass fallback local package salt", label)
}
