use crate::error::{KeylessPassError, Result};
use aes_gcm::aead::{Aead, KeyInit, Payload};
use aes_gcm::{Aes256Gcm, Nonce};

pub const NONCE_LEN: usize = 12;
pub const TAG_LEN: usize = 16;

pub fn encrypt(key: &[u8], plaintext: &[u8], aad: &[u8]) -> Result<(Vec<u8>, Vec<u8>, Vec<u8>)> {
    if key.len() != 32 {
        return Err(KeylessPassError::Crypto(
            "AEAD key must be 32 bytes".to_string(),
        ));
    }
    let nonce = crate::crypto::random_bytes(NONCE_LEN);
    let cipher = Aes256Gcm::new_from_slice(key)
        .map_err(|_| KeylessPassError::Crypto("invalid AEAD key".to_string()))?;
    let mut encrypted = cipher
        .encrypt(
            Nonce::from_slice(&nonce),
            Payload {
                msg: plaintext,
                aad,
            },
        )
        .map_err(|_| KeylessPassError::Crypto("AEAD encryption failed".to_string()))?;
    if encrypted.len() < TAG_LEN {
        return Err(KeylessPassError::Crypto(
            "AEAD ciphertext too short".to_string(),
        ));
    }
    let tag = encrypted.split_off(encrypted.len() - TAG_LEN);
    Ok((nonce, encrypted, tag))
}

pub fn decrypt(
    key: &[u8],
    nonce: &[u8],
    ciphertext: &[u8],
    tag: &[u8],
    aad: &[u8],
) -> Result<Vec<u8>> {
    if key.len() != 32 {
        return Err(KeylessPassError::Crypto(
            "AEAD key must be 32 bytes".to_string(),
        ));
    }
    if nonce.len() != NONCE_LEN {
        return Err(KeylessPassError::Crypto(
            "AEAD nonce must be 12 bytes".to_string(),
        ));
    }
    if tag.len() != TAG_LEN {
        return Err(KeylessPassError::Crypto(
            "AEAD tag must be 16 bytes".to_string(),
        ));
    }
    let cipher = Aes256Gcm::new_from_slice(key)
        .map_err(|_| KeylessPassError::Crypto("invalid AEAD key".to_string()))?;
    let mut encrypted = Vec::with_capacity(ciphertext.len() + tag.len());
    encrypted.extend_from_slice(ciphertext);
    encrypted.extend_from_slice(tag);
    cipher
        .decrypt(
            Nonce::from_slice(nonce),
            Payload {
                msg: &encrypted,
                aad,
            },
        )
        .map_err(|_| KeylessPassError::Crypto("AEAD decryption failed".to_string()))
}
