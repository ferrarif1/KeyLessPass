# Development

## Repository Layout

- `flutter_app/`: Flutter Desktop UI and FFI bindings.
- `rust_core/`: Rust cryptography, CDR storage, factor packages, recovery, and JSON FFI.
- `packaging/`: macOS, Windows, and Linux packaging entry points.
- `docs/`: architecture, productization, and readiness notes.

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

## USB Testing On macOS

Use a writable removable volume such as `/Volumes/WD`. KeyLessPass writes `keylesspass-usb-factor.json` at the volume root. Full Disk Access is not expected for normal removable media access when the app has removable-media entitlements.

If automatic scanning sees the volume but reports limited write access, use the
folder button next to the USB path field and choose the USB root directory. The
macOS runner uses `NSOpenPanel` and `com.apple.security.files.user-selected.read-write`
to grant the running app access to that path.
