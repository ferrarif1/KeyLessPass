# Productization Report

## Summary

KeyLessPass was upgraded from an early desktop build into a product-oriented Flutter Desktop + Rust Core application, with macOS as the primary validation target and Windows/Linux interfaces preserved.

## Completed Changes

- Reworked the Flutter shell into a desktop product layout with Dashboard, Setup, Records, USB Device, Security, Settings, and About sections. Add, derive, and rotation flows are entered from Records; recovery tools are entered from USB Device.
- Added search and status filtering for credential records.
- Added editable metadata fields, including notes, without changing the derivation path.
- Simplified rotation so creating a new current version keeps the previous version derivable for rollback checks.
- Added macOS USB candidate detection for writable removable volumes even before a USB factor package exists.
- Added a macOS native directory picker through `NSOpenPanel` to grant user-selected read/write access to removable USB volumes without adding a CocoaPods plugin.
- Upgraded the USB Device page from read-only status display to device management: choose path, verify USB package structure/integrity without a mnemonic, and rebuild USB package with mnemonic + this computer.
- Expanded Add Record with password-rule controls for required character classes and forbidden characters.
- Added local English and Simplified Chinese mnemonic generation during setup, with generated phrases kept in memory only.
- Added paper-aligned 2-of-3 pairwise-wrapper recovery using `W_MC`, `W_MU`, and `W_CU`.
- Added mnemonic reset using this computer plus the paired USB package through `W_CU`. The operation does not require the old mnemonic, refreshes mnemonic salt/verifier and wrappers, and does not change existing derived service passwords.
- Added schema/package version fields to factor packages and schema version to recovery metadata while keeping legacy MAC verification compatibility.
- Added mnemonic recovery verification for new enrollments so mnemonic + local or mnemonic + USB recovery rejects an incorrect mnemonic instead of creating a different derivation set.
- Added a USB CDR metadata backup file with HMAC verification, local/USB consistency detection, sync local to USB, and restore local from USB.
- Added typed-confirmation local data reset in Settings so a user can intentionally return the app to setup.
- Replaced diagnostics placeholder with a redacted diagnostics dialog and copy action.
- Blocked ordinary re-enrollment at the Rust Core level when local state already exists.
- Added English and Simplified Chinese ARB resources and generated Flutter localizations.
- Added i18n tests for resource completeness and generated labels.
- Updated macOS bundle metadata to `KeyLessPass` and `com.keylesspass.desktop`.
- Added product documentation: privacy, security, release, contributing, development, changelog, and readiness checklist.

## Removed Or Replaced Product-Inappropriate Copy

- Replaced academic and development-stage language in the primary README.
- Replaced screenshot test labels that used development-style sample names with anonymous enterprise-style sample names.
- Moved technical explanation out of primary product flows and kept user-facing UI copy short.
- Removed default Flutter widget-test instructional comments.

## i18n Coverage

- English and Simplified Chinese resources are present in `flutter_app/lib/l10n/`.
- Core navigation, buttons, settings, recovery, USB, security, and error/success text are localized.
- `flutter_app/test/i18n_test.dart` verifies key parity and generated localization loading.

## UI Adjustments

- Left-side desktop navigation is now scroll-safe and avoids small-window overflow.
- Dashboard shows record count, USB status, integrity status, and quick actions.
- Records view supports search, filtering, details, derive, rotate, and metadata editing.
- Derive view masks passwords by default, clears mnemonic input after derivation attempts, and keeps normal derivation scoped to mnemonic + this computer. USB is reserved for setup, recovery, and factor replacement.
- Settings exposes language, theme mode, clipboard timeout, default password length, advanced mode, diagnostics, and typed-confirmation local reset.
- USB Device exposes USB package structural verification, USB factor rebuild, CDR backup status, sync local to USB, restore local from USB, three 2-of-3 recovery paths, and mnemonic reset with confirmation.

## Security Hardening

- UI avoids displaying internal factor secrets, master keys, raw CDR secrets, and derived password history.
- Derived passwords are masked by default and copied to clipboard with automatic clearing.
- Rust Core tests cover derivation stability, metadata immutability, path-field sensitivity, wrapper and package tamper failure, missing factors, platform provider trait smoke tests, and current/previous version rotation behavior.
- Rust Core tests also cover factor/recovery schema fields, USB package structural verification, and two-factor recovery behavior.
- Rust Core tests also cover USB CDR backup sync/restore and mnemonic reset without changing derived passwords.
- Local and USB factor payloads no longer persist plaintext `Kmaster`; the local payload does not store `usbSecret`, and the USB payload does not store `deviceSecret`.
- V2 `encryptedPayload` is retained as a historical schema field name for base64 encoded factor payloads, not for a mnemonic-encrypted USB vault.
- USB discovery no longer logs mount details.

## Release Preparation

- macOS entitlements include removable media and user-selected read/write file access.
- macOS build metadata is productized.
- macOS packaging script supports ad-hoc or Developer ID signing with Release entitlements and optional DMG creation.
- Windows/Linux packaging entry points remain available for later hardening.

## Pending Work

- Real Apple Developer ID signing, notarization, and a public privacy-policy URL.
- Windows/Linux native folder picker parity for USB path selection.
- Full light-mode visual pass; the structure exists, while dark mode is the polished default.
- Windows DPAPI validation on physical Windows 10/11 machines.
- Linux Secret Service/libsecret option and UOS/Kylin packaging validation.
- Real App Store-style onboarding/help copy for the new CDR backup choices.
