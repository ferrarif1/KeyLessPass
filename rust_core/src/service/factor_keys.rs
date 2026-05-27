use crate::crypto::{aead, b64_decode, b64_encode, kdf, mac};
use crate::domain::{
    AppConfig, FactorPackage, LocalFactorPayload, UsbFactorPayload, WrappedMasterKey,
    WRAPPED_MASTER_KEY_SCHEMA_VERSION, WRAP_LABEL_CU, WRAP_LABEL_MC, WRAP_LABEL_MU,
};
use crate::error::{KeylessPassError, Result};
use crate::platform::PlatformFactorProvider;
use crate::storage::{
    load_local_factor_payload, load_usb_factor_payload, read_config, StoragePaths,
};
use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};
use uuid::Uuid;
use zeroize::Zeroize;

static MASTER_KEY_CACHE: OnceLock<Mutex<HashMap<String, [u8; 32]>>> = OnceLock::new();

pub(crate) struct LocalFactorContext {
    pub package: FactorPackage,
    pub payload: LocalFactorPayload,
    pub f_c: [u8; 32],
}

pub(crate) struct UsbFactorContext {
    pub package: FactorPackage,
    pub payload: UsbFactorPayload,
    pub f_u: [u8; 32],
}

pub(crate) fn load_local_context(
    provider: &dyn PlatformFactorProvider,
    local_factor_path: &std::path::Path,
) -> Result<LocalFactorContext> {
    let (package, payload) = load_local_factor_payload(provider, local_factor_path)?;
    let f_c = derive_local_factor(provider, &package, &payload)?;
    Ok(LocalFactorContext {
        package,
        payload,
        f_c,
    })
}

pub(crate) fn load_usb_context(path: impl AsRef<std::path::Path>) -> Result<UsbFactorContext> {
    let (package, payload) = load_usb_factor_payload(path)?;
    let f_u = derive_usb_factor(&package, &payload)?;
    Ok(UsbFactorContext {
        package,
        payload,
        f_u,
    })
}

pub(crate) fn derive_local_factor(
    provider: &dyn PlatformFactorProvider,
    package: &FactorPackage,
    payload: &LocalFactorPayload,
) -> Result<[u8; 32]> {
    if payload.device_id != package.device_id {
        return Err(KeylessPassError::Integrity(
            "local factor device id mismatch".to_string(),
        ));
    }
    if payload.user_id != package.user_id {
        return Err(KeylessPassError::Integrity(
            "local factor user id mismatch".to_string(),
        ));
    }
    let device_secret = provider.get_or_create_device_secret()?;
    let salt_c = b64_decode(&payload.salt_c)?;
    kdf::derive_platform_factor(
        device_secret.expose(),
        &package.device_id,
        &package.user_id,
        &salt_c,
    )
}

pub(crate) fn derive_usb_factor(
    package: &FactorPackage,
    payload: &UsbFactorPayload,
) -> Result<[u8; 32]> {
    if payload.user_id != package.user_id {
        return Err(KeylessPassError::Integrity(
            "USB factor user id mismatch".to_string(),
        ));
    }
    let usb_secret = b64_decode(&payload.usb_secret)?;
    let salt_u = b64_decode(&payload.salt_u)?;
    kdf::derive_usb_factor(&usb_secret, &payload.usb_id, &package.user_id, &salt_u)
}

pub(crate) fn derive_mnemonic_factor_checked(
    mnemonic: &str,
    mnemonic_salt_b64: &str,
    verifier_b64: Option<&str>,
) -> Result<[u8; 32]> {
    let mnemonic_salt = b64_decode(mnemonic_salt_b64)?;
    let f_m = kdf::derive_mnemonic_factor(mnemonic, &mnemonic_salt)?;
    if let Some(verifier_b64) = verifier_b64 {
        let actual = kdf::derive_mnemonic_verifier(&f_m)?;
        if !mac::constant_time_eq_b64(&actual, verifier_b64)? {
            return Err(KeylessPassError::MissingFactor(
                "mnemonic phrase did not pass recovery verification".to_string(),
            ));
        }
    }
    Ok(f_m)
}

