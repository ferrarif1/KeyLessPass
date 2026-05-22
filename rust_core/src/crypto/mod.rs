pub mod aead;
pub mod encoder;
pub mod kdf;
pub mod mac;
pub mod recovery;

use base64::{engine::general_purpose::STANDARD, Engine};
use rand::{rngs::OsRng, RngCore};
use zeroize::{Zeroize, ZeroizeOnDrop};

#[derive(Clone, Zeroize, ZeroizeOnDrop)]
pub struct SecretBytes(Vec<u8>);

impl SecretBytes {
    pub fn new(bytes: Vec<u8>) -> Self {
        Self(bytes)
    }

    pub fn random_32() -> Self {
        let mut bytes = vec![0_u8; 32];
        OsRng.fill_bytes(&mut bytes);
        Self(bytes)
    }

    pub fn expose(&self) -> &[u8] {
        &self.0
    }

    pub fn into_vec(self) -> Vec<u8> {
        self.0.clone()
    }
}

pub fn random_bytes(len: usize) -> Vec<u8> {
    let mut bytes = vec![0_u8; len];
    OsRng.fill_bytes(&mut bytes);
    bytes
}

pub fn random_base64(len: usize) -> String {
    STANDARD.encode(random_bytes(len))
}

pub fn b64_encode(bytes: &[u8]) -> String {
    STANDARD.encode(bytes)
}

pub fn b64_decode(value: &str) -> crate::error::Result<Vec<u8>> {
    Ok(STANDARD.decode(value)?)
}
