pub mod fallback;
pub mod linux;
pub mod macos;
pub mod windows;

use crate::crypto::SecretBytes;
use crate::error::Result;
use std::path::Path;

pub trait PlatformFactorProvider: Send + Sync {
    fn platform_name(&self) -> String;
    fn get_or_create_device_id(&self) -> Result<String>;
    fn get_or_create_device_secret(&self) -> Result<SecretBytes>;
    fn protect_local_package(&self, plaintext: &[u8]) -> Result<Vec<u8>>;
    fn unprotect_local_package(&self, protected: &[u8]) -> Result<Vec<u8>>;
}

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlatformSecurityStatus {
    pub platform: String,
    pub provider: String,
    pub system_keystore_available: bool,
    pub degraded: bool,
    pub message: String,
}

pub fn current_platform_provider(app_dir: &Path) -> Box<dyn PlatformFactorProvider> {
    #[cfg(target_os = "windows")]
    {
        Box::new(windows::WindowsPlatformFactorProvider::new(
            app_dir.to_path_buf(),
        ))
    }
    #[cfg(target_os = "macos")]
    {
        Box::new(macos::MacOSPlatformFactorProvider::new(
            app_dir.to_path_buf(),
        ))
    }
    #[cfg(all(unix, not(target_os = "macos")))]
    {
        Box::new(linux::LinuxPlatformFactorProvider::new(
            app_dir.to_path_buf(),
        ))
    }
    #[cfg(not(any(target_os = "windows", target_os = "macos", unix)))]
    {
        Box::new(fallback::FallbackPlatformFactorProvider::new(
            app_dir.to_path_buf(),
            "fallback",
        ))
    }
}

pub fn current_security_status(provider: &dyn PlatformFactorProvider) -> PlatformSecurityStatus {
    let platform = provider.platform_name();
    let degraded = platform.contains("fallback") || platform.contains("linux-file-aead");
    PlatformSecurityStatus {
        provider: platform.clone(),
        platform,
        system_keystore_available: !degraded,
        degraded,
        message: if degraded {
            "系统钥匙串不可用或未启用，当前使用本地 AEAD 包和文件权限保护；适合受控企业内网环境，但安全能力低于系统钥匙串/TPM。"
                .to_string()
        } else {
            "已启用系统级本机保护能力。".to_string()
        },
    }
}
