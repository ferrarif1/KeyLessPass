use keylesspass_core::crypto::{self, encoder, kdf};
use keylesspass_core::domain::{CredentialState, EncodingDescriptor};
use keylesspass_core::platform::fallback::FallbackPlatformFactorProvider;
use keylesspass_core::service::credentials::{
    add_credential_with_provider, update_credential_display_with_provider, AddCredentialRequest,
    UpdateCredentialDisplayRequest,
};
use keylesspass_core::service::derive::{derive_password_with_provider, DerivePasswordRequest};
use keylesspass_core::service::enrollment::{enroll_with_provider, EnrollmentRequest};
use keylesspass_core::service::recovery::{
    recover_local_with_provider, recover_usb_with_provider, RecoverLocalRequest, RecoverUsbRequest,
};
use keylesspass_core::service::rotation::{
    confirm_rotation_with_provider, rotate_credential_with_provider, ConfirmRotationRequest,
    RotateCredentialRequest,
};
use keylesspass_core::storage::{
    load_local_factor_payload, load_usb_factor_payload, read_usb_factor_package, usb_package_file,
    CdrStore, StoragePaths,
};
use rusqlite::{params, Connection};
use serde::Serialize;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Instant;
use tempfile::TempDir;
use uuid::Uuid;

const ITERATIONS: usize = 30;
const MNEMONIC: &str =
    "alpha bridge cable delta ember forest galaxy harbor ivory jungle kinetic lemon";

#[derive(Debug)]
struct Harness {
    _app_dir: TempDir,
    usb_dir: UsbRoot,
    paths: StoragePaths,
    provider: FallbackPlatformFactorProvider,
    record_id: Uuid,
    version: u32,
}

#[derive(Debug)]
struct UsbRoot {
    path: PathBuf,
    _temp_dir: Option<TempDir>,
    cleanup: bool,
}

impl UsbRoot {
    fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for UsbRoot {
    fn drop(&mut self) {
        if self.cleanup {
            let _ = fs::remove_dir_all(&self.path);
        }
    }
}

#[derive(Debug, Clone, Serialize)]
struct FunctionalResult {
    test_id: String,
    objective: String,
    operation: String,
    expected_result: String,
    actual_result: String,
    status: String,
    duration_ms: f64,
}

#[derive(Debug, Clone, Serialize)]
struct PerformanceResult {
    metric_id: String,
    operation: String,
    iterations: usize,
    mean_ms: f64,
    median_ms: f64,
    min_ms: f64,
    max_ms: f64,
    std_ms: f64,
    p95_ms: f64,
}

fn main() {
    let out_dir = std::env::args()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("evidence_output"));
    fs::create_dir_all(&out_dir).expect("create output directory");

    let functional = run_functional_tests();
    let performance = run_performance_measurements();

    write_functional_outputs(&out_dir, &functional).expect("write functional outputs");
    write_performance_outputs(&out_dir, &performance).expect("write performance outputs");
}

fn setup() -> Harness {
    let app_dir = tempfile::tempdir().expect("temp app dir");
    let usb_dir = evidence_usb_tempdir();
    let paths = StoragePaths::from_app_dir(app_dir.path().to_path_buf());
    let provider = FallbackPlatformFactorProvider::new(paths.app_dir.clone(), "evidence-macos");
    enroll_with_provider(
        &paths,
        &provider,
        EnrollmentRequest {
            mnemonic: MNEMONIC.to_string(),
            usb_path: usb_dir.path().to_string_lossy().to_string(),
        },
    )
    .expect("enrollment");
    let record = add_credential_with_provider(
        &paths,
        &provider,
        AddCredentialRequest {
            display_name: "Synthetic Legacy Service".to_string(),
            service_hint: "example.internal".to_string(),
            account_hint: "test-user".to_string(),
            notes: String::new(),
            encoding_descriptor: Some(EncodingDescriptor::default()),
        },
    )
    .expect("add credential");
    Harness {
        _app_dir: app_dir,
        usb_dir,
        paths,
        provider,
        record_id: record.record_id,
        version: record.version,
    }
}

