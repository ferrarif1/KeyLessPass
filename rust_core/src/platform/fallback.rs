use crate::crypto::{aead, b64_decode, b64_encode, kdf, SecretBytes};
use crate::error::{KeylessPassError, Result};
use crate::platform::PlatformFactorProvider;
use serde::{Deserialize, Serialize};
use std::fs::{self, OpenOptions};
use std::io::{Read, Write};
use std::path::PathBuf;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct FallbackPlatformFactorProvider {
    app_dir: PathBuf,
    platform_name: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ProtectedEnvelope {
    version: u32,
    nonce: String,
    ciphertext: String,
    tag: String,
}

impl FallbackPlatformFactorProvider {
    pub fn new(app_dir: PathBuf, platform_name: impl Into<String>) -> Self {
        Self {
            app_dir,
            platform_name: platform_name.into(),
        }
    }

    fn ensure_dir(&self) -> Result<()> {
        fs::create_dir_all(&self.app_dir)?;
        Ok(())
    }

    fn device_id_path(&self) -> PathBuf {
        self.app_dir
            .join(format!("{}-device-id", self.platform_name))
    }

    fn device_secret_path(&self) -> PathBuf {
        self.app_dir
            .join(format!("{}-device-secret", self.platform_name))
    }

    fn read_or_create_file(
        &self,
        path: PathBuf,
        create: impl FnOnce() -> String,
    ) -> Result<String> {
        self.ensure_dir()?;
        if path.exists() {
            let mut value = String::new();
            OpenOptions::new()
                .read(true)
                .open(&path)?
                .read_to_string(&mut value)?;
            let trimmed = value.trim().to_string();
            if trimmed.is_empty() {
                return Err(KeylessPassError::Validation(format!(
                    "empty provider state file: {}",
                    path.display()
                )));
            }
            return Ok(trimmed);
        }

        let value = create();
        write_private_file(&path, value.as_bytes())?;
        Ok(value)
    }

    fn wrapping_key(&self) -> Result<[u8; 32]> {
        let secret = self.get_or_create_device_secret()?;
        kdf::derive_fallback_package_key(
            secret.expose(),
            format!("{} local package wrapping key", self.platform_name).as_bytes(),
        )
    }
}

impl PlatformFactorProvider for FallbackPlatformFactorProvider {
    fn platform_name(&self) -> String {
        self.platform_name.clone()
    }

    fn get_or_create_device_id(&self) -> Result<String> {
        self.read_or_create_file(self.device_id_path(), || Uuid::new_v4().to_string())
    }

    fn get_or_create_device_secret(&self) -> Result<SecretBytes> {
        let value = self.read_or_create_file(self.device_secret_path(), || {
            b64_encode(&crate::crypto::random_bytes(32))
        })?;
        Ok(SecretBytes::new(b64_decode(&value)?))
    }

    fn protect_local_package(&self, plaintext: &[u8]) -> Result<Vec<u8>> {
        let key = self.wrapping_key()?;
        let (nonce, ciphertext, tag) =
            aead::encrypt(&key, plaintext, self.platform_name.as_bytes())?;
        let envelope = ProtectedEnvelope {
            version: 1,
            nonce: b64_encode(&nonce),
            ciphertext: b64_encode(&ciphertext),
            tag: b64_encode(&tag),
        };
        Ok(serde_json::to_vec(&envelope)?)
    }

    fn unprotect_local_package(&self, protected: &[u8]) -> Result<Vec<u8>> {
        let envelope: ProtectedEnvelope = serde_json::from_slice(protected)?;
        if envelope.version != 1 {
            return Err(KeylessPassError::Validation(
                "unsupported fallback protected envelope version".to_string(),
            ));
        }
        let key = self.wrapping_key()?;
        aead::decrypt(
            &key,
            &b64_decode(&envelope.nonce)?,
            &b64_decode(&envelope.ciphertext)?,
            &b64_decode(&envelope.tag)?,
            self.platform_name.as_bytes(),
        )
    }
}

pub fn write_private_file(path: &PathBuf, bytes: &[u8]) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        let mut file = OpenOptions::new()
            .create(true)
            .truncate(true)
            .write(true)
            .mode(0o600)
            .open(path)?;
        file.write_all(bytes)?;
        Ok(())
    }

    #[cfg(not(unix))]
    {
        let mut file = OpenOptions::new()
            .create(true)
            .truncate(true)
            .write(true)
            .open(path)?;
        file.write_all(bytes)?;
        Ok(())
    }
}
