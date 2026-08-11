use crate::crypto::recovery::{decode_recovery_phrase, recover_root_key};
use crate::domain::{RecoveryManifest, RecoveryShareSet, ShareEnvelope};
use crate::error::{KeylessPassError, Result};
use crate::platform::PlatformFactorProvider;
use crate::storage::{read_json, write_json_private, StoragePaths};
use std::path::{Path, PathBuf};

pub const RECOVERY_V3_MANIFEST_FILE: &str = "recovery-manifest-v3.json";
pub const USB_RECOVERY_V3_DIR: &str = "keylesspass-recovery-v3";
pub const USB_RECOVERY_V3_MANIFEST_FILE: &str = "recovery-manifest-v3.json";

pub fn recovery_manifest_v3_file(paths: &StoragePaths) -> PathBuf {
    paths.app_dir.join(RECOVERY_V3_MANIFEST_FILE)
}

pub fn managed_share_v3_file(paths: &StoragePaths, share_set_id: uuid::Uuid) -> PathBuf {
    paths
        .app_dir
        .join("recovery-v3")
        .join(format!("managed-{share_set_id}.protected"))
}

pub fn usb_share_v3_file(usb_path: impl AsRef<Path>, share_set_id: uuid::Uuid) -> PathBuf {
    usb_path
        .as_ref()
        .join(USB_RECOVERY_V3_DIR)
        .join(format!("usb-{share_set_id}.json"))
}

pub fn usb_recovery_manifest_v3_file(usb_path: impl AsRef<Path>) -> PathBuf {
    usb_path
        .as_ref()
        .join(USB_RECOVERY_V3_DIR)
        .join(USB_RECOVERY_V3_MANIFEST_FILE)
}

pub fn commit_recovery_share_set(
    paths: &StoragePaths,
    provider: &dyn PlatformFactorProvider,
    usb_path: impl AsRef<Path>,
    set: &RecoveryShareSet,
) -> Result<()> {
    let local_file = managed_share_v3_file(paths, set.manifest.share_set_id);
    let protected = provider
        .protect_local_package(&serde_json_canonicalizer::to_vec(&set.managed_computer)?)?;
    crate::platform::fallback::write_private_file(&local_file, &protected)?;

    let usb_file = usb_share_v3_file(&usb_path, set.manifest.share_set_id);
    write_json_private(&usb_file, &set.usb)?;

    let loaded_local = read_managed_share(paths, provider, &set.manifest)?;
    let loaded_usb: ShareEnvelope = read_json(&usb_file)?;
    let recovery = decode_recovery_phrase(&set.recovery_phrase)?;
    recover_root_key(&recovery, &loaded_local, &set.manifest)?;
    recover_root_key(&recovery, &loaded_usb, &set.manifest)?;
    recover_root_key(&loaded_local, &loaded_usb, &set.manifest)?;

    write_json_private(&usb_recovery_manifest_v3_file(&usb_path), &set.manifest)?;
    // The manifest is the commit marker. Generation-specific factor files make
    // a crash before this write leave the previous committed set selectable.
    write_json_private(&recovery_manifest_v3_file(paths), &set.manifest)
}

pub fn read_usb_recovery_manifest_v3(usb_path: impl AsRef<Path>) -> Result<RecoveryManifest> {
    read_json(&usb_recovery_manifest_v3_file(usb_path))
}

pub fn read_recovery_manifest_v3(paths: &StoragePaths) -> Result<RecoveryManifest> {
    read_json(&recovery_manifest_v3_file(paths))
}

pub fn read_managed_share(
    paths: &StoragePaths,
    provider: &dyn PlatformFactorProvider,
    manifest: &RecoveryManifest,
) -> Result<ShareEnvelope> {
    let protected = std::fs::read(managed_share_v3_file(paths, manifest.share_set_id))?;
    let plaintext = provider.unprotect_local_package(&protected)?;
    let share: ShareEnvelope = serde_json::from_slice(&plaintext)?;
    validate_committed_share(&share, manifest)?;
    Ok(share)
}

pub fn read_usb_share(
    usb_path: impl AsRef<Path>,
    manifest: &RecoveryManifest,
) -> Result<ShareEnvelope> {
    let share: ShareEnvelope = read_json(&usb_share_v3_file(usb_path, manifest.share_set_id))?;
    validate_committed_share(&share, manifest)?;
    Ok(share)
}

fn validate_committed_share(share: &ShareEnvelope, manifest: &RecoveryManifest) -> Result<()> {
    if share.vault_id != manifest.vault_id
        || share.root_generation != manifest.root_generation
        || share.share_set_id != manifest.share_set_id
    {
        return Err(KeylessPassError::Integrity(
            "factor share does not match the committed recovery manifest".to_string(),
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crypto::recovery::create_share_set;
    use crate::platform::fallback::FallbackPlatformFactorProvider;
    use uuid::Uuid;

    #[test]
    fn manifest_last_commit_round_trips_all_recovery_paths() {
        let app = tempfile::tempdir().unwrap();
        let usb = tempfile::tempdir().unwrap();
        let paths = StoragePaths::from_app_dir(app.path().to_path_buf());
        let provider = FallbackPlatformFactorProvider::new(paths.app_dir.clone(), "test");
        let root = [0x44_u8; 32];
        let set =
            create_share_set(&root, Uuid::new_v4(), 1, 1, 1, "computer", 1, "usb", 1).unwrap();
        commit_recovery_share_set(&paths, &provider, usb.path(), &set).unwrap();
        let manifest = read_recovery_manifest_v3(&paths).unwrap();
        let local = read_managed_share(&paths, &provider, &manifest).unwrap();
        let usb_share = read_usb_share(usb.path(), &manifest).unwrap();
        assert_eq!(
            recover_root_key(&local, &usb_share, &manifest).unwrap(),
            root
        );
    }
}