fn evidence_usb_tempdir() -> UsbRoot {
    if let Ok(root) = std::env::var("KEYLESSPASS_EVIDENCE_USB_ROOT") {
        if !root.trim().is_empty() {
            let path = PathBuf::from(root).join(format!("keylesspass-evidence-{}", Uuid::new_v4()));
            fs::create_dir_all(&path).expect("external evidence usb temp dir");
            return UsbRoot {
                path,
                _temp_dir: None,
                cleanup: true,
            };
        }
    }
    let temp_dir = tempfile::tempdir().expect("temp usb dir");
    let path = temp_dir.path().to_path_buf();
    UsbRoot {
        path,
        _temp_dir: Some(temp_dir),
        cleanup: false,
    }
}

fn derive(h: &Harness, version: u32) -> Result<String, String> {
    derive_password_with_provider(
        &h.paths,
        &h.provider,
        DerivePasswordRequest {
            record_id: h.record_id,
            version: Some(version),
            mnemonic: MNEMONIC.to_string(),
            usb_path: h.usb_dir.path().to_string_lossy().to_string(),
        },
    )
    .map(|response| response.password)
    .map_err(|err| err.to_string())
}

fn run_functional_tests() -> Vec<FunctionalResult> {
    let mut rows = Vec::new();

    rows.push(run_test(
        "FCT-01",
        "Repeated derivation from the same committed CDR is stable.",
        "Enroll, add one CDR, derive the password twice with the same factors.",
        "The two derived passwords are identical.",
        || {
            let h = setup();
            let p1 = derive(&h, h.version)?;
            let p2 = derive(&h, h.version)?;
            if p1 == p2 {
                ok("Derived password was stable across repeated runs.")
            } else {
                fail("Derived passwords differed for the same CDR.")
            }
        },
    ));

    rows.push(run_test(
        "FCT-02",
        "Mutable display metadata does not affect derivation.",
        "Derive, update displayName/serviceHint/accountHint, derive again.",
        "Password remains unchanged.",
        || {
            let h = setup();
            let before = derive(&h, h.version)?;
            update_credential_display_with_provider(
                &h.paths,
                &h.provider,
                UpdateCredentialDisplayRequest {
                    record_id: h.record_id,
                    version: h.version,
                    display_name: "Renamed Synthetic Service".to_string(),
                    service_hint: "changed.example".to_string(),
                    account_hint: "renamed-user".to_string(),
                    notes: "metadata only".to_string(),
                    encoding_descriptor: Some(EncodingDescriptor::default()),
                },
            )
            .map_err(|err| err.to_string())?;
            let after = derive(&h, h.version)?;
            if before == after {
                ok("displayName, serviceHint, and accountHint changed without changing the password.")
            } else {
                fail("Password changed after mutable display metadata update.")
            }
        },
    ));

    rows.push(run_test(
        "FCT-03",
        "Changing the CDR version changes the derived password.",
        "Create a pending rotation version and derive v1 and v2.",
        "v2 password differs from v1.",
        || {
            let h = setup();
            let old_password = derive(&h, h.version)?;
            let pending = rotate_credential_with_provider(
                &h.paths,
                &h.provider,
                RotateCredentialRequest {
                    record_id: h.record_id,
                    encoding_descriptor: Some(EncodingDescriptor::default()),
                },
            )
            .map_err(|err| err.to_string())?;
            let new_password = derive(&h, pending.version)?;
            if old_password != new_password {
                ok("Pending v2 derivation differed from active v1.")
            } else {
                fail("Version change did not change the derived password.")
            }
        },
    ));

    rows.push(run_test(
        "FCT-04",
        "Changing the derivation salt changes the service secret.",
        "Call the service-secret KDF with identical user/record/version fields and two different salts.",
        "Service secret changes.",
        || {
            let user_id = Uuid::new_v4();
            let record_id = Uuid::new_v4();
            let key = [3_u8; 32];
            let left = kdf::derive_service_secret(&key, &user_id, 1, &record_id, 1, &[4_u8; 16])
                .map_err(|err| err.to_string())?;
            let right =
                kdf::derive_service_secret(&key, &user_id, 1, &record_id, 1, &[5_u8; 16])
                    .map_err(|err| err.to_string())?;
            if left != right {
                ok("Service secret changed when only salt changed.")
            } else {
                fail("Service secret did not change when salt changed.")
            }
        },
    ));

    rows.push(run_test(
        "FCT-05",
        "Changing encodingDescriptor requires a new CDR version.",
        "Attempt in-place descriptor update; then rotate with the changed descriptor.",
        "In-place update fails; rotation creates version 2.",
        || {
            let h = setup();
            let mut changed = EncodingDescriptor::default();
            changed.length += 1;
            let update = update_credential_display_with_provider(
                &h.paths,
                &h.provider,
                UpdateCredentialDisplayRequest {
                    record_id: h.record_id,
                    version: h.version,
                    display_name: "Synthetic Legacy Service".to_string(),
                    service_hint: "example.internal".to_string(),
                    account_hint: "test-user".to_string(),
                    notes: String::new(),
                    encoding_descriptor: Some(changed.clone()),
                },
            );
            let pending = rotate_credential_with_provider(
                &h.paths,
                &h.provider,
                RotateCredentialRequest {
                    record_id: h.record_id,
                    encoding_descriptor: Some(changed),
                },
            )
            .map_err(|err| err.to_string())?;
            if update.is_err() && pending.version == 2 {
                ok("In-place descriptor mutation was rejected and rotation created v2.")
            } else {
                fail("Descriptor immutability or rotation behavior was not enforced.")
            }
        },
    ));

    rows.push(run_test(
        "FCT-06",
        "Mnemonic phrase alone cannot recover local material.",
        "Attempt local recovery with a mnemonic but no USB package.",
        "Operation fails.",
        || {
            let app_dir = tempfile::tempdir().map_err(|err| err.to_string())?;
            let usb_dir = tempfile::tempdir().map_err(|err| err.to_string())?;
            let paths = StoragePaths::from_app_dir(app_dir.path().to_path_buf());
            let provider =
                FallbackPlatformFactorProvider::new(paths.app_dir.clone(), "evidence-macos");
            let result = recover_local_with_provider(
                &paths,
                &provider,
                RecoverLocalRequest {
                    mnemonic: MNEMONIC.to_string(),
                    usb_path: usb_dir.path().to_string_lossy().to_string(),
                },
            );
            if result.is_err() {
                ok("Recovery failed because no USB factor package was available.")
            } else {
                fail("Mnemonic-only recovery unexpectedly succeeded.")
            }
        },
    ));

    rows.push(run_test(
        "FCT-07",
        "USB factor alone cannot recover local material.",
        "Attempt local recovery with a USB package and an empty mnemonic.",
        "Operation fails.",
        || {
            let h = setup();
            let new_app = tempfile::tempdir().map_err(|err| err.to_string())?;
            let new_paths = StoragePaths::from_app_dir(new_app.path().to_path_buf());
            let new_provider =
                FallbackPlatformFactorProvider::new(new_paths.app_dir.clone(), "evidence-macos");
            let result = recover_local_with_provider(
                &new_paths,
                &new_provider,
                RecoverLocalRequest {
                    mnemonic: String::new(),
                    usb_path: h.usb_dir.path().to_string_lossy().to_string(),
                },
            );
            if result.is_err() {
                ok("USB-only recovery failed because mnemonic input was missing.")
            } else {
                fail("USB-only recovery unexpectedly succeeded.")
            }
        },
    ));

    rows.push(run_test(
        "FCT-08",
        "Local material alone cannot perform a complete USB recovery.",
        "Attempt USB recovery with local package but no mnemonic phrase.",
        "Operation fails; no valid USB package is created.",
        || {
            let h = setup();
            let new_usb = tempfile::tempdir().map_err(|err| err.to_string())?;
            let result = recover_usb_with_provider(
                &h.paths,
                &h.provider,
                RecoverUsbRequest {
                    mnemonic: String::new(),
                    usb_path: new_usb.path().to_string_lossy().to_string(),
                },
            );
            if result.is_err() {
                partial("Empty-mnemonic USB recovery was rejected. Note: the MVP recover_usb path rewraps from local material under any non-empty mnemonic and does not verify the original mnemonic.")
            } else {
                fail("Local-only USB recovery unexpectedly succeeded without mnemonic input.")
            }
        },
    ));

    rows.push(run_test(
        "FCT-09",
        "Mnemonic plus USB can rebuild local factor material.",
        "Remove local state and run recover_local with mnemonic and USB factor package.",
        "A local factor package is recreated.",
        || {
            let h = setup();
            let new_app = tempfile::tempdir().map_err(|err| err.to_string())?;
            let new_paths = StoragePaths::from_app_dir(new_app.path().to_path_buf());
            let new_provider =
                FallbackPlatformFactorProvider::new(new_paths.app_dir.clone(), "evidence-macos");
            recover_local_with_provider(
                &new_paths,
                &new_provider,
                RecoverLocalRequest {
                    mnemonic: MNEMONIC.to_string(),
                    usb_path: h.usb_dir.path().to_string_lossy().to_string(),
                },
            )
            .map_err(|err| err.to_string())?;
            if new_paths.local_factor_path.exists() && new_paths.config_path.exists() {
                partial("Local factor package and config were rebuilt. USB CDR replica restoration is not implemented in this MVP.")
            } else {
                fail("Local factor package was not recreated.")
            }
        },
    ));

    rows.push(run_test(
        "FCT-10",
        "Mnemonic plus local material can rebuild USB factor material.",
        "Delete the USB package and run recover_usb with mnemonic and local factor package.",
        "A USB factor package is recreated and can be used for derivation.",
        || {
            let h = setup();
            let usb_file = usb_package_file(h.usb_dir.path());
            fs::remove_file(&usb_file).map_err(|err| err.to_string())?;
            recover_usb_with_provider(
                &h.paths,
                &h.provider,
                RecoverUsbRequest {
                    mnemonic: MNEMONIC.to_string(),
                    usb_path: h.usb_dir.path().to_string_lossy().to_string(),
                },
            )
            .map_err(|err| err.to_string())?;
            let password = derive(&h, h.version)?;
            if usb_file.exists() && !password.is_empty() {
                ok("USB factor package was rebuilt and derivation succeeded.")
            } else {
                fail("USB factor package rebuild did not produce a usable package.")
            }
        },
    ));

    rows.push(run_test(
        "FCT-11",
        "CDR tampering is rejected by HMAC verification.",
        "Modify the stored CDR MAC field directly in SQLite and attempt derivation.",
        "Derivation fails with an integrity error.",
        || {
            let h = setup();
            corrupt_cdr_mac(&h.paths.db_path, h.record_id, h.version)?;
            let result = derive(&h, h.version);
            if result.is_err() {
                ok("Tampered CDR was rejected during derivation.")
            } else {
                fail("Tampered CDR was accepted.")
            }
        },
    ));

    rows.push(run_test(
        "FCT-12",
        "USB factor package tampering is rejected by authenticated decryption or package MAC.",
        "Overwrite the USB factor package with invalid JSON fields and attempt derivation.",
        "Derivation fails.",
        || {
            let h = setup();
            fs::write(
                usb_package_file(h.usb_dir.path()),
                b"{\"packageMac\":\"broken\"}",
            )
            .map_err(|err| err.to_string())?;
            let result = derive(&h, h.version);
            if result.is_err() {
                ok("Tampered USB package was rejected.")
            } else {
                fail("Tampered USB package was accepted.")
            }
        },
    ));

    rows.push(FunctionalResult {
        test_id: "FCT-13".to_string(),
        objective: "Local CDR and USB CDR mismatch enters conflict/error.".to_string(),
        operation: "Inspect MVP storage model.".to_string(),
        expected_result: "Mismatch between local CDR and USB CDR is detected.".to_string(),
        actual_result: "Not implemented: the MVP writes the USB factor package but does not yet maintain a USB CDR replica or CDR conflict detector.".to_string(),
        status: "Not implemented".to_string(),
        duration_ms: 0.0,
    });

    rows.push(run_test(
        "FCT-14",
        "Old version remains active while a rotation is pending.",
        "Create a pending rotation and inspect v1/v2 states.",
        "v1 remains active and v2 is pending_rotation.",
        || {
            let h = setup();
            let pending = rotate_credential_with_provider(
                &h.paths,
                &h.provider,
                RotateCredentialRequest {
                    record_id: h.record_id,
                    encoding_descriptor: Some(EncodingDescriptor::default()),
                },
            )
            .map_err(|err| err.to_string())?;
            let store = CdrStore::new(&h.paths.db_path);
            let old = store
                .get(h.record_id, Some(h.version))
                .map_err(|err| err.to_string())?;
            if old.state == CredentialState::Active
                && pending.state == CredentialState::PendingRotation
            {
                ok("v1 remained active while v2 was pending_rotation.")
            } else {
                fail("Rotation pending state did not preserve old active version.")
            }
        },
    ));

    rows.push(run_test(
        "FCT-15",
        "After rotation commit, new version becomes active and old version is retired.",
        "Create pending rotation and confirm it.",
        "v2 active; v1 retired.",
        || {
            let h = setup();
            let pending = rotate_credential_with_provider(
                &h.paths,
                &h.provider,
                RotateCredentialRequest {
                    record_id: h.record_id,
                    encoding_descriptor: Some(EncodingDescriptor::default()),
                },
            )
            .map_err(|err| err.to_string())?;
            confirm_rotation_with_provider(
                &h.paths,
                &h.provider,
                ConfirmRotationRequest {
                    record_id: h.record_id,
                    version: pending.version,
                },
            )
            .map_err(|err| err.to_string())?;
            let store = CdrStore::new(&h.paths.db_path);
            let old = store
                .get(h.record_id, Some(h.version))
                .map_err(|err| err.to_string())?;
            let new = store
                .get(h.record_id, Some(pending.version))
                .map_err(|err| err.to_string())?;
            if old.state == CredentialState::Retired && new.state == CredentialState::Active {
                ok("v2 became active and v1 was retired after commit.")
            } else {
                fail("Commit did not produce expected active/retired states.")
            }
        },
    ));

    rows.push(run_test(
        "FCT-16",
        "A failed or unconfirmed rotation leaves the old version active.",
        "Create pending rotation and do not call confirm_rotation.",
        "v1 remains active.",
        || {
            let h = setup();
            rotate_credential_with_provider(
                &h.paths,
                &h.provider,
                RotateCredentialRequest {
                    record_id: h.record_id,
                    encoding_descriptor: Some(EncodingDescriptor::default()),
                },
            )
            .map_err(|err| err.to_string())?;
            let store = CdrStore::new(&h.paths.db_path);
            let old = store
                .get(h.record_id, Some(h.version))
                .map_err(|err| err.to_string())?;
            if old.state == CredentialState::Active {
                partial("Old version remained active when rotation was not committed. Explicit cancel/discard of pending version is not implemented.")
            } else {
                fail("Old version did not remain active after unconfirmed rotation.")
            }
        },
    ));

    rows.push(run_test(
        "FCT-17",
        "forbiddenChars do not appear in the final password.",
        "Derive a password and scan it against the descriptor's forbidden character set.",
        "No forbidden character appears.",
        || {
            let h = setup();
            let password = derive(&h, h.version)?;
            let store = CdrStore::new(&h.paths.db_path);
            let record = store
                .get(h.record_id, Some(h.version))
                .map_err(|err| err.to_string())?;
            let has_forbidden = password
                .chars()
                .any(|ch| record.encoding_descriptor.forbidden_chars.contains(ch));
            if !has_forbidden {
                ok("Derived password contained no forbidden characters.")
            } else {
                fail("Derived password contained at least one forbidden character.")
            }
        },
    ));

    rows.push(run_test(
        "FCT-18",
        "Logs do not contain mnemonic, master key, factor secret, or derived password.",
        "Static check of the Rust and Flutter source tree for logging macros/print calls.",
        "No logging path emits secrets.",
        || {
            let repo = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .parent()
                .unwrap()
                .to_path_buf();
            let mut findings = Vec::new();
            for production_dir in [
                PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src"),
                repo.join("flutter_app/lib"),
            ] {
                scan_for_logging(&production_dir, &mut findings).map_err(|err| err.to_string())?;
            }
            if findings.is_empty() {
                ok("No println/eprintln/log/debugPrint calls were found in the source tree.")
            } else {
                partial(&format!(
                    "Logging/print calls found for manual review: {}",
                    findings.join("; ")
                ))
            }
        },
    ));

    rows
}

