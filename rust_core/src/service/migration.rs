use crate::crypto::recovery::{create_share_set, recover_root_key, recover_root_key_with_phrase};
use crate::domain::RecoveryManifest;
use crate::error::{KeylessPassError, Result};
use crate::platform::PlatformFactorProvider;
use crate::service::factor_keys::{
    load_local_context, load_usb_context, master_key_from_local_usb,
    master_key_from_mnemonic_local, master_key_from_mnemonic_usb,
};
use crate::storage::{
    commit_recovery_share_set, read_managed_share, read_recovery_manifest_v3, read_usb_share,
    usb_package_file, write_json_private, StoragePaths,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use zeroize::Zeroize;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PairwiseMigrationRequest {
    pub mnemonic: String,
    pub usb_path: String,
    #[serde(default)]
    pub dry_run: bool,
    #[serde(default)]
    pub archive_legacy_wrappers: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PairwiseMigrationReport {
    pub migration_version: u32,
    pub dry_run: bool,
    pub source_schema: String,
    pub target_schema: String,
    pub vault_id: uuid::Uuid,
    pub root_generation: u64,
    pub share_set_id: Option<uuid::Uuid>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recovery_phrase: Option<String>,
    pub verified_recovery_paths: Vec<String>,
    pub legacy_archive: Option<PathBuf>,
    pub completed_at: DateTime<Utc>,
}

pub fn migrate_pairwise_recovery_default(
    request: PairwiseMigrationRequest,
) -> std::result::Result<PairwiseMigrationReport, String> {
    let paths = StoragePaths::default().map_err(String::from)?;
    let provider = crate::platform::current_platform_provider(&paths.app_dir);
    migrate_pairwise_recovery(&paths, provider.as_ref(), request).map_err(String::from)
}

pub fn migrate_pairwise_recovery(
    paths: &StoragePaths,
    provider: &dyn PlatformFactorProvider,
    request: PairwiseMigrationRequest,
) -> Result<PairwiseMigrationReport> {
    let local = load_local_context(provider, &paths.local_factor_path)?;
    let usb = load_usb_context(&request.usb_path)?;
    let mut from_recovery_local = master_key_from_mnemonic_local(&request.mnemonic, &local)?;
    let mut from_recovery_usb = master_key_from_mnemonic_usb(&request.mnemonic, &usb)?;
    let mut from_local_usb = master_key_from_local_usb(&local, &usb)?;
    if from_recovery_local != from_recovery_usb || from_recovery_local != from_local_usb {
        from_recovery_local.zeroize();
        from_recovery_usb.zeroize();
        from_local_usb.zeroize();
        return Err(KeylessPassError::Integrity(
            "legacy pairwise wrappers do not recover the same Root Key".to_string(),
        ));
    }
    let root_generation = local
        .payload
        .recovery_generation
        .max(usb.payload.recovery_generation);
    let verified_recovery_paths = vec![
        "recovery+managed-computer".to_string(),
        "recovery+usb".to_string(),
        "managed-computer+usb".to_string(),
    ];
    let mut report = PairwiseMigrationReport {
        migration_version: 1,
        dry_run: request.dry_run,
        source_schema: "pairwise-complete-key-wrappers-v2".to_string(),
        target_schema: "authenticated-shamir-2-of-3-v3".to_string(),
        vault_id: local.package.user_id,
        root_generation,
        share_set_id: None,
        recovery_phrase: None,
        verified_recovery_paths,
        legacy_archive: None,
        completed_at: Utc::now(),
    };
    if !request.dry_run {
        let set = create_share_set(
            &from_recovery_local,
            local.package.user_id,
            root_generation,
            root_generation,
            &local.package.device_id,
            root_generation,
            &usb.payload.usb_id,
            root_generation,
        )?;
        commit_recovery_share_set(paths, provider, &request.usb_path, &set)?;
        report.share_set_id = Some(set.manifest.share_set_id);
        report.recovery_phrase = Some(set.recovery_phrase.clone());
        if request.archive_legacy_wrappers {
            report.legacy_archive = Some(archive_legacy_wrappers(
                paths,
                &request.usb_path,
                set.manifest.share_set_id,
            )?);
        }
        let mut audit_report = report.clone();
        audit_report.recovery_phrase = None;
        write_json_private(
            &paths.app_dir.join("recovery-migration-v3-audit.json"),
            &audit_report,
        )?;
    }
    from_recovery_local.zeroize();
    from_recovery_usb.zeroize();
    from_local_usb.zeroize();
    Ok(report)
}

pub fn unlock_v3_with_recovery_phrase(
    paths: &StoragePaths,
    provider: &dyn PlatformFactorProvider,
    recovery_phrase: &str,
) -> Result<[u8; 32]> {
    let manifest = read_recovery_manifest_v3(paths)?;
    let managed = read_managed_share(paths, provider, &manifest)?;
    recover_root_key_with_phrase(recovery_phrase, &managed, &manifest)
}

pub fn unlock_v3_with_local_and_usb(
    paths: &StoragePaths,
    provider: &dyn PlatformFactorProvider,
    usb_path: impl AsRef<Path>,
) -> Result<[u8; 32]> {
    let manifest: RecoveryManifest = read_recovery_manifest_v3(paths)?;
    let managed = read_managed_share(paths, provider, &manifest)?;
    let usb = read_usb_share(usb_path, &manifest)?;
    recover_root_key(&managed, &usb, &manifest)
}

fn archive_legacy_wrappers(
    paths: &StoragePaths,
    usb_path: impl AsRef<Path>,
    share_set_id: uuid::Uuid,
) -> Result<PathBuf> {
    let archive = paths
        .app_dir
        .join("deprecated-pairwise-wrappers")
        .join(share_set_id.to_string());
    std::fs::create_dir_all(&archive)?;
    let local_archive = archive.join("local-factor-package.json");
    let usb_file = usb_package_file(usb_path);
    let usb_archive = archive.join("keylesspass-usb-factor.json");

    // The USB is commonly a different filesystem, so rename(2) is not a safe
    // cross-volume migration primitive. Copy both recoverable legacy packages
    // first; remove the sources only after both copies are durable enough to
    // be opened and compared byte-for-byte.
    std::fs::copy(&paths.local_factor_path, &local_archive)?;
    if let Err(error) = std::fs::copy(&usb_file, &usb_archive) {
        let _ = std::fs::remove_file(&local_archive);
        return Err(error.into());
    }
    if std::fs::read(&paths.local_factor_path)? != std::fs::read(&local_archive)?
        || std::fs::read(&usb_file)? != std::fs::read(&usb_archive)?
    {
        return Err(KeylessPassError::Integrity(
            "legacy recovery archive verification failed".to_string(),
        ));
    }
    std::fs::remove_file(&paths.local_factor_path)?;
    std::fs::remove_file(&usb_file)?;
    Ok(archive)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::PasswordDerivationAlgorithm;
    use crate::platform::fallback::FallbackPlatformFactorProvider;
    use crate::service::enrollment::{enroll_with_provider, EnrollmentRequest};

    #[test]
    fn migrates_legacy_wrappers_without_changing_the_root_key() {
        let app = tempfile::tempdir().unwrap();
        let usb = tempfile::tempdir().unwrap();
        let paths = StoragePaths::from_app_dir(app.path().to_path_buf());
        let provider = FallbackPlatformFactorProvider::new(paths.app_dir.clone(), "migration-test");
        let phrase =
            "alpha bridge cable delta ember forest galaxy harbor ivory jungle kinetic lemon";
        enroll_with_provider(
            &paths,
            &provider,
            EnrollmentRequest {
                mnemonic: phrase.to_string(),
                usb_path: usb.path().to_string_lossy().to_string(),
                password_derivation_algorithm: PasswordDerivationAlgorithm::HkdfSha256,
            },
        )
        .unwrap();
        let legacy_root = master_key_from_mnemonic_local(
            phrase,
            &load_local_context(&provider, &paths.local_factor_path).unwrap(),
        )
        .unwrap();
        let report = migrate_pairwise_recovery(
            &paths,
            &provider,
            PairwiseMigrationRequest {
                mnemonic: phrase.to_string(),
                usb_path: usb.path().to_string_lossy().to_string(),
                dry_run: false,
                archive_legacy_wrappers: false,
            },
        )
        .unwrap();
        let recovered = unlock_v3_with_local_and_usb(&paths, &provider, usb.path()).unwrap();
        assert_eq!(recovered, legacy_root);
        assert!(report.share_set_id.is_some());
        assert!(report.recovery_phrase.is_some());
    }
}
