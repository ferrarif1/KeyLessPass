use crate::error::{KeylessPassError, Result};
use base64::{engine::general_purpose::STANDARD, Engine};
use hmac::{Hmac, Mac};
use sha2::Sha256;

type HmacSha256 = Hmac<Sha256>;

pub fn hmac_sha256(key: &[u8], data: &[u8]) -> Result<Vec<u8>> {
    let mut mac = HmacSha256::new_from_slice(key)
        .map_err(|_| KeylessPassError::Crypto("invalid HMAC key".to_string()))?;
    mac.update(data);
    Ok(mac.finalize().into_bytes().to_vec())
}

pub fn hmac_sha256_base64(key: &[u8], data: &[u8]) -> Result<String> {
    Ok(STANDARD.encode(hmac_sha256(key, data)?))
}

pub fn constant_time_eq_b64(left: &str, right: &str) -> Result<bool> {
    let left = STANDARD.decode(left)?;
    let right = STANDARD.decode(right)?;
    Ok(left.len() == right.len()
        && left
            .iter()
            .zip(right.iter())
            .fold(0_u8, |acc, (a, b)| acc | (a ^ b))
            == 0)
}

pub fn cdr_mac_key(master_key: &[u8]) -> Vec<u8> {
    hmac_sha256(master_key, b"KeylessPass CDR MAC key").unwrap_or_else(|_| vec![0_u8; 32])
}

pub fn package_mac_key(key: &[u8]) -> Vec<u8> {
    hmac_sha256(key, b"KeylessPass factor package MAC key").unwrap_or_else(|_| vec![0_u8; 32])
}

pub fn cdr_backup_mac_key(master_key: &[u8]) -> Vec<u8> {
    hmac_sha256(master_key, b"KeyLessPass USB CDR backup MAC key")
        .unwrap_or_else(|_| vec![0_u8; 32])
}