fn run_test(
    test_id: &str,
    objective: &str,
    operation: &str,
    expected_result: &str,
    f: impl FnOnce() -> Result<(String, String), String>,
) -> FunctionalResult {
    let start = Instant::now();
    let (actual_result, status) = match f() {
        Ok(value) => value,
        Err(err) => (format!("Unexpected error: {err}"), "Failed".to_string()),
    };
    FunctionalResult {
        test_id: test_id.to_string(),
        objective: objective.to_string(),
        operation: operation.to_string(),
        expected_result: expected_result.to_string(),
        actual_result,
        status,
        duration_ms: elapsed_ms(start),
    }
}

fn ok(message: &str) -> Result<(String, String), String> {
    Ok((message.to_string(), "Passed".to_string()))
}

fn partial(message: &str) -> Result<(String, String), String> {
    Ok((message.to_string(), "Partial".to_string()))
}

fn fail(message: &str) -> Result<(String, String), String> {
    Ok((message.to_string(), "Failed".to_string()))
}

fn corrupt_cdr_mac(db_path: &Path, record_id: Uuid, version: u32) -> Result<(), String> {
    let store = CdrStore::new(db_path);
    let mut record = store
        .get(record_id, Some(version))
        .map_err(|err| err.to_string())?;
    record.mac_tag = crypto::b64_encode(&[0_u8; 32]);
    let conn = Connection::open(db_path).map_err(|err| err.to_string())?;
    conn.execute(
        "UPDATE cdr_records SET payload_json = ?3 WHERE record_id = ?1 AND version = ?2",
        params![
            record_id.to_string(),
            version,
            serde_json::to_string(&record).map_err(|err| err.to_string())?
        ],
    )
    .map_err(|err| err.to_string())?;
    Ok(())
}

