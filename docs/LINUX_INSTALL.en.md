# Linux Install And Build Guide

Chinese: [LINUX_INSTALL.md](LINUX_INSTALL.md)

Use this guide to build and test KeyLessPass on Linux desktop environments.

## Requirements

- 64-bit Linux desktop.
- GTK 3 desktop dependencies.
- Flutter SDK with Linux desktop support.
- Rust stable GNU toolchain.
- A writable USB drive for USB factor testing.

## Install Flutter

Follow Flutter's Linux desktop setup guide, add Flutter to `PATH`, then run:

```bash
flutter --version
flutter doctor -v
```

Enable desktop if needed:

```bash
flutter config --enable-linux-desktop
flutter devices
```

## Install System Dependencies

Ubuntu/Debian:

```bash
sudo apt update
sudo apt install -y \
  clang \
  cmake \
  ninja-build \
  pkg-config \
  libgtk-3-dev \
  liblzma-dev \
  libstdc++-12-dev
```

For UOS, Kylin, or other Debian-based systems, install the equivalent GTK 3, CMake, Ninja, Clang, and pkg-config packages.

## Install Rust

Install Rust from <https://www.rust-lang.org/tools/install>, then check:

```bash
rustc --version
cargo --version
```

## Run Locally

```bash
cd rust_core
cargo build

cd ../flutter_app
flutter pub get
flutter run -d linux
```

## Test

```bash
cd rust_core
cargo test

cd ../flutter_app
flutter analyze
flutter test
```

## Build Packages

```bash
FLUTTER_BIN="$HOME/development/flutter/bin/flutter" \
packaging/linux/build_packages.sh
```

Outputs:

```text
flutter_app/build/linux/x64/release/bundle
dist/linux/KeyLessPass-linux-x64-0.1.0.tar.gz
dist/linux/keylesspass_0.1.0_amd64.deb
dist/linux/KeyLessPass-linux-x64-0.1.0.AppImage
```

AppImage is generated only when `appimagetool` is available.

Run the bundle directly:

```bash
cd flutter_app/build/linux/x64/release/bundle
./keylesspass_desktop
```

## Common Problems

If Flutter reports missing Linux desktop dependencies, install GTK 3, CMake, Ninja, Clang, and pkg-config.

If `libkeylesspass_core.so` is missing, build with `packaging/linux/build_packages.sh` rather than `flutter build linux` alone.

If no `.deb` is generated, install `dpkg-dev`.

If no AppImage is generated, install `appimagetool` or use the `.tar.gz` package.

On UOS/Kylin, validate USB access, CPU architecture, GTK version, and enterprise endpoint restrictions on real target machines.
