use crate::crypto::{aead, b64_decode, b64_encode, kdf, mac};
use crate::domain::{CredentialDescriptionRecord, FactorPackage, PackageType, UsbFactorPayload};
use crate::error::{KeylessPassError, Result};
use crate::storage::{read_json, write_json_private};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use uuid::Uuid;

pub const USB_FACTOR_FILE: &str = "keylesspass-usb-factor.json";
pub const USB_CDR_BACKUP_FILE: &str = "keylesspass-cdr-backup.json";

pub fn usb_package_file(path: impl AsRef<Path>) -> PathBuf {
    let path = path.as_ref();
    if path.is_dir() || path.extension().is_none() {
        path.join(USB_FACTOR_FILE)
    } else {
        path.to_path_buf()
    }
}

pub fn usb_cdr_backup_file(path: impl AsRef<Path>) -> PathBuf {
    let path = path.as_ref();
    if path.is_dir() || path.extension().is_none() {
        path.join(USB_CDR_BACKUP_FILE)
    } else {
        path.to_path_buf()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UsbCdrBackup {
    pub schema_version: u32,
    pub user_id: Uuid,
    pub records: Vec<CredentialDescriptionRecord>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub backup_mac: String,
}

impl UsbCdrBackup {
    pub fn new(user_id: Uuid, records: Vec<CredentialDescriptionRecord>) -> Self {
        let now = Utc::now();
        Self {
            schema_version: 1,
            user_id,
            records,
            created_at: now,
            updated_at: now,
            backup_mac: String::new(),
        }
    }

    pub fn mac_payload(&self) -> Result<Vec<u8>> {
        let mut copy = self.clone();
        copy.backup_mac.clear();
        Ok(serde_json::to_vec(&copy)?)
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

pub fn write_usb_cdr_backup(
    path: impl AsRef<Path>,
    user_id: Uuid,
    master_key: &[u8],
    records: &[CredentialDescriptionRecord],
) -> Result<PathBuf> {
    let mut backup = UsbCdrBackup::new(user_id, records.to_vec());
    backup.backup_mac =
        mac::hmac_sha256_base64(&mac::cdr_backup_mac_key(master_key), &backup.mac_payload()?)?;
    let file = usb_cdr_backup_file(path);
    write_json_private(&file, &backup)?;
    Ok(file)
}

pub fn read_usb_cdr_backup(path: impl AsRef<Path>) -> Result<UsbCdrBackup> {
    let file = usb_cdr_backup_file(path);
    if !file.exists() {
        return Err(KeylessPassError::MissingFactor(
            "USB CDR backup not found".to_string(),
        ));
    }
    read_json(&file)
}

pub fn verify_usb_cdr_backup(
    path: impl AsRef<Path>,
    expected_user_id: Uuid,
    master_key: &[u8],
) -> Result<UsbCdrBackup> {
    let backup = read_usb_cdr_backup(path)?;
    if backup.schema_version != 1 {
        return Err(KeylessPassError::Validation(
            "unsupported USB CDR backup schema version".to_string(),
        ));
    }
    if backup.user_id != expected_user_id {
        return Err(KeylessPassError::Integrity(
            "USB CDR backup user mismatch".to_string(),
        ));
    }
    let expected =
        mac::hmac_sha256_base64(&mac::cdr_backup_mac_key(master_key), &backup.mac_payload()?)?;
    if !mac::constant_time_eq_b64(&expected, &backup.backup_mac)? {
        return Err(KeylessPassError::Integrity(
            "USB CDR backup MAC mismatch".to_string(),
        ));
    }
    for record in &backup.records {
        record.verify_mac(master_key)?;
    }
    Ok(backup)
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
    let legacy_expected =
        mac::hmac_sha256_base64(&mac::package_mac_key(&f_m), &package.legacy_mac_payload()?)?;
    if !mac::constant_time_eq_b64(&expected, &package.package_mac)?
        && !mac::constant_time_eq_b64(&legacy_expected, &package.package_mac)?
    {
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
