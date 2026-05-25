use crate::crypto::kdf;
use crate::domain::{CredentialState, EncodingDescriptor};
use crate::platform::fallback::FallbackPlatformFactorProvider;
use crate::platform::linux::LinuxPlatformFactorProvider;
use crate::platform::macos::MacOSPlatformFactorProvider;
use crate::platform::windows::WindowsPlatformFactorProvider;
use crate::platform::PlatformFactorProvider;
use crate::service::credentials::{
    add_credential_with_provider, update_credential_display_with_provider, AddCredentialRequest,
    UpdateCredentialDisplayRequest,
};
use crate::service::derive::{derive_password_with_provider, DerivePasswordRequest};
use crate::service::enrollment::{enroll_with_provider, EnrollmentRequest};
use crate::service::recovery::{
    recover_local_with_provider, recover_usb_with_provider, reset_mnemonic_with_provider,
    RecoverLocalRequest, RecoverUsbRequest, ResetMnemonicRequest,
};
use crate::service::rotation::{
    cancel_rotation_with_provider, confirm_rotation_with_provider, rotate_credential_with_provider,
    CancelRotationRequest, ConfirmRotationRequest, RotateCredentialRequest,
};
use crate::service::usb::{
    get_usb_cdr_status_with_provider, restore_cdr_from_usb_with_provider,
    sync_cdr_to_usb_with_provider, verify_usb_package, UsbCdrRequest, VerifyUsbPackageRequest,
};
use crate::storage::{usb_package_file, CdrStore, StoragePaths};
use tempfile::TempDir;
use uuid::Uuid;

const MNEMONIC: &str =
    "alpha bridge cable delta ember forest galaxy harbor ivory jungle kinetic lemon";
const NEW_MNEMONIC: &str =
    "anchor bridge cedar delta ember forest galaxy harbor ivory jasmine kernel lantern";

struct Harness {
    _app_dir: TempDir,
    usb_dir: TempDir,
    paths: StoragePaths,
    provider: FallbackPlatformFactorProvider,
    record_id: Uuid,
    version: u32,
}

fn setup() -> Harness {
    let app_dir = tempfile::tempdir().unwrap();
    let usb_dir = tempfile::tempdir().unwrap();
    let paths = StoragePaths::from_app_dir(app_dir.path().to_path_buf());
    let provider = FallbackPlatformFactorProvider::new(paths.app_dir.clone(), "test-platform");
    enroll_with_provider(
        &paths,
        &provider,
        EnrollmentRequest {
            mnemonic: MNEMONIC.to_string(),
            usb_path: usb_dir.path().to_string_lossy().to_string(),
        },
    )
    .unwrap();
    let record = add_credential_with_provider(
        &paths,
        &provider,
        AddCredentialRequest {
            display_name: "Legacy ERP".to_string(),
            service_hint: "erp.internal".to_string(),
            account_hint: "alice".to_string(),
            notes: String::new(),
            encoding_descriptor: Some(EncodingDescriptor::default()),
        },
    )
    .unwrap();
    Harness {
        _app_dir: app_dir,
        usb_dir,
        paths,
        provider,
        record_id: record.record_id,
        version: record.version,
    }
}

fn derive(harness: &Harness) -> String {
    derive_password_with_provider(
        &harness.paths,
        &harness.provider,
        DerivePasswordRequest {
            record_id: harness.record_id,
            version: Some(harness.version),
            mnemonic: MNEMONIC.to_string(),
            usb_path: harness.usb_dir.path().to_string_lossy().to_string(),
        },
    )
    .unwrap()
    .password
}

#[test]
fn same_cdr_derivation_is_stable() {
    let harness = setup();
    assert_eq!(derive(&harness), derive(&harness));
}

#[test]
fn factor_packages_and_recovery_metadata_include_schema_versions() {
    let harness = setup();
    let local_text = std::fs::read_to_string(&harness.paths.local_factor_path).unwrap();
    let usb_text = std::fs::read_to_string(usb_package_file(harness.usb_dir.path())).unwrap();
    let recovery_text = std::fs::read_to_string(&harness.paths.recovery_path).unwrap();

    assert!(local_text.contains("\"schemaVersion\""));
    assert!(local_text.contains("\"packageVersion\""));
    assert!(usb_text.contains("\"schemaVersion\""));
    assert!(usb_text.contains("\"packageVersion\""));
    assert!(recovery_text.contains("\"schemaVersion\""));
}

