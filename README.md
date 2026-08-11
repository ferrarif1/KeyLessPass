# KeyLessPass

<p align="center">
  <img src="docs/readme-assets/logo.png" width="112" alt="KeyLessPass logo" />
</p>

<p align="center">
  <strong>Service-password derivation and lifecycle management for legacy enterprise systems.</strong>
</p>

<p align="center">
  <a href="README.zh-CN.md">简体中文</a>
  ·
  <a href="SECURITY.md">Security</a>
  ·
  <a href="PRIVACY.md">Privacy</a>
  ·
  <a href="RELEASE.md">Release</a>
  ·
  <a href="DOCS.md">Docs</a>
</p>

<p align="center">
  <img alt="Local only" src="https://img.shields.io/badge/local--only-by%20default-101010">
  <img alt="Desktop" src="https://img.shields.io/badge/desktop-macOS%20%7C%20Windows%20%7C%20Linux-101010">
  <img alt="Core" src="https://img.shields.io/badge/core-Rust-101010">
  <img alt="UI" src="https://img.shields.io/badge/UI-Flutter%20Desktop-101010">
</p>

<p align="center">
  <a href="https://github.com/ferrarif1/KeyLessPass/releases/tag/v1.0-jisa-2026">
    <img alt="Download macOS DMG" src="https://img.shields.io/badge/Download-macOS%20DMG-0A84FF?style=for-the-badge&logo=apple&logoColor=white">
  </a>
  <a href="https://github.com/ferrarif1/KeyLessPass/releases/tag/v1.0-jisa-2026">
    <img alt="Download Windows installer" src="https://img.shields.io/badge/Download-Windows%20Installer-0078D4?style=for-the-badge&logo=windows11&logoColor=white">
  </a>
</p>

<p align="center">
  <sub>Open the release page and choose the macOS or Windows client asset for your device.</sub>
</p>

KeyLessPass is a native desktop client for deriving text passwords on demand for internal systems that still depend on legacy password login. It is intended for operations consoles, vendor portals, database gateways, network appliances, and small enterprise environments where a local-only security posture is required.

It is not a web app, browser extension, cloud password manager, or password vault. KeyLessPass does not store target-system plaintext passwords or maintain an encrypted service-password database. Its v3 paper recovery share is an offline encoding of a random high-entropy Shamir share; it is not intended to be memorized and is not persisted by the application.

## Product Screenshots

| Setup | Records |
| --- | --- |
| ![Setup](docs/readme-assets/screenshots/01-enrollment.png) | ![Records](docs/readme-assets/screenshots/02-records.png) |

| Derive Password | Rotation |
| --- | --- |
| ![Derive Password](docs/readme-assets/screenshots/03-derive-password.png) | ![Rotation](docs/readme-assets/screenshots/04-rotation.png) |

| USB Device and Recovery |
| --- |
| ![USB Device and Recovery](docs/readme-assets/screenshots/05-usb-recovery.png) |

## Why KeyLessPass

Traditional password managers protect a vault of stored secrets. KeyLessPass takes a different path: it stores only protected local state, USB factor material, Credential Description Records, and recovery metadata. The service password is derived only when the user provides the required local factors.

This makes it useful when an organization must keep using legacy password-based systems but wants to avoid accumulating a recoverable password vault on endpoints.

## Core Features

- Native Flutter Desktop UI with a Rust security core.
- Local SQLite storage for non-secret CDR metadata and integrity tags.
- Ordinary removable USB drive support for USB factor packages.
- CDR metadata backup on the USB drive, with local/USB consistency checks and explicit sync/restore choices.
- Random per-user master key generated during enrollment.
- Mature Shamir 2-of-3 Root-Key sharing through `vsss-rs`, with authenticated, vault/version-bound share envelopes.
- A checksum-protected paper recovery share (currently rendered as 108 words), a platform-protected managed-computer share, and an ordinary copyable USB share.
- Share-set refresh and recovery/computer/USB factor replacement with generation-based stale-share rejection.
- Verified migration from legacy v2 pairwise complete-key wrappers to v3 shares without changing the Root Key or derived service passwords.
- Service derivation based on stable `recordSeq`, `recordId`, `version`, `salt`, and `encodingDescriptor`.
- Selectable service derivation algorithm for new profiles: HKDF-SHA256, Argon2id, scrypt, or PBKDF2-HMAC-SHA256.
- Editable display metadata that does not change derived passwords.
- Evidence-bounded password rotation with authenticated old/new probes and explicit atomic-replacement, overlap-then-revoke, and opaque-target contracts.
- Local recovery workflows for rebuilding missing USB or local factor packages.
- USB management for path selection, package verification, and package rebuild.
- Redacted diagnostics export.
- Cross-platform provider abstraction for macOS, Windows, Linux, and fallback secure storage.
- English and Simplified Chinese UI resources.

## Factor-Preserving Peer Recovery Research

