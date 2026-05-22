use crate::crypto::SecretBytes;
use crate::error::Result;
use crate::platform::fallback::FallbackPlatformFactorProvider;
use crate::platform::PlatformFactorProvider;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct LinuxPlatformFactorProvider {
    fallback: FallbackPlatformFactorProvider,
}

impl LinuxPlatformFactorProvider {
    pub fn new(app_dir: PathBuf) -> Self {
        Self {
            fallback: FallbackPlatformFactorProvider::new(app_dir, "linux-file-aead"),
        }
    }
}

impl PlatformFactorProvider for LinuxPlatformFactorProvider {
    fn platform_name(&self) -> String {
        self.fallback.platform_name()
    }

    fn get_or_create_device_id(&self) -> Result<String> {
        self.fallback.get_or_create_device_id()
    }

    fn get_or_create_device_secret(&self) -> Result<SecretBytes> {
        self.fallback.get_or_create_device_secret()
    }

    fn protect_local_package(&self, plaintext: &[u8]) -> Result<Vec<u8>> {
        self.fallback.protect_local_package(plaintext)
    }

    fn unprotect_local_package(&self, protected: &[u8]) -> Result<Vec<u8>> {
        self.fallback.unprotect_local_package(protected)
    }
}