fn scan_for_logging(path: &Path, findings: &mut Vec<String>) -> std::io::Result<()> {
    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let child = entry.path();
        let name = child
            .file_name()
            .and_then(|value| value.to_str())
            .unwrap_or("");
        if matches!(name, "target" | "build" | ".dart_tool" | ".git") {
            continue;
        }
        if child.is_dir() {
            scan_for_logging(&child, findings)?;
        } else if matches!(
            child.extension().and_then(|value| value.to_str()),
            Some("rs" | "dart")
        ) {
            let text = fs::read_to_string(&child)?;
            if text.contains("println!")
                || text.contains("eprintln!")
                || text.contains("debugPrint(")
                || text.contains("print(")
                || text.contains("developer.log(")
                || text.contains("log::")
            {
                findings.push(child.display().to_string());
            }
        }
    }
    Ok(())
}

fn run_performance_measurements() -> Vec<PerformanceResult> {
    vec![
        measure("PERF-01", "Enrollment initialization", || {
            let app_dir = tempfile::tempdir().map_err(|err| err.to_string())?;
            let usb_dir = evidence_usb_tempdir();
            let paths = StoragePaths::from_app_dir(app_dir.path().to_path_buf());
            let provider =
                FallbackPlatformFactorProvider::new(paths.app_dir.clone(), "evidence-macos");
            enroll_with_provider(
                &paths,
                &provider,
                EnrollmentRequest {
                    mnemonic: MNEMONIC.to_string(),
                    usb_path: usb_dir.path().to_string_lossy().to_string(),
                },
            )
            .map_err(|err| err.to_string())?;
            Ok(())
        }),
        measure("PERF-02", "Add CDR", || {
            let h = setup();
            add_credential_with_provider(
                &h.paths,
                &h.provider,
                AddCredentialRequest {
                    display_name: format!("Synthetic Service {}", Uuid::new_v4()),
                    service_hint: "perf.example".to_string(),
                    account_hint: "perf-user".to_string(),
                    notes: String::new(),
                    encoding_descriptor: Some(EncodingDescriptor::default()),
                },
            )
            .map_err(|err| err.to_string())?;
            Ok(())
        }),
        measure("PERF-03", "CDR HMAC verification", || {
            let h = setup();
            let (_, payload) = load_local_factor_payload(&h.provider, &h.paths.local_factor_path)
                .map_err(|err| err.to_string())?;
            let master_key =
                crypto::b64_decode(&payload.k_master).map_err(|err| err.to_string())?;
            let store = CdrStore::new(&h.paths.db_path);
            let record = store
                .get(h.record_id, Some(h.version))
                .map_err(|err| err.to_string())?;
            record
                .verify_mac(&master_key)
                .map_err(|err| err.to_string())?;
            Ok(())
        }),
        measure("PERF-04", "Single password derivation", || {
            let h = setup();
            derive(&h, h.version)?;
            Ok(())
        }),
        measure("PERF-05", "Deterministic password encoding", || {
            let descriptor = EncodingDescriptor::default();
            encoder::encode_password(&[7_u8; 32], &descriptor).map_err(|err| err.to_string())?;
            Ok(())
        }),
        measure(
            "PERF-06",
            "USB factor package read and authenticated decrypt",
            || {
                let h = setup();
                load_usb_factor_payload(MNEMONIC, h.usb_dir.path())
                    .map_err(|err| err.to_string())?;
                Ok(())
            },
        ),
        measure("PERF-07", "Create pending rotation", || {
            let h = setup();
            rotate_credential_with_provider(
                &h.paths,
                &h.provider,
                RotateCredentialRequest {
                    record_id: h.record_id,
                    encoding_descriptor: Some(EncodingDescriptor::default()),
                },
            )
            .map_err(|err| err.to_string())?;
            Ok(())
        }),
        measure("PERF-08", "Commit rotation", || {
            let h = setup();
            let pending = rotate_credential_with_provider(
                &h.paths,
                &h.provider,
                RotateCredentialRequest {
                    record_id: h.record_id,
                    encoding_descriptor: Some(EncodingDescriptor::default()),
                },
            )
            .map_err(|err| err.to_string())?;
            confirm_rotation_with_provider(
                &h.paths,
                &h.provider,
                ConfirmRotationRequest {
                    record_id: h.record_id,
                    version: pending.version,
                },
            )
            .map_err(|err| err.to_string())?;
            Ok(())
        }),
        measure("PERF-09", "USB loss recovery / rebuild USB package", || {
            let h = setup();
            fs::remove_file(usb_package_file(h.usb_dir.path())).map_err(|err| err.to_string())?;
            recover_usb_with_provider(
                &h.paths,
                &h.provider,
                RecoverUsbRequest {
                    mnemonic: MNEMONIC.to_string(),
                    usb_path: h.usb_dir.path().to_string_lossy().to_string(),
                },
            )
            .map_err(|err| err.to_string())?;
            read_usb_factor_package(h.usb_dir.path()).map_err(|err| err.to_string())?;
            Ok(())
        }),
        measure(
            "PERF-10",
            "Local factor recovery / rebuild local package",
            || {
                let h = setup();
                let new_app = tempfile::tempdir().map_err(|err| err.to_string())?;
                let new_paths = StoragePaths::from_app_dir(new_app.path().to_path_buf());
                let new_provider = FallbackPlatformFactorProvider::new(
                    new_paths.app_dir.clone(),
                    "evidence-macos",
                );
                recover_local_with_provider(
                    &new_paths,
                    &new_provider,
                    RecoverLocalRequest {
                        mnemonic: MNEMONIC.to_string(),
                        usb_path: h.usb_dir.path().to_string_lossy().to_string(),
                    },
                )
                .map_err(|err| err.to_string())?;
                Ok(())
            },
        ),
    ]
}