The optional `peer-recovery` Rust feature contains a separate research
prototype that replaces the paper share with an encrypted network share while
preserving the Root-Key 2-of-3 factor boundary. It Shamir-splits the canonical
network share envelope into 3-of-5 node fragments and requires two independent
Ed25519 approvals before session-bound X25519/AES-GCM release. It uses no View
Key, Data Key, OPRF, or opaque-object scan. It is not enabled in the desktop
product path and does not claim a production recovery transport or Byzantine
node tolerance. See
[`docs/research/FACTOR_PRESERVING_PEER_RECOVERY.zh-CN.md`](docs/research/FACTOR_PRESERVING_PEER_RECOVERY.zh-CN.md).

## ASTER Research Artifact

The optional `research` feature also contains ASTER's authorization-scoped
exact-domain evaluator and failure-safe Root-Epoch migration model. The normal
desktop product path remains local and does not enable this backend. The
repository keeps the implementation, experiment harnesses, recorded raw
results, TLA+ models, and reproducibility scripts under version control; paper
manuscripts, rendered figures, and submission bundles are intentionally
excluded through `.gitignore`.

```bash
cd rust_core
cargo test --all-targets --all-features
cd ..
./research/aster/scripts/reproduce_all.sh --quick
```

See [`research/aster/README.md`](research/aster/README.md) for the evidence
layers, full reproduction command, measured-result boundary, and expensive MPC
steps.

## What Is Stored

| Stored | Not stored |
| --- | --- |
| Platform-protected managed-computer share envelope and committed recovery manifest | Target-system plaintext passwords |
| USB share envelope, committed recovery manifest, and optional CDR replica | Encrypted service-password vault |
| Canonical versioned CDR metadata, salts, state, replica metadata, and MAC tags | Plaintext Root Key in any persisted v3 object |
| Legacy v2 pairwise wrappers only until an explicit verified migration archives them | Paper recovery share representation (the application displays but does not persist it) |

## How It Works

In the selectable v3 recovery schema, enrollment or migration starts with a random 256-bit Root Key and uses the external `vsss-rs` finite-field implementation to split it into three shares at threshold two. The dependency currently describes itself as under audit; KeyLessPass does not claim an independent audit result:

```text
K_root <- Random(256 bits)
(S_recovery, S_computer, S_usb) <- ShamirSplit(2, 3, K_root)
K_purpose <- HKDF(K_root, vaultID || rootGeneration || suite || purposeLabel)
```

Every share envelope binds the vault, Root-Key generation, share-set ID, factor type/ID/generation, threshold, suite, encoding version, and creation time. A Root-Key-derived HMAC authenticates that metadata after reconstruction, and a key-confirmation value rejects the wrong recovered key. Shamir itself is not claimed to authenticate shares, revoke factors, or prevent rollback; those properties come from envelopes, committed manifests, generation changes, and an optional freshness anchor.

Legacy v2 profiles retain a read-only pairwise-wrapper path solely for compatibility and verified migration. New v3 recovery takes precedence once its manifest is committed. The current Flutter enrollment screens still create v2 data, so selecting v3 currently requires the Rust migration API; this is an explicit prototype limitation.

For compatibility, existing profiles without an algorithm field are treated as legacy HKDF-SHA256. New profiles can choose HKDF-SHA256, Argon2id, scrypt, or PBKDF2-HMAC-SHA256 before enrollment; the choice is locked for that local profile and can be changed only after resetting local application data and initializing again.

For each credential record, KeyLessPass stores only non-secret CDR metadata. Display fields such as name, service hint, account hint, and notes are searchable and editable, but they are not part of the derivation path. Password rule changes create a new CDR version and are treated as rotation.

When a paired USB drive is present, KeyLessPass can write a signed CDR metadata backup to the USB drive. On refresh or insertion detection, the app compares local CDR metadata with the USB backup and prompts the user to either sync local records to USB or restore local records from the USB backup.

```mermaid
flowchart LR
    M["Paper recovery share<br/>offline, high entropy"] --> R["Shamir 2-of-3<br/>same-set shares"]
    FC["Computer share<br/>platform protected"] --> R
    FU["USB share<br/>ordinary copyable file"] --> R
    R --> KM["Recovered Root Key<br/>transient memory"]
    KM --> D["Selected KDF + deterministic encoding"]
    C["CDR stable fields<br/>recordSeq + recordId + version + salt + Rule"] --> D
    D --> P["Service password<br/>shown briefly / clipboard timeout"]
    FU --> U["USB stores<br/>USB factor package<br/>optional CDR replica<br/"]
    C --> U
```

## Security Model

- No target-system plaintext password is written to disk.
- No encrypted service-password vault is maintained.
- The v3 paper recovery share representation is not stored by the application.
- The Root Key is not persisted in any v3 local or USB payload.
- The USB package is an ordinary copyable factor container, not an uncopyable hardware key.
- Any two valid shares from the committed vault/share set/generation can recover the Root Key; the recovery API rejects a single share.
- No cloud sync, remote backend, browser autofill, or account login is included.
- All random values come from the operating system CSPRNG.
- CDR and factor packages are integrity checked before use.
- USB CDR backups are MAC-protected and contain metadata only, never service passwords.
- Derived passwords are masked by default and cleared from the clipboard after a configurable timeout.
- Sensitive values such as mnemonic text, master key, factor secrets, raw HKDF output, AEAD keys, HMAC keys, and derived passwords must not be logged.