#[test]
fn usb_package_authentication_verification_rejects_wrong_mnemonic() {
    let harness = setup();
    let ok = verify_usb_package(VerifyUsbPackageRequest {
        mnemonic: MNEMONIC.to_string(),
        usb_path: harness.usb_dir.path().to_string_lossy().to_string(),
    })
    .unwrap();
    assert!(ok.valid);

    assert!(verify_usb_package(VerifyUsbPackageRequest {
        mnemonic: "wrong mnemonic phrase".to_string(),
        usb_path: harness.usb_dir.path().to_string_lossy().to_string(),
    })
    .is_err());
}

#[test]
fn recovery_requires_two_valid_factors() {
    let harness = setup();
    let usb_file = usb_package_file(harness.usb_dir.path());
    std::fs::remove_file(&usb_file).unwrap();

    assert!(recover_usb_with_provider(
        &harness.paths,
        &harness.provider,
        RecoverUsbRequest {
            mnemonic: "wrong mnemonic phrase".to_string(),
            usb_path: harness.usb_dir.path().to_string_lossy().to_string(),
        },
    )
    .is_err());

    recover_usb_with_provider(
        &harness.paths,
        &harness.provider,
        RecoverUsbRequest {
            mnemonic: MNEMONIC.to_string(),
            usb_path: harness.usb_dir.path().to_string_lossy().to_string(),
        },
    )
    .unwrap();
    assert!(usb_file.exists());

    let new_app_dir = tempfile::tempdir().unwrap();
    let new_paths = StoragePaths::from_app_dir(new_app_dir.path().to_path_buf());
    let new_provider =
        FallbackPlatformFactorProvider::new(new_paths.app_dir.clone(), "replacement-device");
    recover_local_with_provider(
        &new_paths,
        &new_provider,
        RecoverLocalRequest {
            mnemonic: MNEMONIC.to_string(),
            usb_path: harness.usb_dir.path().to_string_lossy().to_string(),
        },
    )
    .unwrap();
    assert!(new_paths.local_factor_path.exists());

    assert!(recover_local_with_provider(
        &new_paths,
        &new_provider,
        RecoverLocalRequest {
            mnemonic: "wrong mnemonic phrase".to_string(),
            usb_path: harness.usb_dir.path().to_string_lossy().to_string(),
        },
    )
    .is_err());
}

#[test]
fn local_and_usb_can_reset_mnemonic_without_changing_passwords() {
    let harness = setup();
    let before = derive(&harness);

    reset_mnemonic_with_provider(
        &harness.paths,
        &harness.provider,
        ResetMnemonicRequest {
            new_mnemonic: NEW_MNEMONIC.to_string(),
            usb_path: harness.usb_dir.path().to_string_lossy().to_string(),
        },
    )
    .unwrap();

    let after = derive_password_with_provider(
        &harness.paths,
        &harness.provider,
        DerivePasswordRequest {
            record_id: harness.record_id,
            version: Some(harness.version),
            mnemonic: NEW_MNEMONIC.to_string(),
            usb_path: harness.usb_dir.path().to_string_lossy().to_string(),
        },
    )
    .unwrap()
    .password;
    assert_eq!(before, after);

    assert!(derive_password_with_provider(
        &harness.paths,
        &harness.provider,
        DerivePasswordRequest {
            record_id: harness.record_id,
            version: Some(harness.version),
            mnemonic: MNEMONIC.to_string(),
            usb_path: harness.usb_dir.path().to_string_lossy().to_string(),
        },
    )
    .is_err());
}

#[test]
fn usb_cdr_backup_sync_and_restore_round_trip() {
    let harness = setup();
    let usb_path = harness.usb_dir.path().to_string_lossy().to_string();

    let status = get_usb_cdr_status_with_provider(
        &harness.paths,
        &harness.provider,
        UsbCdrRequest {
            usb_path: usb_path.clone(),
        },
    )
    .unwrap();
    assert_eq!(status.status, "local_newer");

    sync_cdr_to_usb_with_provider(
        &harness.paths,
        &harness.provider,
        UsbCdrRequest {
            usb_path: usb_path.clone(),
        },
    )
    .unwrap();
    let status = get_usb_cdr_status_with_provider(
        &harness.paths,
        &harness.provider,
        UsbCdrRequest {
            usb_path: usb_path.clone(),
        },
    )
    .unwrap();
    assert_eq!(status.status, "consistent");

    add_credential_with_provider(
        &harness.paths,
        &harness.provider,
        AddCredentialRequest {
            display_name: "Admin Gateway".to_string(),
            service_hint: "gateway.internal".to_string(),
            account_hint: "root".to_string(),
            notes: String::new(),
            encoding_descriptor: Some(EncodingDescriptor::default()),
        },
    )
    .unwrap();
    let status = get_usb_cdr_status_with_provider(
        &harness.paths,
        &harness.provider,
        UsbCdrRequest {
            usb_path: usb_path.clone(),
        },
    )
    .unwrap();
    assert_eq!(status.status, "local_newer");

    restore_cdr_from_usb_with_provider(
        &harness.paths,
        &harness.provider,
        UsbCdrRequest { usb_path },
    )
    .unwrap();
    let records = CdrStore::new(&harness.paths.db_path).list_all().unwrap();
    assert_eq!(records.len(), 1);
}

