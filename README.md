# KeyLessPass

KeyLessPass is a local-only desktop client for deriving enterprise passwords on demand. It is designed for internal legacy systems, operations consoles, vendor portals, database gateways, and other environments that still require text passwords.

KeyLessPass is not a web app, not a browser extension, and not a cloud password manager. It does not store target-system plaintext passwords, does not maintain an encrypted service-password vault, and does not store the mnemonic phrase.

## Key Features

- Native desktop client built with Flutter Desktop and a Rust security core.
- Local SQLite CDR metadata storage with integrity protection.
- Ordinary removable USB drive support for the USB factor package.
- Random per-user master key generated during enrollment.
- Local English and Simplified Chinese mnemonic generation; generated phrases are not stored.
- Password derivation based on `recordSeq`, stable `recordId`, `version`, `salt`, and `encodingDescriptor`.
- Display metadata such as name, service hint, and account hint can be edited without changing the derived password.
- Two-phase rotation: create pending version, derive and update the target system, then commit or cancel.
- Local recovery flows for rebuilding missing USB or local factor packages with two available factors.
- USB device management for path selection, package verification, and USB package rebuild.
- Redacted diagnostics export for support without secrets.
- macOS, Windows, and Linux architecture with platform factor provider abstraction.
- English and Simplified Chinese UI resources.

## How It Works

During enrollment, KeyLessPass generates a random 256-bit master key. The mnemonic phrase can be entered manually or generated locally in English or Simplified Chinese. It is used only as one recovery/derivation factor and is not the root seed for service passwords. The local platform factor, USB factor, and mnemonic-derived factor are combined through HKDF to derive a service-specific secret.

For each credential record, only non-secret CDR metadata is stored locally. The actual service password is generated on demand, encoded deterministically according to the record's password rule, and then cleared from the UI/clipboard after a short timeout.

Changing `displayName`, `serviceHint`, `accountHint`, or notes does not change the derived password. Changing password rules requires a new CDR version and is treated as rotation.

## Security Model

- No target-system plaintext password is written to disk.
- No encrypted service-password vault is maintained.
- No mnemonic phrase is stored.
- No network sync, cloud account, browser autofill, or remote backend is included.
- CDR and factor packages are integrity checked before use.
- Sensitive values such as mnemonic text, master key, factor secrets, HKDF output, and derived passwords must not be logged.
- Clipboard clearing is enabled by default and configurable in settings.

Client-only rollback detection is limited to local/USB metadata comparisons and integrity checks. Full protection against coordinated rollback of all local copies requires an external trusted state or append-only audit integration.

## Install / Build

### Prerequisites

- Flutter Desktop SDK
- Rust toolchain
- macOS: Xcode for macOS desktop builds
- Windows: Visual Studio build tools for Windows desktop builds
- Linux: Flutter Linux desktop dependencies

### Build Rust Core

```bash
cd rust_core
cargo build
cargo test
```

### Build Flutter Desktop

```bash
cd flutter_app
flutter pub get
flutter analyze
flutter test
flutter run -d macos
```

On macOS systems where `xcode-select` points to Command Line Tools, use:

```bash
DEVELOPER_DIR=/Applications/Xcode.app/Contents/Developer flutter build macos --release
```

### Release Builds

```bash
packaging/macos/build_dmg.sh
packaging/linux/build_packages.sh
powershell -ExecutionPolicy Bypass -File packaging/windows/build_installer.ps1
```

See [RELEASE.md](RELEASE.md) for signing, notarization, and packaging details.

## Internationalization

UI text is sourced from Flutter ARB files:

- `flutter_app/lib/l10n/app_en.arb`
- `flutter_app/lib/l10n/app_zh.arb`

The app follows the system language by default and provides manual English / Simplified Chinese selection in Settings. Resource completeness is checked by `flutter_app/test/i18n_test.dart`.

## Privacy

KeyLessPass is local-only by default. It does not upload passwords, mnemonic phrases, factor secrets, CDR records, or usage analytics. See [PRIVACY.md](PRIVACY.md).

## Current Status

The macOS desktop path is the primary tested target. Rust core tests cover derivation stability, metadata immutability boundaries, CDR/USB tamper failures, rotation, and platform provider abstraction. Windows and Linux code paths are structured for build and packaging follow-up.

## Roadmap

- macOS Developer ID signing and notarized DMG.
- Windows DPAPI hardening and MSI installer.
- Linux Secret Service/libsecret option and deb/rpm/AppImage packaging.
- External optional version digest or append-only audit integration for stronger rollback detection.
- Enterprise diagnostics export with strict sensitive-data redaction.

## Documentation

- [SECURITY.md](SECURITY.md)
- [PRIVACY.md](PRIVACY.md)
- [RELEASE.md](RELEASE.md)
- [CONTRIBUTING.md](CONTRIBUTING.md)
- [DEVELOPMENT.md](DEVELOPMENT.md)
- [docs/PRODUCTIZATION_REPORT.md](docs/PRODUCTIZATION_REPORT.md)
- [docs/STORE_READINESS_CHECKLIST.md](docs/STORE_READINESS_CHECKLIST.md)
