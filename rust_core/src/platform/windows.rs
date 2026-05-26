use crate::crypto::SecretBytes;
use crate::error::Result;
use crate::platform::fallback::FallbackPlatformFactorProvider;
use crate::platform::PlatformFactorProvider;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct WindowsPlatformFactorProvider {
    fallback: FallbackPlatformFactorProvider,
}

impl WindowsPlatformFactorProvider {
    pub fn new(app_dir: PathBuf) -> Self {
        Self {
            fallback: FallbackPlatformFactorProvider::new(app_dir, "windows-dpapi-fallback"),
        }
    }
}

impl PlatformFactorProvider for WindowsPlatformFactorProvider {
    fn platform_name(&self) -> String {
        #[cfg(windows)]
        {
            "windows-dpapi".to_string()
        }
        #[cfg(not(windows))]
        {
            self.fallback.platform_name()
        }
    }

    fn get_or_create_device_id(&self) -> Result<String> {
        self.fallback.get_or_create_device_id()
    }

    fn get_or_create_device_secret(&self) -> Result<SecretBytes> {
        self.fallback.get_or_create_device_secret()
    }

    fn protect_local_package(&self, plaintext: &[u8]) -> Result<Vec<u8>> {
        #[cfg(windows)]
        {
            match dpapi_protect(plaintext) {
                Ok(value) => return Ok(value),
                Err(_) => return self.fallback.protect_local_package(plaintext),
            }
        }
        #[cfg(not(windows))]
        {
            self.fallback.protect_local_package(plaintext)
        }
    }

    fn unprotect_local_package(&self, protected: &[u8]) -> Result<Vec<u8>> {
        #[cfg(windows)]
        {
            match dpapi_unprotect(protected) {
                Ok(value) => return Ok(value),
                Err(_) => return self.fallback.unprotect_local_package(protected),
            }
        }
        #[cfg(not(windows))]
        {
            self.fallback.unprotect_local_package(protected)
        }
    }
}

#[cfg(windows)]
fn dpapi_protect(plaintext: &[u8]) -> Result<Vec<u8>> {
    use crate::error::KeylessPassError;
    use std::ptr::{null, null_mut};
    use windows_sys::Win32::Foundation::LocalFree;
    use windows_sys::Win32::Security::Cryptography::{
        CryptProtectData, CRYPTPROTECT_UI_FORBIDDEN, CRYPT_INTEGER_BLOB,
    };

    let mut input = CRYPT_INTEGER_BLOB {
        cbData: plaintext.len() as u32,
        pbData: plaintext.as_ptr() as *mut u8,
    };
    let mut output = CRYPT_INTEGER_BLOB {
        cbData: 0,
        pbData: null_mut(),
    };
    let ok = unsafe {
        CryptProtectData(
            &mut input,
            null(),
            null_mut(),
            null_mut(),
            null_mut(),
            CRYPTPROTECT_UI_FORBIDDEN,
            &mut output,
        )
    };
    if ok == 0 {
        return Err(KeylessPassError::Crypto(
            "Windows DPAPI protect failed".to_string(),
        ));
    }
    let protected =
        unsafe { std::slice::from_raw_parts(output.pbData, output.cbData as usize).to_vec() };
    unsafe {
        LocalFree(output.pbData as _);
    }
    Ok(protected)
}

#[cfg(windows)]
fn dpapi_unprotect(protected: &[u8]) -> Result<Vec<u8>> {
    use crate::error::KeylessPassError;
    use std::ptr::null_mut;
    use windows_sys::Win32::Foundation::LocalFree;
    use windows_sys::Win32::Security::Cryptography::{
        CryptUnprotectData, CRYPTPROTECT_UI_FORBIDDEN, CRYPT_INTEGER_BLOB,
    };

    let mut input = CRYPT_INTEGER_BLOB {
        cbData: protected.len() as u32,
        pbData: protected.as_ptr() as *mut u8,
    };
    let mut output = CRYPT_INTEGER_BLOB {
        cbData: 0,
        pbData: null_mut(),
    };
    let ok = unsafe {
        CryptUnprotectData(
            &mut input,
            null_mut(),
            null_mut(),
            null_mut(),
            null_mut(),
            CRYPTPROTECT_UI_FORBIDDEN,
            &mut output,
        )
    };
    if ok == 0 {
        return Err(KeylessPassError::Crypto(
            "Windows DPAPI unprotect failed".to_string(),
        ));
    }
    let plaintext =
        unsafe { std::slice::from_raw_parts(output.pbData, output.cbData as usize).to_vec() };
    unsafe {
        LocalFree(output.pbData as _);
    }
    Ok(plaintext)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::platform::PlatformFactorProvider;

    #[test]
    fn windows_provider_trait_smoke() {
        let dir = tempfile::tempdir().unwrap();
        let provider = WindowsPlatformFactorProvider::new(dir.path().to_path_buf());
        let protected = provider.protect_local_package(b"hello").unwrap();
        let plaintext = provider.unprotect_local_package(&protected).unwrap();
        assert_eq!(plaintext, b"hello");
    }
}
