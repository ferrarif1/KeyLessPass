use crate::storage::{read_usb_factor_package, usb_package_file};
use serde::Serialize;
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

pub fn list_usb_candidates() -> std::result::Result<Vec<UsbFactorCandidate>, String> {
    let mut roots = candidate_roots();
    roots.sort();
    roots.dedup();
    let mut candidates = Vec::new();
    for root in roots {
        let package_path = usb_package_file(&root);
        if !package_path.exists() {
            if is_writable_volume(&root) {
                candidates.push(UsbFactorCandidate {
                    root_path: root,
                    package_path,
                    readable: false,
                    user_id: None,
                    device_id: None,
                    message: "未发现 KeylessPass USB 因子包，可用于初始化或恢复。".to_string(),
                });
            }
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
    roots.extend(children_of(Path::new("/Volumes")));

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