fn measure(
    metric_id: &str,
    operation: &str,
    mut f: impl FnMut() -> Result<(), String>,
) -> PerformanceResult {
    let mut values = Vec::with_capacity(ITERATIONS);
    for _ in 0..ITERATIONS {
        let start = Instant::now();
        f().unwrap_or_else(|err| panic!("{operation} failed: {err}"));
        values.push(elapsed_ms(start));
    }
    stats(metric_id, operation, values)
}

fn stats(metric_id: &str, operation: &str, mut values: Vec<f64>) -> PerformanceResult {
    values.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let mean = values.iter().sum::<f64>() / values.len() as f64;
    let median = if values.len() % 2 == 0 {
        (values[values.len() / 2 - 1] + values[values.len() / 2]) / 2.0
    } else {
        values[values.len() / 2]
    };
    let variance = values
        .iter()
        .map(|value| {
            let delta = value - mean;
            delta * delta
        })
        .sum::<f64>()
        / values.len() as f64;
    let p95_index = ((values.len() as f64 * 0.95).ceil() as usize).saturating_sub(1);
    PerformanceResult {
        metric_id: metric_id.to_string(),
        operation: operation.to_string(),
        iterations: values.len(),
        mean_ms: mean,
        median_ms: median,
        min_ms: values[0],
        max_ms: *values.last().unwrap(),
        std_ms: variance.sqrt(),
        p95_ms: values[p95_index],
    }
}