#[test]
fn reenrollment_is_blocked_after_local_state_exists() {
    let harness = setup();
    let err = enroll_with_provider(
        &harness.paths,
        &harness.provider,
        EnrollmentRequest {
            mnemonic: MNEMONIC.to_string(),
            usb_path: harness.usb_dir.path().to_string_lossy().to_string(),
        },
    )
    .unwrap_err();
    assert!(err.to_string().contains("already enrolled"));
}

#[test]
fn display_service_and_account_hints_do_not_affect_derivation() {
    let harness = setup();
    let before = derive(&harness);
    update_credential_display_with_provider(
        &harness.paths,
        &harness.provider,
        UpdateCredentialDisplayRequest {
            record_id: harness.record_id,
            version: harness.version,
            display_name: "Renamed ERP".to_string(),
            service_hint: "changed.example".to_string(),
            account_hint: "bob".to_string(),
            notes: "metadata only".to_string(),
            encoding_descriptor: Some(EncodingDescriptor::default()),
        },
    )
    .unwrap();
    let after = derive(&harness);
    assert_eq!(before, after);
}

#[test]
fn derivation_path_fields_change_service_secret() {
    let user_id = Uuid::new_v4();
    let record_id = Uuid::new_v4();
    let derivation_key = [3_u8; 32];
    let salt = [4_u8; 16];
    let base =
        kdf::derive_service_secret(&derivation_key, &user_id, 1, &record_id, 1, &salt).unwrap();
    assert_ne!(
        base,
        kdf::derive_service_secret(&derivation_key, &user_id, 2, &record_id, 1, &salt).unwrap()
    );
    assert_ne!(
        base,
        kdf::derive_service_secret(&derivation_key, &user_id, 1, &Uuid::new_v4(), 1, &salt)
            .unwrap()
    );
    assert_ne!(
        base,
        kdf::derive_service_secret(&derivation_key, &user_id, 1, &record_id, 2, &salt).unwrap()
    );
    assert_ne!(
        base,
        kdf::derive_service_secret(&derivation_key, &user_id, 1, &record_id, 1, &[5_u8; 16])
            .unwrap()
    );
}

#[test]
fn encoding_descriptor_change_requires_rotation() {
    let harness = setup();
    let mut changed = EncodingDescriptor::default();
    changed.length += 1;
    let err = update_credential_display_with_provider(
        &harness.paths,
        &harness.provider,
        UpdateCredentialDisplayRequest {
            record_id: harness.record_id,
            version: harness.version,
            display_name: "Legacy ERP".to_string(),
            service_hint: "erp.internal".to_string(),
            account_hint: "alice".to_string(),
            notes: String::new(),
            encoding_descriptor: Some(changed.clone()),
        },
    )
    .unwrap_err();
    assert!(err.to_string().contains("immutable"));

    let rotated = rotate_credential_with_provider(
        &harness.paths,
        &harness.provider,
        RotateCredentialRequest {
            record_id: harness.record_id,
            encoding_descriptor: Some(changed),
        },
    )
    .unwrap();
    assert_eq!(rotated.version, harness.version + 1);
}

