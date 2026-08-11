use crate::crypto::recovery::{create_share_set, decode_recovery_phrase, recover_root_key};
use crate::domain::RecoveryManifest;
use crate::error::{KeylessPassError, Result};
use crate::platform::PlatformFactorProvider;
use crate::service::factor_keys::remember_master_key;
use crate::storage::{
    commit_recovery_share_set, read_config, read_managed_share, read_recovery_manifest_v3,
    read_usb_recovery_manifest_v3, read_usb_share, CdrStore, StoragePaths,
};
use rand::RngCore;
use serde::Serialize;
use std::path::Path;
use uuid::Uuid;
use zeroize::Zeroize;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RecoveryLifecycleResult {
    pub recovery_phrase: String,
    pub manifest: RecoveryManifest,
}

pub fn refresh_share_set(
    paths: &StoragePaths,
    provider: &dyn PlatformFactorProvider,
    usb_path: impl AsRef<Path>,
) -> Result<RecoveryLifecycleResult> {
    let manifest = read_recovery_manifest_v3(paths)?;
    let managed = read_managed_share(paths, provider, &manifest)?;
    let usb = read_usb_share(&usb_path, &manifest)?;
    let mut root = recover_root_key(&managed, &usb, &manifest)?;
    let next_factor_generation = managed.factor_generation.max(usb.factor_generation) + 1;
    let set = create_share_set(
        &root,
        manifest.vault_id,
        manifest.root_generation,
        manifest.share_set_generation + 1,
        next_factor_generation,
        &managed.factor_id,
        next_factor_generation,
        &usb.factor_id,
        next_factor_generation,
    )?;
    commit_recovery_share_set(paths, provider, usb_path, &set)?;
    root.zeroize();
    Ok(RecoveryLifecycleResult {
        recovery_phrase: set.recovery_phrase,
        manifest: set.manifest,
    })
}

pub fn replace_usb_factor(
    paths: &StoragePaths,
    provider: &dyn PlatformFactorProvider,
    recovery_phrase: &str,
    new_usb_path: impl AsRef<Path>,
) -> Result<RecoveryLifecycleResult> {
    let manifest = read_recovery_manifest_v3(paths)?;
    let managed = read_managed_share(paths, provider, &manifest)?;
    let recovery = decode_recovery_phrase(recovery_phrase)?;
    let mut root = recover_root_key(&recovery, &managed, &manifest)?;
    let next_factor_generation = managed.factor_generation.max(recovery.factor_generation) + 1;
    let set = create_share_set(
        &root,
        manifest.vault_id,
        manifest.root_generation,
        manifest.share_set_generation + 1,
        next_factor_generation,
        &managed.factor_id,
        next_factor_generation,
        &Uuid::new_v4().to_string(),
        next_factor_generation,
    )?;
    commit_recovery_share_set(paths, provider, new_usb_path, &set)?;
    root.zeroize();
    Ok(RecoveryLifecycleResult {
        recovery_phrase: set.recovery_phrase,
        manifest: set.manifest,
    })
}

pub fn replace_managed_computer(
    new_paths: &StoragePaths,
    new_provider: &dyn PlatformFactorProvider,
    recovery_phrase: &str,
    usb_path: impl AsRef<Path>,
) -> Result<RecoveryLifecycleResult> {
    let manifest = read_usb_recovery_manifest_v3(&usb_path)?;
    let usb = read_usb_share(&usb_path, &manifest)?;
    let recovery = decode_recovery_phrase(recovery_phrase)?;
    let mut root = recover_root_key(&recovery, &usb, &manifest)?;
    let next_factor_generation = recovery.factor_generation.max(usb.factor_generation) + 1;
    let device_id = new_provider.get_or_create_device_id()?;
    let set = create_share_set(
        &root,
        manifest.vault_id,
        manifest.root_generation,
        manifest.share_set_generation + 1,
        next_factor_generation,
        &device_id,
        next_factor_generation,
        &usb.factor_id,
        next_factor_generation,
    )?;
    commit_recovery_share_set(new_paths, new_provider, usb_path, &set)?;
    root.zeroize();
    Ok(RecoveryLifecycleResult {
        recovery_phrase: set.recovery_phrase,
        manifest: set.manifest,
    })
}

