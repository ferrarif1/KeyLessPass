use crate::crypto::signing::b64url_encode;
use crate::error::{KeylessPassError, Result};
use crate::platform::PlatformFactorProvider;
use crate::storage::StoragePaths;
use ed25519_dalek::{Signer, SigningKey};
use sha2::{Digest, Sha256};
use std::fs;

pub struct DeviceIdentity {
    signing_key: SigningKey,
}

impl DeviceIdentity {
    pub fn load_or_create(
        paths: &StoragePaths,
        provider: &dyn PlatformFactorProvider,
    ) -> Result<Self> {
        let path = paths.app_dir.join("license/device-identity-v2.bin");
        let seed = if path.is_file() {
            provider.unprotect_local_package(&fs::read(path)?)?
        } else {
            let seed = crate::crypto::random_bytes(32);
            let protected = provider.protect_local_package(&seed)?;
            crate::platform::fallback::write_private_file(&path, &protected)?;
            seed
        };
        let seed: [u8; 32] = seed.try_into().map_err(|_| {
            KeylessPassError::Integrity("invalid protected device identity key".to_string())
        })?;
        Ok(Self {
            signing_key: SigningKey::from_bytes(&seed),
        })
    }

    pub fn public_key_b64url(&self) -> String {
        b64url_encode(self.signing_key.verifying_key().as_bytes())
    }

    pub fn key_id(&self) -> String {
        let digest = Sha256::digest(self.signing_key.verifying_key().as_bytes());
        digest.iter().map(|byte| format!("{byte:02x}")).collect()
    }

    pub fn sign_b64url(&self, message: &[u8]) -> String {
        b64url_encode(&self.signing_key.sign(message).to_bytes())
    }

    pub fn prove_possession(&self) -> Result<()> {
        let challenge = crate::crypto::random_bytes(32);
        let signature = self.signing_key.sign(&challenge);
        use ed25519_dalek::Verifier;
        self.signing_key
            .verifying_key()
            .verify(&challenge, &signature)
            .map_err(|_| KeylessPassError::Integrity("device identity proof failed".to_string()))
    }
}
