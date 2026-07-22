# Windows Install And Build Guide

Chinese: [WINDOWS_INSTALL.md](WINDOWS_INSTALL.md)

Use this guide to build and test KeyLessPass on Windows 10/11.

## Requirements

- Flutter SDK with Windows desktop support.
- Visual Studio 2022 or Build Tools with `Desktop development with C++`.
- Rust stable MSVC toolchain.
- Inno Setup 6 if you want to build the installer.

## Install Flutter

Install Flutter, add `flutter\bin` to `PATH`, reopen PowerShell, then run:

```powershell
flutter --version
flutter doctor -v
```

Enable desktop if needed:

```powershell
flutter config --enable-windows-desktop
flutter devices
```

## Install Visual Studio Tools

Open Visual Studio Installer and install:

- `Desktop development with C++`
- MSVC v143 x64/x86 tools
- Windows 10 or Windows 11 SDK
- CMake tools for Windows

Check:

```powershell
flutter doctor -v
```

## Install Rust

Install Rust from <https://www.rust-lang.org/tools/install>, then run:

```powershell
rustup default stable-x86_64-pc-windows-msvc
rustc --version
cargo --version
```

## Run Locally

```powershell
cd rust_core
cargo build

cd ..\flutter_app
flutter pub get
flutter run -d windows
```

## Test

```powershell
cd rust_core
cargo test

cd ..\flutter_app
flutter analyze
flutter test
```

## Build The Installer

Install Inno Setup 6 or set `ISCC`:

```powershell
$env:ISCC = "C:\Program Files (x86)\Inno Setup 6\ISCC.exe"
```

Build:

```powershell
powershell -ExecutionPolicy Bypass -File packaging\windows\build_installer.ps1
```

Outputs:

```text
flutter_app\build\windows\x64\runner\Release
dist\windows\KeyLessPass-Setup-0.1.0.exe
```

Use the script rather than `flutter build windows` alone; the script copies `keylesspass_core.dll` into the release directory.

## Common Problems

If Flutter cannot find Visual Studio, install the C++ desktop workload and Windows SDK.

If `keylesspass_core.dll` is missing, rebuild with `packaging\windows\build_installer.ps1`.

If paths cause build issues, use simple paths such as `C:\src\flutter` and `C:\work\KeyLessPass`.

Unsigned local builds may be blocked by Windows Defender or enterprise endpoint controls. Production releases should be code signed.
