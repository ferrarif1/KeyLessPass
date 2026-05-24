# Release Guide

## macOS

Build the Rust core and Flutter app:

```bash
cd rust_core
cargo build --release

cd ../flutter_app
DEVELOPER_DIR=/Applications/Xcode.app/Contents/Developer flutter build macos --release
```

The release app is expected at:

```text
flutter_app/build/macos/Build/Products/Release/KeyLessPass.app
```

The packaging script copies and signs the Rust dynamic library:

```bash
CODESIGN_IDENTITY="Developer ID Application: Example Team" packaging/macos/build_dmg.sh
```

For local unsigned validation:

```bash
DEVELOPER_DIR=/Applications/Xcode.app/Contents/Developer \
FLUTTER_BIN=/Users/zhangyuanyi/development/flutter/bin/flutter \
CODESIGN_IDENTITY="-" CREATE_DMG=1 packaging/macos/build_dmg.sh
```

Do not re-sign the `.app` without entitlements. Re-signing without
`macos/Runner/Release.entitlements` removes removable-media and user-selected
read/write access, which will break USB factor creation and recovery.

Distribution checklist:

- Configure the real Apple Developer Team ID.
- Use a real Developer ID Application certificate.
- Verify entitlements for removable media and user-selected read/write files.
- Create a DMG.
- Submit the signed app or DMG for notarization.
- Staple the notarization ticket.

## Windows

The current packaging script builds the Rust DLL and Flutter Windows release directory:

```powershell
powershell -ExecutionPolicy Bypass -File packaging/windows/build_installer.ps1
```

Production release still requires:

- Code signing certificate.
- MSI or EXE installer tooling such as WiX Toolset or Inno Setup.
- Windows DPAPI validation on Windows 10/11.
- Installer upgrade and uninstall tests.

## Linux

The current packaging script builds the Rust shared library and Flutter Linux bundle:

```bash
packaging/linux/build_packages.sh
```

Production release still requires:

- deb/rpm/AppImage packaging.
- Desktop entry and icon validation.
- Permission validation on Ubuntu/Debian/UOS/Kylin.
- Optional Secret Service/libsecret integration tests.
