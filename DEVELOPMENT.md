# Development

## Repository Layout

- `flutter_app/`: Flutter Desktop UI and FFI bindings.
- `rust_core/`: Rust cryptography, CDR storage, factor packages, recovery, and JSON FFI.
- `packaging/`: macOS, Windows, and Linux packaging entry points.
- `docs/`: architecture, productization, and readiness notes.
- `research/aster/`: ASTER specifications, executable experiments, formal models, and result provenance. Manuscripts and journal-delivery files are intentionally ignored.

## Local Setup

```bash
cd rust_core
cargo build

cd ../flutter_app
flutter pub get
flutter run -d macos
```

If macOS builds fail because Xcode Command Line Tools are selected:

```bash
DEVELOPER_DIR=/Applications/Xcode.app/Contents/Developer flutter run -d macos
```

## i18n

Edit ARB files in `flutter_app/lib/l10n/`, then regenerate:

```bash
cd flutter_app
flutter gen-l10n
flutter test test/i18n_test.dart
```

## ASTER-Aligned Core Checks

The desktop product uses exact-policy-space v3 for newly created credentials. Run the complete Rust and Flutter checks with:

```bash
cd rust_core
cargo test
cargo check --all-targets --all-features

cd ../flutter_app
flutter gen-l10n
flutter analyze
flutter test
```

The ASTER semantic research profile and its bounded evidence suite are separate from the local desktop compatibility profile:

```bash
cd ..
./research/aster/scripts/reproduce_all.sh --quick
```

The semantic evaluator is process-local and must not be described as a production threshold deployment. The checked-in MP-SPDZ experiment establishes fixed-circuit feasibility only. See `docs/ASTER_IMPLEMENTATION_PROFILE.md` and `research/aster/LIMITATIONS.md`.

## USB Testing On macOS

Use a writable removable volume such as `/Volumes/WD`. KeyLessPass writes `keylesspass-usb-factor.json` at the volume root. Full Disk Access is not expected for normal removable media access when the app has removable-media entitlements.

If automatic scanning sees the volume but reports limited write access, use the
folder button next to the USB path field and choose the USB root directory. The
macOS runner uses `NSOpenPanel` and `com.apple.security.files.user-selected.read-write`
to grant the running app access to that path.