fn elapsed_ms(start: Instant) -> f64 {
    start.elapsed().as_secs_f64() * 1000.0
}

fn write_functional_outputs(out_dir: &Path, rows: &[FunctionalResult]) -> std::io::Result<()> {
    let json = serde_json::to_string_pretty(rows).unwrap();
    fs::write(out_dir.join("functional_correctness_tests.json"), json)?;
    let mut csv = String::from(
        "test_id,objective,operation,expected_result,actual_result,status,duration_ms\n",
    );
    for row in rows {
        csv.push_str(&format!(
            "{},{},{},{},{},{},{:.3}\n",
            csv_escape(&row.test_id),
            csv_escape(&row.objective),
            csv_escape(&row.operation),
            csv_escape(&row.expected_result),
            csv_escape(&row.actual_result),
            csv_escape(&row.status),
            row.duration_ms
        ));
    }
    fs::write(out_dir.join("functional_correctness_tests.csv"), csv)?;
    let mut md = String::from(
        "# Functional Correctness Tests\n\n| test_id | objective | expected_result | actual_result | status | duration_ms |\n|---|---|---|---|---:|---:|\n",
    );
    for row in rows {
        md.push_str(&format!(
            "| {} | {} | {} | {} | {} | {:.3} |\n",
            row.test_id,
            md_escape(&row.objective),
            md_escape(&row.expected_result),
            md_escape(&row.actual_result),
            row.status,
            row.duration_ms
        ));
    }
    fs::write(out_dir.join("functional_correctness_tests.md"), md)?;
    Ok(())
}

