use crate::error::{KeylessPassError, Result};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use ed25519_dalek::{Signature, Verifier, VerifyingKey};
use std::collections::BTreeMap;

#[derive(Debug, Clone)]
pub struct LicenseVerifier {
    trusted_keys: BTreeMap<String, String>,
}

impl LicenseVerifier {
    pub fn new(keys: impl IntoIterator<Item = (impl Into<String>, impl Into<String>)>) -> Self {
        Self {
            trusted_keys: keys
                .into_iter()
                .map(|(key_id, value)| (key_id.into(), value.into()))
                .collect(),
        }
    }

    pub fn verify(&self, key_id: &str, payload: &[u8], signature_b64url: &str) -> Result<()> {
        let public_key_b64 = self.trusted_keys.get(key_id).ok_or_else(|| {
            KeylessPassError::Validation("unknown license signing key".to_string())
        })?;
        let public_key = b64url_or_standard_decode(public_key_b64)?;
        let signature = b64url_or_standard_decode(signature_b64url)?;
        let public_key: [u8; 32] = public_key.try_into().map_err(|_| {
            KeylessPassError::Validation("license public key must be 32 bytes".to_string())
        })?;
        let signature: [u8; 64] = signature.try_into().map_err(|_| {
            KeylessPassError::Validation("license signature must be 64 bytes".to_string())
        })?;
        let verifying_key = VerifyingKey::from_bytes(&public_key)
            .map_err(|_| KeylessPassError::Crypto("invalid license public key".to_string()))?;
        let signature = Signature::from_bytes(&signature);
        verifying_key
            .verify(payload, &signature)
            .map_err(|_| KeylessPassError::Integrity("license signature mismatch".to_string()))
    }
}

pub fn b64url_encode(bytes: &[u8]) -> String {
    URL_SAFE_NO_PAD.encode(bytes)
}

pub fn b64url_decode(value: &str) -> Result<Vec<u8>> {
    Ok(URL_SAFE_NO_PAD.decode(value)?)
}

fn b64url_or_standard_decode(value: &str) -> Result<Vec<u8>> {
    URL_SAFE_NO_PAD
        .decode(value)
        .or_else(|_| base64::engine::general_purpose::STANDARD.decode(value))
        .map_err(KeylessPassError::from)
}