#[test]
fn rotation_cancel_and_commit_preserve_expected_states() {
    let harness = setup();
    let store = CdrStore::new(&harness.paths.db_path);
    let before = derive(&harness);

    let pending = rotate_credential_with_provider(
        &harness.paths,
        &harness.provider,
        RotateCredentialRequest {
            record_id: harness.record_id,
            encoding_descriptor: None,
        },
    )
    .unwrap();
    assert_eq!(pending.version, harness.version + 1);
    assert_eq!(
        store
            .get(harness.record_id, Some(harness.version))
            .unwrap()
            .state,
        CredentialState::Active
    );

    cancel_rotation_with_provider(
        &harness.paths,
        &harness.provider,
        CancelRotationRequest {
            record_id: harness.record_id,
            version: pending.version,
        },
    )
    .unwrap();
    assert!(store.get(harness.record_id, Some(pending.version)).is_err());
    assert_eq!(before, derive(&harness));

    let pending = rotate_credential_with_provider(
        &harness.paths,
        &harness.provider,
        RotateCredentialRequest {
            record_id: harness.record_id,
            encoding_descriptor: None,
        },
    )
    .unwrap();
    confirm_rotation_with_provider(
        &harness.paths,
        &harness.provider,
        ConfirmRotationRequest {
            record_id: harness.record_id,
            version: pending.version,
        },
    )
    .unwrap();

    let old = store.get(harness.record_id, Some(harness.version)).unwrap();
    let new_record = store.get(harness.record_id, Some(pending.version)).unwrap();
    assert_eq!(old.state, CredentialState::Retired);
    assert_eq!(new_record.state, CredentialState::Active);
}

#[test]
fn missing_factors_fail_derivation() {
    let harness = setup();
    assert!(derive_password_with_provider(
        &harness.paths,
        &harness.provider,
        DerivePasswordRequest {
            record_id: harness.record_id,
            version: Some(harness.version),
            mnemonic: String::new(),
            usb_path: harness.usb_dir.path().to_string_lossy().to_string(),
        },
    )
    .is_err());

    assert!(derive_password_with_provider(
        &harness.paths,
        &harness.provider,
        DerivePasswordRequest {
            record_id: harness.record_id,
            version: Some(harness.version),
            mnemonic: MNEMONIC.to_string(),
            usb_path: harness
                .usb_dir
                .path()
                .join("missing")
                .to_string_lossy()
                .to_string(),
        },
    )
    .is_err());

    std::fs::remove_file(&harness.paths.local_factor_path).unwrap();
    assert!(derive_password_with_provider(
        &harness.paths,
        &harness.provider,
        DerivePasswordRequest {
            record_id: harness.record_id,
            version: Some(harness.version),
            mnemonic: MNEMONIC.to_string(),
            usb_path: harness.usb_dir.path().to_string_lossy().to_string(),
        },
    )
    .is_err());
}

#[test]
fn corrupt_usb_package_fails() {
    let harness = setup();
    let usb_file = usb_package_file(harness.usb_dir.path());
    std::fs::write(usb_file, b"{\"packageMac\":\"broken\"}").unwrap();
    assert!(derive_password_with_provider(
        &harness.paths,
        &harness.provider,
        DerivePasswordRequest {
            record_id: harness.record_id,
            version: Some(harness.version),
            mnemonic: MNEMONIC.to_string(),
            usb_path: harness.usb_dir.path().to_string_lossy().to_string(),
        },
    )
    .is_err());
}

#[test]
fn corrupt_cdr_mac_fails() {
    let harness = setup();
    let store = CdrStore::new(&harness.paths.db_path);
    store
        .corrupt_mac_for_test(harness.record_id, harness.version)
        .unwrap();
    assert!(derive_password_with_provider(
        &harness.paths,
        &harness.provider,
        DerivePasswordRequest {
            record_id: harness.record_id,
            version: Some(harness.version),
            mnemonic: MNEMONIC.to_string(),
            usb_path: harness.usb_dir.path().to_string_lossy().to_string(),
        },
    )
    .is_err());
}

#[test]
fn platform_provider_trait_smoke_tests() {
    fn smoke(provider: &dyn PlatformFactorProvider) {
        let device_id = provider.get_or_create_device_id().unwrap();
        let device_secret = provider.get_or_create_device_secret().unwrap();
        assert!(!device_id.is_empty());
        assert_eq!(device_secret.expose().len(), 32);
        let protected = provider.protect_local_package(b"package").unwrap();
        let plaintext = provider.unprotect_local_package(&protected).unwrap();
        assert_eq!(plaintext, b"package");
    }

    let fallback_dir = tempfile::tempdir().unwrap();
    smoke(&FallbackPlatformFactorProvider::new(
        fallback_dir.path().to_path_buf(),
        "fallback-test",
    ));

    let linux_dir = tempfile::tempdir().unwrap();
    smoke(&LinuxPlatformFactorProvider::new(
        linux_dir.path().to_path_buf(),
    ));

    let mac_dir = tempfile::tempdir().unwrap();
    smoke(&MacOSPlatformFactorProvider::fallback_only(
        mac_dir.path().to_path_buf(),
    ));

    let windows_dir = tempfile::tempdir().unwrap();
    smoke(&WindowsPlatformFactorProvider::new(
        windows_dir.path().to_path_buf(),
    ));
}
