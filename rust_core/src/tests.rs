use crate::crypto::kdf;
use crate::domain::{CredentialState, EncodingDescriptor, PasswordDerivationAlgorithm};
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
use crate::storage::{
    create_local_factor_package, create_usb_factor_package, load_local_factor_payload,
    load_usb_factor_payload, read_usb_factor_package, usb_package_file, write_local_factor_package,
    write_usb_factor_package, CdrStore, StoragePaths,
};
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
    setup_with_algorithm(PasswordDerivationAlgorithm::HkdfSha256)
}

fn setup_with_algorithm(password_derivation_algorithm: PasswordDerivationAlgorithm) -> Harness {
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
            password_derivation_algorithm,
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
    derive_with_mnemonic(harness, MNEMONIC).unwrap()
}

fn derive_with_mnemonic(
    harness: &Harness,
    mnemonic: &str,
) -> Result<String, crate::error::KeylessPassError> {
    derive_password_with_provider(
        &harness.paths,
        &harness.provider,
        DerivePasswordRequest {
            record_id: harness.record_id,
            version: Some(harness.version),
            mnemonic: mnemonic.to_string(),
            usb_path: harness.usb_dir.path().to_string_lossy().to_string(),
        },
    )
    .map(|response| response.password)
}

fn rewrite_local_payload(
    harness: &Harness,
    mutate: impl FnOnce(&mut crate::domain::LocalFactorPayload),
) {
    let (package, mut payload) =
        load_local_factor_payload(&harness.provider, &harness.paths.local_factor_path).unwrap();
    mutate(&mut payload);
    let package = create_local_factor_package(
        &harness.provider,
        package.user_id,
        &package.device_id,
        &package.platform,
        &payload,
    )
    .unwrap();
    write_local_factor_package(&harness.paths.local_factor_path, &package).unwrap();
}