pub(crate) fn wrap_master_key(
    master_key: &[u8],
    factor_a: &[u8],
    factor_b: &[u8],
    label: &str,
    aad: &str,
) -> Result<WrappedMasterKey> {
    let wrap_key = kdf::derive_pairwise_wrap_key(factor_a, factor_b, label)?;
    let (nonce, ciphertext, tag) = aead::encrypt(&wrap_key, master_key, aad.as_bytes())?;
    Ok(WrappedMasterKey::new(
        wrapper_type_for_label(label)?,
        b64_encode(&nonce),
        b64_encode(&ciphertext),
        b64_encode(&tag),
        aad,
    ))
}

pub(crate) fn unwrap_master_key(
    wrapper: &WrappedMasterKey,
    factor_a: &[u8],
    factor_b: &[u8],
    label: &str,
    aad: &str,
) -> Result<[u8; 32]> {
    if wrapper.version != WRAPPED_MASTER_KEY_SCHEMA_VERSION {
        return Err(KeylessPassError::Validation(
            "unsupported wrapped master key schema version".to_string(),
        ));
    }
    if wrapper.wrapper_type != wrapper_type_for_label(label)? {
        return Err(KeylessPassError::Integrity(
            "wrapped master key type mismatch".to_string(),
        ));
    }
    if wrapper.aad != aad {
        return Err(KeylessPassError::Integrity(
            "wrapped master key AAD mismatch".to_string(),
        ));
    }
    let wrap_key = kdf::derive_pairwise_wrap_key(factor_a, factor_b, label)?;
    let mut plaintext = aead::decrypt(
        &wrap_key,
        &b64_decode(&wrapper.nonce)?,
        &b64_decode(&wrapper.ciphertext)?,
        &b64_decode(&wrapper.tag)?,
        aad.as_bytes(),
    )?;
    if plaintext.len() != 32 {
        plaintext.zeroize();
        return Err(KeylessPassError::Integrity(
            "wrapped master key length mismatch".to_string(),
        ));
    }
    let mut master_key = [0_u8; 32];
    master_key.copy_from_slice(&plaintext);
    plaintext.zeroize();
    Ok(master_key)
}

pub(crate) fn master_key_from_mnemonic_local(
    mnemonic: &str,
    local: &LocalFactorContext,
) -> Result<[u8; 32]> {
    let f_m = derive_mnemonic_factor_checked(
        mnemonic,
        &local.payload.mnemonic_salt,
        local.payload.mnemonic_verifier.as_deref(),
    )?;
    unwrap_master_key(
        &local.payload.w_mc,
        &f_m,
        &local.f_c,
        WRAP_LABEL_MC,
        &mc_wrap_aad(
            local.package.user_id,
            &local.package.device_id,
            &local.payload.mnemonic_salt,
            &local.payload.salt_c,
        ),
    )
}

pub(crate) fn master_key_from_mnemonic_usb(
    mnemonic: &str,
    usb: &UsbFactorContext,
) -> Result<[u8; 32]> {
    let f_m = derive_mnemonic_factor_checked(
        mnemonic,
        &usb.payload.mnemonic_salt,
        usb.payload.mnemonic_verifier.as_deref(),
    )?;
    unwrap_master_key(
        &usb.payload.w_mu,
        &f_m,
        &usb.f_u,
        WRAP_LABEL_MU,
        &mu_wrap_aad(
            usb.package.user_id,
            &usb.payload.usb_id,
            &usb.payload.mnemonic_salt,
            &usb.payload.salt_u,
        ),
    )
}

pub(crate) fn master_key_from_local_usb(
    local: &LocalFactorContext,
    usb: &UsbFactorContext,
) -> Result<[u8; 32]> {
    if local.package.user_id != usb.package.user_id {
        return Err(KeylessPassError::Integrity(
            "local and USB factor user mismatch".to_string(),
        ));
    }
    if local.package.device_id != usb.package.device_id {
        return Err(KeylessPassError::Integrity(
            "USB factor package does not match this managed computer".to_string(),
        ));
    }
    unwrap_master_key(
        &usb.payload.w_cu,
        &local.f_c,
        &usb.f_u,
        WRAP_LABEL_CU,
        &cu_wrap_aad(
            local.package.user_id,
            &local.package.device_id,
            &usb.payload.usb_id,
            &local.payload.salt_c,
            &usb.payload.salt_u,
        ),
    )
}