Client-only rollback detection is limited to partial-copy inconsistency. Enterprise-anchored mode exposes a minimal compare-and-set freshness service for the latest generation/epoch/digest; no production remote service is shipped.

## Desktop Navigation

The product UI is organized around stable desktop modules:

- Dashboard
- Setup
- Records
- USB Device
- Security
- Settings
- About

Record-centric actions such as add, derive, and rotation are launched from Records. Recovery tools, USB factor verification, CDR backup sync, and local restore from USB are grouped under USB Device.

## Architecture

```text
KeyLessPass
├── flutter_app/          # Flutter Desktop UI
├── rust_core/            # Rust cryptography, storage, recovery, and FFI core
├── packaging/            # macOS, Windows, and Linux packaging scripts
├── experiments/          # Reproducible harness inputs and recorded results
├── artifact/             # Machine-readable EPSCD result package
├── formal/, models/, tla/ # TLA+ specifications and checked configurations
├── research/aster/       # ASTER implementation, adapters, results, and scripts
├── docs/                 # Product, security, reproducibility, and design docs
└── releases/             # Local release artifacts, ignored by git
```

Manuscript sources, rendered paper output, and journal submission packages are
kept outside the versioned software/artifact boundary and are ignored by git.

The Rust core is intentionally independent from platform-specific secure storage details. Platform factor providers implement a common interface, with macOS Keychain, Windows DPAPI, Linux local/fallback storage, and future TPM/Secure Enclave hooks isolated behind the provider layer.

## Quick Start

### Prerequisites

- Flutter Desktop SDK
- Rust toolchain
- macOS: Xcode for desktop builds. See [docs/MACOS_INSTALL.en.md](docs/MACOS_INSTALL.en.md) / [中文](docs/MACOS_INSTALL.md).
- Windows: Visual Studio Build Tools for desktop builds. See [docs/WINDOWS_INSTALL.en.md](docs/WINDOWS_INSTALL.en.md) / [中文](docs/WINDOWS_INSTALL.md).
- Linux: Flutter Linux desktop dependencies. See [docs/LINUX_INSTALL.en.md](docs/LINUX_INSTALL.en.md) / [中文](docs/LINUX_INSTALL.md).

Each platform guide starts from Flutter installation and continues through Rust, local run, release build, and packaging notes.

### Build and Test the Core

```bash
cd rust_core
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-targets --all-features
```

### Run the Desktop App

```bash
cd flutter_app
flutter pub get
flutter analyze
flutter test
flutter run -d macos
```

### macOS Release Build

```bash
DEVELOPER_DIR=/Applications/Xcode.app/Contents/Developer \
FLUTTER_BIN=/path/to/flutter \
CODESIGN_IDENTITY='-' \
packaging/macos/build_dmg.sh
```

The local unsigned app bundle is produced under:

```text
flutter_app/build/macos/Build/Products/Release/KeyLessPass.app
```

For distribution, sign with a Developer ID Application certificate and notarize the DMG. See [RELEASE.md](RELEASE.md).

## Internationalization

UI strings are sourced from Flutter ARB resources:

- `flutter_app/lib/l10n/app_en.arb`
- `flutter_app/lib/l10n/app_zh.arb`

The app follows the system language by default and supports manual English / Simplified Chinese selection in Settings.

## Current Status

macOS is the primary tested desktop target. The architecture and packaging scripts reserve Windows and Linux support, including platform factor provider separation for future hardening.

The Rust test suite covers derivation stability, metadata immutability boundaries, path-field sensitivity, tamper failures, missing factors, rotation behavior, recovery behavior, USB CDR backup sync/restore, mnemonic reset, and platform provider trait tests. Flutter tests cover UI construction, navigation, language switching, and i18n resource completeness.

## Roadmap

- Developer ID signing and notarized macOS DMG.
- Windows DPAPI hardening and MSI packaging validation.
- Linux Secret Service/libsecret option and deb/rpm/AppImage packaging validation.
- Optional external version digest or append-only audit integration.
- Enterprise diagnostics export with stricter redaction review.

## Documentation

See the full bilingual documentation map: [DOCS.md](DOCS.md) / [中文](DOCS.zh-CN.md).

| Goal | Read |
| --- | --- |
| Run the desktop client | [DEVELOPMENT.md](DEVELOPMENT.md) |
| Build on macOS / Windows / Linux | [macOS](docs/MACOS_INSTALL.en.md), [Windows](docs/WINDOWS_INSTALL.en.md), [Linux](docs/LINUX_INSTALL.en.md) |
| Prepare a release | [RELEASE.md](RELEASE.md) |
| Review security and privacy | [SECURITY.md](SECURITY.md), [PRIVACY.md](PRIVACY.md) |