fn rewrite_usb_payload(
    harness: &Harness,
    mutate: impl FnOnce(&mut crate::domain::UsbFactorPayload),
) {
    let (package, mut payload) = load_usb_factor_payload(harness.usb_dir.path()).unwrap();
    mutate(&mut payload);
    let package = create_usb_factor_package(
        package.user_id,
        &package.device_id,
        &package.platform,
        &payload,
    )
    .unwrap();
    write_usb_factor_package(harness.usb_dir.path(), &package).unwrap();
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
fn factor_payloads_do_not_persist_plaintext_master_key() {
    let harness = setup();
    let (_, local_payload) =
        load_local_factor_payload(&harness.provider, &harness.paths.local_factor_path).unwrap();
    let (_, usb_payload) = load_usb_factor_payload(harness.usb_dir.path()).unwrap();

    let local_json = serde_json::to_value(local_payload).unwrap();
    let usb_json = serde_json::to_value(usb_payload).unwrap();
    assert!(local_json.get("kMaster").is_none());
    assert!(local_json.get("usbSecret").is_none());
    assert!(local_json.get("deviceSecret").is_none());
    assert!(usb_json.get("kMaster").is_none());
    assert!(usb_json.get("deviceSecret").is_none());
    assert!(local_json.get("wMc").is_some());
    assert!(usb_json.get("wMu").is_some());
    assert!(usb_json.get("wCu").is_some());
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
fn legacy_usb_package_returns_mnemonic_encrypted_recovery_error() {
    let harness = setup();
    let mut package = read_usb_factor_package(harness.usb_dir.path()).unwrap();
    package.schema_version = crate::domain::LEGACY_FACTOR_PACKAGE_SCHEMA_VERSION;
    write_usb_factor_package(harness.usb_dir.path(), &package).unwrap();

    let err = load_usb_factor_payload(harness.usb_dir.path()).unwrap_err();
    assert!(err.to_string().contains(
        "legacy factor package stores master-key payload and does not support strict pairwise-wrapper recovery"
    ));
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
fn mnemonic_and_usb_can_rebuild_local_package() {
    let harness = setup();
    let before = derive(&harness);
    let usb_path = harness.usb_dir.path().to_string_lossy().to_string();
    sync_cdr_to_usb_with_provider(
        &harness.paths,
        &harness.provider,
        UsbCdrRequest {
            usb_path: usb_path.clone(),
        },
    )
    .unwrap();

    let new_app_dir = tempfile::tempdir().unwrap();
    let new_paths = StoragePaths::from_app_dir(new_app_dir.path().to_path_buf());
    let new_provider =
        FallbackPlatformFactorProvider::new(new_paths.app_dir.clone(), "replacement-device");
    recover_local_with_provider(
        &new_paths,
        &new_provider,
        RecoverLocalRequest {
            mnemonic: MNEMONIC.to_string(),
            usb_path: usb_path.clone(),
        },
    )
    .unwrap();

    let after = derive_password_with_provider(
        &new_paths,
        &new_provider,
        DerivePasswordRequest {
            record_id: harness.record_id,
            version: Some(harness.version),
            mnemonic: MNEMONIC.to_string(),
            usb_path,
        },
    )
    .unwrap()
    .password;
    assert_eq!(before, after);
}

#[test]
fn mnemonic_and_local_can_rebuild_usb_package() {
    let harness = setup();
    let before = derive(&harness);
    let usb_file = usb_package_file(harness.usb_dir.path());
    std::fs::remove_file(&usb_file).unwrap();

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
    assert_eq!(before, derive(&harness));
}

#[test]
fn single_factors_do_not_recover_master_key() {
    let only_usb = setup();
    std::fs::remove_file(&only_usb.paths.local_factor_path).unwrap();
    assert!(reset_mnemonic_with_provider(
        &only_usb.paths,
        &only_usb.provider,
        ResetMnemonicRequest {
            new_mnemonic: NEW_MNEMONIC.to_string(),
            usb_path: only_usb.usb_dir.path().to_string_lossy().to_string(),
        },
    )
    .is_err());

    let only_local = setup();
    std::fs::remove_file(usb_package_file(only_local.usb_dir.path())).unwrap();
    assert!(reset_mnemonic_with_provider(
        &only_local.paths,
        &only_local.provider,
        ResetMnemonicRequest {
            new_mnemonic: NEW_MNEMONIC.to_string(),
            usb_path: only_local.usb_dir.path().to_string_lossy().to_string(),
        },
    )
    .is_err());

    let mnemonic_only_app = tempfile::tempdir().unwrap();
    let mnemonic_only_usb = tempfile::tempdir().unwrap();
    let mnemonic_only_paths = StoragePaths::from_app_dir(mnemonic_only_app.path().to_path_buf());
    let mnemonic_only_provider =
        FallbackPlatformFactorProvider::new(mnemonic_only_paths.app_dir.clone(), "mnemonic-only");
    assert!(recover_local_with_provider(
        &mnemonic_only_paths,
        &mnemonic_only_provider,
        RecoverLocalRequest {
            mnemonic: MNEMONIC.to_string(),
            usb_path: mnemonic_only_usb.path().to_string_lossy().to_string(),
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
fn copied_usb_without_matching_local_factor_cannot_use_cu_wrapper() {
    let source = setup();
    let other = setup();
    let copied_usb = tempfile::tempdir().unwrap();
    std::fs::copy(
        usb_package_file(source.usb_dir.path()),
        usb_package_file(copied_usb.path()),
    )
    .unwrap();

    let err = reset_mnemonic_with_provider(
        &other.paths,
        &other.provider,
        ResetMnemonicRequest {
            new_mnemonic: NEW_MNEMONIC.to_string(),
            usb_path: copied_usb.path().to_string_lossy().to_string(),
        },
    )
    .unwrap_err();
    assert!(
        err.to_string().contains("does not match")
            || err.to_string().contains("user mismatch")
            || err.to_string().contains("managed computer")
    );
}

#[test]
fn tampering_pairwise_wrappers_and_bound_ids_fails() {
    let tamper_w_mc = setup();
    rewrite_local_payload(&tamper_w_mc, |payload| {
        payload.w_mc.tag = crate::crypto::b64_encode(&[0_u8; 16]);
    });
    assert!(derive_with_mnemonic(&tamper_w_mc, MNEMONIC).is_err());

    let tamper_w_mu = setup();
    rewrite_usb_payload(&tamper_w_mu, |payload| {
        payload.w_mu.nonce = crate::crypto::b64_encode(&[1_u8; 12]);
    });
    assert!(derive_with_mnemonic(&tamper_w_mu, MNEMONIC).is_err());

    let tamper_w_cu = setup();
    rewrite_usb_payload(&tamper_w_cu, |payload| {
        payload.w_cu.ciphertext = crate::crypto::b64_encode(&[2_u8; 32]);
    });
    assert!(reset_mnemonic_with_provider(
        &tamper_w_cu.paths,
        &tamper_w_cu.provider,
        ResetMnemonicRequest {
            new_mnemonic: NEW_MNEMONIC.to_string(),
            usb_path: tamper_w_cu.usb_dir.path().to_string_lossy().to_string(),
        },
    )
    .is_err());

    let tamper_usb_id = setup();
    rewrite_usb_payload(&tamper_usb_id, |payload| {
        payload.usb_id = uuid::Uuid::new_v4().to_string();
    });
    assert!(derive_with_mnemonic(&tamper_usb_id, MNEMONIC).is_err());

    let tamper_salt_c = setup();
    rewrite_local_payload(&tamper_salt_c, |payload| {
        payload.salt_c = crate::crypto::b64_encode(&[3_u8; 16]);
    });
    assert!(derive_with_mnemonic(&tamper_salt_c, MNEMONIC).is_err());

    let tamper_salt_u = setup();
    rewrite_usb_payload(&tamper_salt_u, |payload| {
        payload.salt_u = crate::crypto::b64_encode(&[4_u8; 16]);
    });
    assert!(derive_with_mnemonic(&tamper_salt_u, MNEMONIC).is_err());

    let tamper_device_id = setup();
    let (package, payload) = load_usb_factor_payload(tamper_device_id.usb_dir.path()).unwrap();
    let package = create_usb_factor_package(
        package.user_id,
        "copied-device-id",
        &package.platform,
        &payload,
    )
    .unwrap();
    write_usb_factor_package(tamper_device_id.usb_dir.path(), &package).unwrap();
    assert!(derive_with_mnemonic(&tamper_device_id, MNEMONIC).is_err());

    let tamper_user_id = setup();
    let (package, mut payload) = load_usb_factor_payload(tamper_user_id.usb_dir.path()).unwrap();
    let other_user_id = uuid::Uuid::new_v4();
    payload.user_id = other_user_id;
    let package = create_usb_factor_package(
        other_user_id,
        &package.device_id,
        &package.platform,
        &payload,
    )
    .unwrap();
    write_usb_factor_package(tamper_user_id.usb_dir.path(), &package).unwrap();
    assert!(derive_with_mnemonic(&tamper_user_id, MNEMONIC).is_err());
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
            password_derivation_algorithm: PasswordDerivationAlgorithm::HkdfSha256,
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
fn selectable_service_derivation_algorithms_are_stable() {
    let algorithms = [
        PasswordDerivationAlgorithm::HkdfSha256,
        PasswordDerivationAlgorithm::Argon2id,
        PasswordDerivationAlgorithm::Scrypt,
        PasswordDerivationAlgorithm::Pbkdf2HmacSha256,
    ];
    let mut outputs = std::collections::BTreeSet::new();
    for algorithm in algorithms {
        let harness = setup_with_algorithm(algorithm);
        let config = crate::storage::read_config(&harness.paths).unwrap();
        assert_eq!(config.password_derivation_algorithm, algorithm);
        let first = derive(&harness);
        let second = derive(&harness);
        assert_eq!(first, second);
        assert!(outputs.insert(first));
    }
}

#[test]
fn legacy_config_without_algorithm_defaults_to_hkdf() {
    let value = serde_json::json!({
        "appVersion": "0.1.0",
        "userId": Uuid::new_v4(),
        "platform": "test",
        "deviceId": "device",
        "cdrStorePath": "/tmp/keylesspass-test.sqlite3",
        "localFactorPath": "/tmp/keylesspass-local-factor.json",
        "createdAt": "2026-01-01T00:00:00Z",
        "updatedAt": "2026-01-01T00:00:00Z"
    });
    let config: crate::domain::AppConfig = serde_json::from_value(value).unwrap();
    assert_eq!(
        config.password_derivation_algorithm,
        PasswordDerivationAlgorithm::HkdfSha256
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