pub fn rotate_root_for_empty_vault(
    paths: &StoragePaths,
    provider: &dyn PlatformFactorProvider,
    usb_path: impl AsRef<Path>,
) -> Result<RecoveryLifecycleResult> {
    let config = read_config(paths)?;
    let store = CdrStore::new(&config.cdr_store_path);
    store.init()?;
    if !store.list_all()?.is_empty() {
        return Err(KeylessPassError::Validation(
            "Root Key rotation for a non-empty vault requires remote rotation of every service password and is not automatically safe"
                .to_string(),
        ));
    }
    let manifest = read_recovery_manifest_v3(paths)?;
    let managed = read_managed_share(paths, provider, &manifest)?;
    let usb = read_usb_share(&usb_path, &manifest)?;
    let next_factor_generation = managed.factor_generation.max(usb.factor_generation) + 1;
    let mut root = [0_u8; 32];
    rand::rngs::OsRng.fill_bytes(&mut root);
    let set = create_share_set(
        &root,
        manifest.vault_id,
        manifest.root_generation + 1,
        manifest.share_set_generation + 1,
        next_factor_generation,
        &managed.factor_id,
        next_factor_generation,
        &usb.factor_id,
        next_factor_generation,
    )?;
    commit_recovery_share_set(paths, provider, usb_path, &set)?;
    remember_master_key(&config, &root)?;
    root.zeroize();
    Ok(RecoveryLifecycleResult {
        recovery_phrase: set.recovery_phrase,
        manifest: set.manifest,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crypto::recovery::create_share_set;
    use crate::domain::{AppConfig, PasswordDerivationAlgorithm};
    use crate::platform::fallback::FallbackPlatformFactorProvider;
    use crate::storage::write_config;

    fn setup() -> (
        tempfile::TempDir,
        tempfile::TempDir,
        StoragePaths,
        FallbackPlatformFactorProvider,
        RecoveryLifecycleResult,
    ) {
        let app = tempfile::tempdir().unwrap();
        let usb = tempfile::tempdir().unwrap();
        let paths = StoragePaths::from_app_dir(app.path().to_path_buf());
        let provider = FallbackPlatformFactorProvider::new(paths.app_dir.clone(), "lifecycle");
        let vault = Uuid::new_v4();
        let root = [0x31_u8; 32];
        let set = create_share_set(&root, vault, 1, 1, 1, "computer", 1, "usb", 1).unwrap();
        commit_recovery_share_set(&paths, &provider, usb.path(), &set).unwrap();
        let config = AppConfig::new(
            "test",
            vault,
            "test",
            "computer",
            paths.db_path.clone(),
            paths.local_factor_path.clone(),
            PasswordDerivationAlgorithm::HkdfSha256,
        );
        write_config(&paths, &config).unwrap();
        (
            app,
            usb,
            paths,
            provider,
            RecoveryLifecycleResult {
                recovery_phrase: set.recovery_phrase,
                manifest: set.manifest,
            },
        )
    }

    #[test]
    fn refresh_invalidates_cross_generation_share_mixing() {
        let (_app, usb, paths, provider, old) = setup();
        let old_local = read_managed_share(&paths, &provider, &old.manifest).unwrap();
        let refreshed = refresh_share_set(&paths, &provider, usb.path()).unwrap();
        let new_usb = read_usb_share(usb.path(), &refreshed.manifest).unwrap();
        assert!(recover_root_key(&old_local, &new_usb, &refreshed.manifest).is_err());
    }

    #[test]
    fn empty_vault_root_rotation_rejects_old_threshold_shares() {
        let (_app, usb, paths, provider, old) = setup();
        let old_local = read_managed_share(&paths, &provider, &old.manifest).unwrap();
        let old_usb = read_usb_share(usb.path(), &old.manifest).unwrap();
        let rotated = rotate_root_for_empty_vault(&paths, &provider, usb.path()).unwrap();
        assert!(recover_root_key(&old_local, &old_usb, &rotated.manifest).is_err());
        assert_eq!(rotated.manifest.root_generation, 2);
    }
}
