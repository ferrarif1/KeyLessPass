use crate::crypto::b64_encode;
use crate::error::{KeylessPassError, Result};
use crate::platform::{current_platform_provider, PlatformFactorProvider};
use crate::service::factor_keys::{
    cached_master_key_with_local_factor, load_usb_context, master_key_from_mnemonic_usb,
};
use crate::storage::{
    read_usb_factor_package, usb_cdr_backup_file, usb_package_file, verify_usb_cdr_backup,
    write_usb_cdr_backup, CdrStore, StoragePaths,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::io::Write;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UsbFactorCandidate {
    pub root_path: PathBuf,
    pub package_path: PathBuf,
    pub readable: bool,
    pub user_id: Option<String>,
    pub device_id: Option<String>,
    pub message: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VerifyUsbPackageRequest {
    pub mnemonic: String,
    pub usb_path: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VerifyUsbPackageResponse {
    pub valid: bool,
    pub package_path: PathBuf,
    pub user_id: String,
    pub device_id: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UsbCdrRequest {
    pub usb_path: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UsbCdrStatusResponse {
    pub status: String,
    pub backup_path: PathBuf,
    pub local_record_count: usize,
    pub usb_record_count: usize,
    pub message: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UsbCdrSyncResponse {
    pub backup_path: PathBuf,
    pub record_count: usize,
}

pub fn verify_usb_package(
    request: VerifyUsbPackageRequest,
) -> std::result::Result<VerifyUsbPackageResponse, String> {
    let usb = load_usb_context(&request.usb_path).map_err(String::from)?;
    let _ = master_key_from_mnemonic_usb(&request.mnemonic, &usb).map_err(String::from)?;
    Ok(VerifyUsbPackageResponse {
        valid: true,
        package_path: usb_package_file(&request.usb_path),
        user_id: usb.package.user_id.to_string(),
        device_id: usb.package.device_id,
    })
}

pub fn get_usb_cdr_status(
    request: UsbCdrRequest,
) -> std::result::Result<UsbCdrStatusResponse, String> {
    let paths = StoragePaths::default().map_err(String::from)?;
    let provider = current_platform_provider(&paths.app_dir);
    get_usb_cdr_status_with_provider(&paths, provider.as_ref(), request).map_err(String::from)
}

pub fn sync_cdr_to_usb(request: UsbCdrRequest) -> std::result::Result<UsbCdrSyncResponse, String> {
    let paths = StoragePaths::default().map_err(String::from)?;
    let provider = current_platform_provider(&paths.app_dir);
    sync_cdr_to_usb_with_provider(&paths, provider.as_ref(), request).map_err(String::from)
}

pub fn restore_cdr_from_usb(
    request: UsbCdrRequest,
) -> std::result::Result<UsbCdrSyncResponse, String> {
    let paths = StoragePaths::default().map_err(String::from)?;
    let provider = current_platform_provider(&paths.app_dir);
    restore_cdr_from_usb_with_provider(&paths, provider.as_ref(), request).map_err(String::from)
}

pub fn get_usb_cdr_status_with_provider(
    paths: &StoragePaths,
    provider: &dyn PlatformFactorProvider,
    request: UsbCdrRequest,
) -> Result<UsbCdrStatusResponse> {
    let (user_id, master_key, local_records) = local_cdr_context(paths, provider)?;
    let backup_path = usb_cdr_backup_file(&request.usb_path);

    match read_usb_factor_package(&request.usb_path) {
        Ok(package) if package.user_id == user_id => {}
        Ok(_) => {
            return Ok(UsbCdrStatusResponse {
                status: "invalid".to_string(),
                backup_path,
                local_record_count: local_records.len(),
                usb_record_count: 0,
                message: "USB factor package belongs to a different KeyLessPass profile."
                    .to_string(),
            });
        }
        Err(_) => {
            return Ok(UsbCdrStatusResponse {
                status: "invalid".to_string(),
                backup_path,
                local_record_count: local_records.len(),
                usb_record_count: 0,
                message: "USB factor package is missing or unreadable.".to_string(),
            });
        }
    }

    match verify_usb_cdr_backup(&request.usb_path, user_id, &master_key) {
        Ok(backup) => {
            let local_digest = records_digest(&local_records)?;
            let usb_digest = records_digest(&backup.records)?;
            let status = if local_digest == usb_digest {
                "consistent"
            } else {
                match (
                    max_updated_at(&local_records),
                    max_updated_at(&backup.records),
                ) {
                    (Some(local), Some(usb)) if local > usb => "local_newer",
                    (Some(local), Some(usb)) if usb > local => "usb_newer",
                    (Some(_), None) => "local_newer",
                    (None, Some(_)) => "usb_newer",
                    _ => "conflict",
                }
            };
            Ok(UsbCdrStatusResponse {
                status: status.to_string(),
                backup_path,
                local_record_count: local_records.len(),
                usb_record_count: backup.records.len(),
                message: status_message(status).to_string(),
            })
        }
        Err(KeylessPassError::MissingFactor(_)) => Ok(UsbCdrStatusResponse {
            status: "missing".to_string(),
            backup_path,
            local_record_count: local_records.len(),
            usb_record_count: 0,
            message: "USB CDR backup is missing.".to_string(),
        }),
        Err(_) => Ok(UsbCdrStatusResponse {
            status: "invalid".to_string(),
            backup_path,
            local_record_count: local_records.len(),
            usb_record_count: 0,
            message: "USB CDR backup failed integrity verification.".to_string(),
        }),
    }
}

pub fn sync_cdr_to_usb_with_provider(
    paths: &StoragePaths,
    provider: &dyn PlatformFactorProvider,
    request: UsbCdrRequest,
) -> Result<UsbCdrSyncResponse> {
    let (user_id, master_key, local_records) = local_cdr_context(paths, provider)?;
    let package = read_usb_factor_package(&request.usb_path)?;
    if package.user_id != user_id {
        return Err(KeylessPassError::Integrity(
            "USB factor package user mismatch".to_string(),
        ));
    }
    let backup_path =
        write_usb_cdr_backup(&request.usb_path, user_id, &master_key, &local_records)?;
    Ok(UsbCdrSyncResponse {
        backup_path,
        record_count: local_records.len(),
    })
}

pub fn restore_cdr_from_usb_with_provider(
    paths: &StoragePaths,
    provider: &dyn PlatformFactorProvider,
    request: UsbCdrRequest,
) -> Result<UsbCdrSyncResponse> {
    let (user_id, master_key, _) = local_cdr_context(paths, provider)?;
    let backup = verify_usb_cdr_backup(&request.usb_path, user_id, &master_key)?;
    let store = CdrStore::new(&paths.db_path);
    store.replace_all(&backup.records)?;
    Ok(UsbCdrSyncResponse {
        backup_path: usb_cdr_backup_file(&request.usb_path),
        record_count: backup.records.len(),
    })
}

pub fn list_usb_candidates() -> std::result::Result<Vec<UsbFactorCandidate>, String> {
    let mut roots = candidate_roots();
    roots.sort();
    roots.dedup();
    let mut candidates = Vec::new();
    for root in roots {
        let package_path = usb_package_file(&root);
        if !package_path.exists() {
            let writable = is_writable_volume(&root);
            candidates.push(UsbFactorCandidate {
                root_path: root,
                package_path,
                readable: false,
                user_id: None,
                device_id: None,
                message: if writable {
                    "未发现 KeylessPass USB 因子包，可用于初始化或 USB 丢失恢复。".to_string()
                } else {
                    "发现可移动卷。若要初始化或重建 USB 因子包，请通过目录选择按钮授权该 U 盘，或确认卷格式可写。".to_string()
                },
            });
            continue;
        }
        match read_usb_factor_package(&root) {
            Ok(package) => candidates.push(UsbFactorCandidate {
                root_path: root,
                package_path,
                readable: true,
                user_id: Some(package.user_id.to_string()),
                device_id: Some(package.device_id),
                message: "发现 KeylessPass USB 因子包。".to_string(),
            }),
            Err(_) => candidates.push(UsbFactorCandidate {
                root_path: root,
                package_path,
                readable: false,
                user_id: None,
                device_id: None,
                message: "发现疑似 USB 因子包，但读取或校验失败。".to_string(),
            }),
        }
    }
    candidates.sort_by_key(|item| (!item.readable, item.root_path.clone()));
    Ok(candidates)
}

fn candidate_roots() -> Vec<PathBuf> {
    let mut roots = Vec::new();

    #[cfg(target_os = "macos")]
    {
        roots.extend(macos_mounted_volume_roots());
        if roots.is_empty() {
            // Non-sandboxed CLI/tests may still enumerate /Volumes directly.
            roots.extend(children_of(Path::new("/Volumes")));
        }
    }

    #[cfg(all(unix, not(target_os = "macos")))]
    {
        if let Ok(user) = std::env::var("USER") {
            roots.extend(children_of(Path::new("/media").join(&user).as_path()));
            roots.extend(children_of(Path::new("/run/media").join(&user).as_path()));
        }
        roots.extend(children_of(Path::new("/mnt")));
    }

    #[cfg(windows)]
    {
        for letter in b'D'..=b'Z' {
            let path = format!("{}:\\", letter as char);
            let root = PathBuf::from(path);
            if root.exists() {
                roots.push(root);
            }
        }
    }

    roots
}

#[cfg(target_os = "macos")]
fn macos_mounted_volume_roots() -> Vec<PathBuf> {
    use std::ffi::CStr;

    let mut buf: *mut libc::statfs = std::ptr::null_mut();
    let count = unsafe { libc::getmntinfo(&mut buf, libc::MNT_WAIT) };
    if count <= 0 || buf.is_null() {
        return Vec::new();
    }

    let entries = unsafe { std::slice::from_raw_parts(buf, count as usize) };
    let mut roots = Vec::new();
    for entry in entries {
        let mount_on = match unsafe { CStr::from_ptr(entry.f_mntonname.as_ptr()) }.to_str() {
            Ok(value) => value,
            Err(_) => continue,
        };
        if !is_macos_usb_candidate_mount(mount_on) {
            continue;
        }
        let path = PathBuf::from(mount_on);
        if path.is_dir() {
            roots.push(path);
        }
    }
    roots
}

#[cfg(target_os = "macos")]
fn is_macos_usb_candidate_mount(mount_on: &str) -> bool {
    const PREFIX: &str = "/Volumes/";
    if !mount_on.starts_with(PREFIX) || mount_on.len() <= PREFIX.len() {
        return false;
    }
    let path = Path::new(mount_on);
    let Some(name) = path.file_name().and_then(|value| value.to_str()) else {
        return false;
    };
    if name == "Macintosh HD" || name.starts_with('.') {
        return false;
    }
    true
}

#[cfg(unix)]
fn children_of(path: &Path) -> Vec<PathBuf> {
    let Ok(entries) = std::fs::read_dir(path) else {
        return Vec::new();
    };
    entries
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .filter(|path| path.is_dir())
        .collect()
}

fn is_writable_volume(root: &Path) -> bool {
    let Some(name) = root.file_name().and_then(|value| value.to_str()) else {
        return false;
    };
    if name == "Macintosh HD" || name.starts_with('.') {
        return false;
    }

    let probe_path = root.join(format!(".keylesspass_probe_{}", std::process::id()));
    let Ok(mut file) = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&probe_path)
    else {
        return false;
    };
    let wrote = file.write_all(b"keylesspass probe").is_ok();
    drop(file);
    let _ = std::fs::remove_file(&probe_path);
    wrote
}

fn local_cdr_context(
    paths: &StoragePaths,
    provider: &dyn PlatformFactorProvider,
) -> Result<(
    uuid::Uuid,
    Vec<u8>,
    Vec<crate::domain::CredentialDescriptionRecord>,
)> {
    let (config, master_key) = cached_master_key_with_local_factor(paths, provider)?;
    let store = CdrStore::new(&config.cdr_store_path);
    store.init()?;
    let records = store.list_all()?;
    for record in &records {
        record.verify_mac(&master_key)?;
    }
    Ok((config.user_id, master_key.to_vec(), records))
}

fn records_digest(records: &[crate::domain::CredentialDescriptionRecord]) -> Result<String> {
    let mut normalized = records.to_vec();
    normalized.sort_by(|left, right| {
        left.record_seq
            .cmp(&right.record_seq)
            .then_with(|| left.record_id.to_string().cmp(&right.record_id.to_string()))
            .then_with(|| left.version.cmp(&right.version))
    });
    let digest = Sha256::digest(serde_json::to_vec(&normalized)?);
    Ok(b64_encode(&digest))
}

fn max_updated_at(
    records: &[crate::domain::CredentialDescriptionRecord],
) -> Option<chrono::DateTime<chrono::Utc>> {
    records.iter().map(|record| record.updated_at).max()
}

fn status_message(status: &str) -> &'static str {
    match status {
        "consistent" => "Local CDR records and USB backup are consistent.",
        "local_newer" => "Local CDR records are newer than the USB backup.",
        "usb_newer" => "USB CDR backup is newer than local records.",
        "conflict" => "Local and USB CDR records differ.",
        _ => "USB CDR backup needs attention.",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(target_os = "macos")]
    #[test]
    fn macos_mount_filter_skips_system_and_hidden() {
        assert!(!is_macos_usb_candidate_mount("/"));
        assert!(!is_macos_usb_candidate_mount("/Volumes"));
        assert!(!is_macos_usb_candidate_mount("/Volumes/"));
        assert!(!is_macos_usb_candidate_mount("/Volumes/Macintosh HD"));
        assert!(!is_macos_usb_candidate_mount("/Volumes/.hidden"));
        assert!(is_macos_usb_candidate_mount("/Volumes/WD"));
        assert!(is_macos_usb_candidate_mount("/Volumes/My USB"));
    }

    #[test]
    fn writable_volume_skips_macintosh_hd_name() {
        assert!(!is_writable_volume(Path::new("/Volumes/Macintosh HD")));
    }
}
