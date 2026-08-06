# macOS Install And Build Guide

Chinese: [MACOS_INSTALL.md](MACOS_INSTALL.md)

Use this guide to build and test the KeyLessPass desktop client on macOS.

## Requirements

- macOS 13 or newer is recommended.
- Full Xcode, not only Command Line Tools.
- Flutter Desktop SDK.
- Rust stable toolchain.
- A writable USB drive for USB factor testing.

## Install Xcode

Install Xcode from the Mac App Store or Apple Developer. Open it once, accept the license, and install additional components.

If Flutter cannot find `xcodebuild`, run commands with:

```bash
DEVELOPER_DIR=/Applications/Xcode.app/Contents/Developer flutter build macos
```

or switch globally:

```bash
sudo xcode-select -s /Applications/Xcode.app/Contents/Developer
```

## Install Flutter

Follow Flutter's macOS desktop setup guide, then check:

```bash
flutter --version
flutter doctor -v
```

If Flutter is installed under `$HOME/development/flutter`, add this to your shell profile:

```bash
export PATH="$HOME/development/flutter/bin:$PATH"
```

## Install Rust

Install Rust from <https://www.rust-lang.org/tools/install>, then check:

```bash
rustc --version
cargo --version
```

## Run Locally

From the repository root:

```bash
cd rust_core
cargo build

cd ../flutter_app
flutter pub get
DEVELOPER_DIR=/Applications/Xcode.app/Contents/Developer flutter run -d macos
```

## Test

```bash
cd rust_core
cargo test

cd ../flutter_app
flutter analyze
flutter test
```

## Build A Local DMG

Install both macOS Rust targets:

```bash
rustup target add x86_64-apple-darwin aarch64-apple-darwin
```

Run:

```bash
DEVELOPER_DIR=/Applications/Xcode.app/Contents/Developer \
FLUTTER_BIN="$HOME/development/flutter/bin/flutter" \
CODESIGN_IDENTITY="-" \
packaging/macos/build_dmg.sh
```

Outputs:

```text
flutter_app/build/macos/Build/Products/Release/KeyLessPass.app
dist/macos/KeyLessPass-0.1.0-macos.dmg
```

Use ad-hoc signing only for local PoC testing. Public distribution requires Developer ID signing, notarization, and stapling.

## Common Problems

If `xcodebuild` is unavailable, use `DEVELOPER_DIR=/Applications/Xcode.app/Contents/Developer`.

If USB is visible but not writable, click the folder button in the app and choose the USB root path, for example `/Volumes/WD`.

If re-signing breaks USB access, re-sign with `flutter_app/macos/Runner/Release.entitlements`; it contains removable-media and user-selected file access permissions.