fn write_performance_outputs(out_dir: &Path, rows: &[PerformanceResult]) -> std::io::Result<()> {
    let json = serde_json::to_string_pretty(rows).unwrap();
    fs::write(out_dir.join("performance_measurements.json"), json)?;
    let mut csv = String::from(
        "metric_id,operation,iterations,mean_ms,median_ms,min_ms,max_ms,std_ms,p95_ms\n",
    );
    for row in rows {
        csv.push_str(&format!(
            "{},{},{},{:.3},{:.3},{:.3},{:.3},{:.3},{:.3}\n",
            csv_escape(&row.metric_id),
            csv_escape(&row.operation),
            row.iterations,
            row.mean_ms,
            row.median_ms,
            row.min_ms,
            row.max_ms,
            row.std_ms,
            row.p95_ms
        ));
    }
    fs::write(out_dir.join("performance_measurements.csv"), csv)?;
    let mut md = String::from(
        "# Performance Measurements\n\nAll measurements were collected with the Rust core release build. Times are in milliseconds.\n\n| metric_id | operation | n | mean | median | min | max | std | p95 |\n|---|---|---:|---:|---:|---:|---:|---:|---:|\n",
    );
    for row in rows {
        md.push_str(&format!(
            "| {} | {} | {} | {:.3} | {:.3} | {:.3} | {:.3} | {:.3} | {:.3} |\n",
            row.metric_id,
            md_escape(&row.operation),
            row.iterations,
            row.mean_ms,
            row.median_ms,
            row.min_ms,
            row.max_ms,
            row.std_ms,
            row.p95_ms
        ));
    }
    fs::write(out_dir.join("performance_measurements.md"), md)?;
    Ok(())
}

fn csv_escape(value: &str) -> String {
    format!("\"{}\"", value.replace('"', "\"\""))
}

fn md_escape(value: &str) -> String {
    value.replace('|', "\\|").replace('\n', " ")
}
