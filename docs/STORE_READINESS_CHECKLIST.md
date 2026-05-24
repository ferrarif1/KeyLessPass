# Store Readiness Checklist

## macOS

### Completed

- App name set to KeyLessPass.
- Bundle identifier set to `com.keylesspass.desktop`.
- Release entitlements include removable-media access and user-selected read/write files.
- Packaging script builds Rust Core, builds Flutter macOS release, copies the Rust dynamic library, signs locally, and can create a DMG.
- Privacy, security, release, and support placeholder documents are present.
- Native macOS USB folder selection is implemented without third-party plugins.
- Release entitlement check has been validated for removable media and user-selected read/write access.

### Partially Complete

- App icon assets exist, but final App Store/DMG icon review is still required.
- Sandbox compatibility is prepared for removable media, but needs notarized distribution testing.
- Reset application data remains disabled until a production confirmation and backup flow is implemented.

### Needs Manual Configuration

- Apple Developer Team ID.
- Developer ID Application certificate.
- Notarization credentials.
- Public privacy-policy URL.
- Support email.
- Website or product page.

## Windows

### Completed

- Rust Core has a Windows provider abstraction and smoke tests through the unified trait.
- Windows packaging script builds Flutter Windows release and copies `keylesspass_core.dll`.

### Partially Complete

- DPAPI production validation is prepared at the architecture level.
- Installer tooling is documented but not finalized.

### Needs Manual Configuration

- Code signing certificate.
- MSI/EXE installer configuration.
- Windows 10/11 physical-machine validation.
- Installer upgrade/uninstall testing.

## Linux / UOS / Kylin

### Completed

- Linux provider abstraction is present and covered by trait smoke tests.
- Linux packaging script builds Flutter Linux release and copies the Rust shared library.
- Architecture avoids mandatory cloud or browser dependencies.

### Partially Complete

- Local AEAD/file-permission provider is the first supported path.
- Secret Service/libsecret integration is reserved for a later hardening pass.

### Needs Manual Configuration

- deb/rpm/AppImage packaging.
- Desktop entry validation.
- Distribution-specific QA on Ubuntu, Debian, UOS, and Kylin.
- Offline installer packaging for enterprise environments.
