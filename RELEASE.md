# Release Guide

## macOS

Start with the full macOS setup guide:

- [docs/MACOS_INSTALL.md](docs/MACOS_INSTALL.md)

Build the Rust core and Flutter app:

```bash
cd rust_core
cargo build --release

cd ../flutter_app
DEVELOPER_DIR=/Applications/Xcode.app/Contents/Developer flutter build macos --release
```

The packaging script creates a universal macOS app. Install both Rust standard
library targets before running it:

```bash
rustup target add x86_64-apple-darwin aarch64-apple-darwin
```

The release app is expected at:

```text
flutter_app/build/macos/Build/Products/Release/KeyLessPass.app
```

The packaging script copies and signs the Rust dynamic library and creates a
DMG distribution package:

```bash
CODESIGN_IDENTITY="Developer ID Application: Example Team" packaging/macos/build_dmg.sh
```

For local unsigned validation:

```bash
DEVELOPER_DIR=/Applications/Xcode.app/Contents/Developer \
FLUTTER_BIN=/Users/zhangyuanyi/development/flutter/bin/flutter \
CODESIGN_IDENTITY="-" CREATE_DMG=1 packaging/macos/build_dmg.sh
```

The local test DMG is expected at:

```text
dist/macos/KeyLessPass-0.1.0-macos.dmg
```

Do not re-sign the `.app` without entitlements. Re-signing without
`macos/Runner/Release.entitlements` removes removable-media and user-selected
read/write access, which will break USB factor creation and recovery.

Distribution checklist:

- Configure the real Apple Developer Team ID.
- Use a real Developer ID Application certificate.
- Verify entitlements for removable media and user-selected read/write files.
- Verify both the app executable and `libkeylesspass_core.dylib` contain
  `x86_64 arm64` with `lipo -archs`.
- Create a DMG.
- Submit the signed app or DMG for notarization.
- Staple the notarization ticket.

## Windows

Start with the full Windows setup guide:

- [docs/WINDOWS_INSTALL.md](docs/WINDOWS_INSTALL.md)

At a minimum, install Flutter for Windows desktop development, Visual Studio 2022 with the `Desktop development with C++` workload, and the Rust MSVC toolchain. Verify the environment first:

```powershell
flutter doctor -v
rustc --version
cargo --version
```

The current packaging script builds the Rust DLL, Flutter Windows release
directory, and an Inno Setup installer:

```powershell
powershell -ExecutionPolicy Bypass -File packaging/windows/build_installer.ps1
```

The runnable output is expected at:

```text
flutter_app\build\windows\x64\runner\Release
```

The installer output is expected at:

```text
dist\windows\KeyLessPass-Setup-0.1.0.exe
```

Production release still requires:

- Code signing certificate.
- Installer signing and SmartScreen validation.
- Windows DPAPI validation on Windows 10/11.
- Installer upgrade and uninstall tests.

## Linux

Start with the full Linux setup guide:

- [docs/LINUX_INSTALL.md](docs/LINUX_INSTALL.md)

The current packaging script builds the Rust shared library, Flutter Linux
bundle, and distributable packages:

```bash
packaging/linux/build_packages.sh
```

Expected Linux outputs:

```text
dist/linux/KeyLessPass-linux-x64-0.1.0.tar.gz
dist/linux/keylesspass_0.1.0_amd64.deb
dist/linux/KeyLessPass-linux-x64-0.1.0.AppImage
```

The AppImage is generated only when `appimagetool` is available.

Production release still requires:

- Signing/checksum publication for generated packages.
- Desktop entry and icon validation.
- Permission validation on Ubuntu/Debian/UOS/Kylin.
- Optional Secret Service/libsecret integration tests.
