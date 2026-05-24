use crate::storage::{load_usb_factor_payload, read_usb_factor_package, usb_package_file};
use serde::{Deserialize, Serialize};
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

pub fn verify_usb_package(
    request: VerifyUsbPackageRequest,
) -> std::result::Result<VerifyUsbPackageResponse, String> {
    let (package, _) =
        load_usb_factor_payload(&request.mnemonic, &request.usb_path).map_err(String::from)?;
    Ok(VerifyUsbPackageResponse {
        valid: true,
        package_path: usb_package_file(&request.usb_path),
        user_id: package.user_id.to_string(),
        device_id: package.device_id,
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
