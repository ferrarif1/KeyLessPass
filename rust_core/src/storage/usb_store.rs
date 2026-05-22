use crate::crypto::{aead, b64_decode, b64_encode, kdf, mac};
use crate::domain::{FactorPackage, PackageType, UsbFactorPayload};
use crate::error::{KeylessPassError, Result};
use crate::storage::{read_json, write_json_private};
use std::path::{Path, PathBuf};
use uuid::Uuid;

pub const USB_FACTOR_FILE: &str = "keylesspass-usb-factor.json";

pub fn usb_package_file(path: impl AsRef<Path>) -> PathBuf {
    let path = path.as_ref();
    if path.is_dir() || path.extension().is_none() {
        path.join(USB_FACTOR_FILE)
    } else {
        path.to_path_buf()
    }
}

pub fn create_usb_factor_package(
    mnemonic: &str,
    user_id: Uuid,
    device_id: &str,
    platform: &str,
    payload: &UsbFactorPayload,
) -> Result<FactorPackage> {
    let salt = b64_decode(&payload.mnemonic_salt)?;
    let f_m = kdf::derive_mnemonic_factor(mnemonic, &user_id, &salt)?;
    let key = kdf::derive_usb_package_key(&f_m)?;
    let plaintext = serde_json::to_vec(payload)?;
    let aad = usb_aad(user_id);
    let (nonce, ciphertext, tag) = aead::encrypt(&key, &plaintext, aad.as_bytes())?;
    let mut package = FactorPackage::new(
        PackageType::Usb,
        user_id,
        device_id,
        platform,
        payload.mnemonic_salt.clone(),
        b64_encode(&ciphertext),
        b64_encode(&nonce),
        b64_encode(&tag),
    );
    package.package_mac =
        mac::hmac_sha256_base64(&mac::package_mac_key(&f_m), &package.mac_payload()?)?;
    Ok(package)
}

pub fn write_usb_factor_package(
    path: impl AsRef<Path>,
    package: &FactorPackage,
) -> Result<PathBuf> {
    let file = usb_package_file(path);
    write_json_private(&file, package)?;
    Ok(file)
}

pub fn read_usb_factor_package(path: impl AsRef<Path>) -> Result<FactorPackage> {
    let file = usb_package_file(path);
    if !file.exists() {
        return Err(KeylessPassError::MissingFactor(
            "USB factor package not found".to_string(),
        ));
    }
    read_json(&file)
}

pub fn load_usb_factor_payload(
    mnemonic: &str,
    path: impl AsRef<Path>,
) -> Result<(FactorPackage, UsbFactorPayload)> {
    let package = read_usb_factor_package(path)?;
    if package.package_type != PackageType::Usb {
        return Err(KeylessPassError::Validation(
            "factor package is not USB".to_string(),
        ));
    }
    let salt = b64_decode(&package.kdf_salt)?;
    let f_m = kdf::derive_mnemonic_factor(mnemonic, &package.user_id, &salt)?;
    let expected = mac::hmac_sha256_base64(&mac::package_mac_key(&f_m), &package.mac_payload()?)?;
    if !mac::constant_time_eq_b64(&expected, &package.package_mac)? {
        return Err(KeylessPassError::Integrity(
            "USB package MAC mismatch".to_string(),
        ));
    }
    let key = kdf::derive_usb_package_key(&f_m)?;
    let plaintext = aead::decrypt(
        &key,
        &b64_decode(&package.nonce)?,
        &b64_decode(&package.encrypted_payload)?,
        &b64_decode(&package.aead_tag)?,
        usb_aad(package.user_id).as_bytes(),
    )?;
    let payload: UsbFactorPayload = serde_json::from_slice(&plaintext)?;
    Ok((package, payload))
}

fn usb_aad(user_id: Uuid) -> String {
    format!("KeylessPass USB factor package:{user_id}")
}