pub(crate) fn master_key_from_all_factors(
    mnemonic: &str,
    local: &LocalFactorContext,
    usb: &UsbFactorContext,
) -> Result<[u8; 32]> {
    let mc = master_key_from_mnemonic_local(mnemonic, local)?;
    let mu = master_key_from_mnemonic_usb(mnemonic, usb)?;
    let cu = master_key_from_local_usb(local, usb)?;
    if mc != mu || mc != cu {
        return Err(KeylessPassError::Integrity(
            "pairwise master key wrappers disagree".to_string(),
        ));
    }
    Ok(mc)
}

pub(crate) fn mc_wrap_aad(
    user_id: Uuid,
    device_id: &str,
    mnemonic_salt: &str,
    salt_c: &str,
) -> String {
    format!(
        "{WRAP_LABEL_MC}|userId={user_id}|deviceId={device_id}|mnemonicSalt={mnemonic_salt}|saltC={salt_c}"
    )
}

pub(crate) fn mu_wrap_aad(
    user_id: Uuid,
    usb_id: &str,
    mnemonic_salt: &str,
    salt_u: &str,
) -> String {
    format!(
        "{WRAP_LABEL_MU}|userId={user_id}|usbId={usb_id}|mnemonicSalt={mnemonic_salt}|saltU={salt_u}"
    )
}

pub(crate) fn cu_wrap_aad(
    user_id: Uuid,
    device_id: &str,
    usb_id: &str,
    salt_c: &str,
    salt_u: &str,
) -> String {
    format!(
        "{WRAP_LABEL_CU}|userId={user_id}|deviceId={device_id}|usbId={usb_id}|saltC={salt_c}|saltU={salt_u}"
    )
}

fn wrapper_type_for_label(label: &str) -> Result<&'static str> {
    match label {
        WRAP_LABEL_MC => Ok("MC"),
        WRAP_LABEL_MU => Ok("MU"),
        WRAP_LABEL_CU => Ok("CU"),
        _ => Err(KeylessPassError::Validation(
            "unsupported wrapped master key label".to_string(),
        )),
    }
}

pub(crate) fn remember_master_key(config: &AppConfig, master_key: &[u8; 32]) -> Result<()> {
    let cache = MASTER_KEY_CACHE.get_or_init(|| Mutex::new(HashMap::new()));
    let mut guard = cache
        .lock()
        .map_err(|_| KeylessPassError::Crypto("master key cache lock poisoned".to_string()))?;
    guard.insert(master_cache_key(config), *master_key);
    Ok(())
}

pub(crate) fn cached_master_key(config: &AppConfig) -> Result<[u8; 32]> {
    let cache = MASTER_KEY_CACHE.get_or_init(|| Mutex::new(HashMap::new()));
    let guard = cache
        .lock()
        .map_err(|_| KeylessPassError::Crypto("master key cache lock poisoned".to_string()))?;
    guard
        .get(&master_cache_key(config))
        .copied()
        .ok_or_else(|| {
            KeylessPassError::MissingFactor(
                "master key is locked; unlock it with two recovery factors first".to_string(),
            )
        })
}

pub(crate) fn cached_master_key_with_local_factor(
    paths: &StoragePaths,
    provider: &dyn PlatformFactorProvider,
) -> Result<(AppConfig, [u8; 32])> {
    let config = read_config(paths)?;
    let _ = load_local_context(provider, &config.local_factor_path)?;
    let master_key = cached_master_key(&config)?;
    Ok((config, master_key))
}

fn master_cache_key(config: &AppConfig) -> String {
    format!(
        "{}:{}",
        config.user_id,
        config.local_factor_path.to_string_lossy()
    )
}
