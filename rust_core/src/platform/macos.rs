use crate::crypto::{aead, b64_decode, b64_encode, kdf, SecretBytes};
use crate::error::{KeylessPassError, Result};
use crate::platform::fallback::FallbackPlatformFactorProvider;
use crate::platform::PlatformFactorProvider;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::process::Command;

#[derive(Debug, Clone)]
pub struct MacOSPlatformFactorProvider {
    fallback: FallbackPlatformFactorProvider,
    use_keychain: bool,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct KeychainEnvelope {
    version: u32,
    nonce: String,
    ciphertext: String,
    tag: String,
}

impl MacOSPlatformFactorProvider {
    pub fn new(app_dir: PathBuf) -> Self {
        Self {
            fallback: FallbackPlatformFactorProvider::new(app_dir, "macos-keychain-fallback"),
            use_keychain: true,
        }
    }

    pub fn fallback_only(app_dir: PathBuf) -> Self {
        Self {
            fallback: FallbackPlatformFactorProvider::new(app_dir, "macos-keychain-fallback"),
            use_keychain: false,
        }
    }

    fn keychain_secret(&self) -> Result<SecretBytes> {
        if !self.use_keychain {
            return self.fallback.get_or_create_device_secret();
        }

        match read_keychain_password() {
            Ok(value) => Ok(SecretBytes::new(b64_decode(value.trim())?)),
            Err(_) => {
                let secret = b64_encode(&crate::crypto::random_bytes(32));
                if write_keychain_password(&secret).is_ok() {
                    Ok(SecretBytes::new(b64_decode(&secret)?))
                } else {
                    self.fallback.get_or_create_device_secret()
                }
            }
        }
    }

    fn keychain_wrapping_key(&self) -> Result<[u8; 32]> {
        if !self.use_keychain {
            return Err(KeylessPassError::MissingFactor(
                "macOS Keychain protection is disabled".to_string(),
            ));
        }
        let secret = match read_keychain_password() {
            Ok(value) => SecretBytes::new(b64_decode(value.trim())?),
            Err(_) => {
                let encoded = b64_encode(&crate::crypto::random_bytes(32));
                write_keychain_password(&encoded)?;
                SecretBytes::new(b64_decode(&encoded)?)
            }
        };
        kdf::derive_fallback_package_key(secret.expose(), b"macOS Keychain local package key")
    }
}

impl PlatformFactorProvider for MacOSPlatformFactorProvider {
    fn platform_name(&self) -> String {
        if self.use_keychain && read_keychain_password().is_ok() {
            "macos-keychain".to_string()
        } else {
            self.fallback.platform_name()
        }
    }

    fn get_or_create_device_id(&self) -> Result<String> {
        self.fallback.get_or_create_device_id()
    }

    fn get_or_create_device_secret(&self) -> Result<SecretBytes> {
        self.keychain_secret()
    }

    fn protect_local_package(&self, plaintext: &[u8]) -> Result<Vec<u8>> {
        if !self.use_keychain {
            return self.fallback.protect_local_package(plaintext);
        }
        let key = self.keychain_wrapping_key()?;
        let (nonce, ciphertext, tag) =
            aead::encrypt(&key, plaintext, b"macos-keychain-local-package")?;
        let envelope = KeychainEnvelope {
            version: 1,
            nonce: b64_encode(&nonce),
            ciphertext: b64_encode(&ciphertext),
            tag: b64_encode(&tag),
        };
        Ok(serde_json::to_vec(&envelope)?)
    }

    fn unprotect_local_package(&self, protected: &[u8]) -> Result<Vec<u8>> {
        if !self.use_keychain {
            return self.fallback.unprotect_local_package(protected);
        }
        let envelope: KeychainEnvelope = serde_json::from_slice(protected)?;
        if envelope.version != 1 {
            return Err(KeylessPassError::Validation(
                "unsupported macOS protected envelope version".to_string(),
            ));
        }
        let key = self.keychain_wrapping_key()?;
        aead::decrypt(
            &key,
            &b64_decode(&envelope.nonce)?,
            &b64_decode(&envelope.ciphertext)?,
            &b64_decode(&envelope.tag)?,
            b"macos-keychain-local-package",
        )
    }
}

fn read_keychain_password() -> Result<String> {
    let output = Command::new("/usr/bin/security")
        .args([
            "find-generic-password",
            "-a",
            "keylesspass",
            "-s",
            "com.keylesspass.local-factor",
            "-w",
        ])
        .output()?;
    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else {
        Err(KeylessPassError::MissingFactor(
            "macOS Keychain secret not found".to_string(),
        ))
    }
}

fn write_keychain_password(secret: &str) -> Result<()> {
    let output = Command::new("/usr/bin/security")
        .args([
            "add-generic-password",
            "-U",
            "-a",
            "keylesspass",
            "-s",
            "com.keylesspass.local-factor",
            "-w",
            secret,
        ])
        .output()?;
    if output.status.success() {
        Ok(())
    } else {
        Err(KeylessPassError::Crypto(
            "failed to store macOS Keychain secret".to_string(),
        ))
    }
}

#[cfg(target_os = "macos")]
pub fn delete_keychain_password() {
    let _ = Command::new("/usr/bin/security")
        .args([
            "delete-generic-password",
            "-a",
            "keylesspass",
            "-s",
            "com.keylesspass.local-factor",
        ])
        .output();
}
