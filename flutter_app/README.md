# KeyLessPass Desktop Client

Chinese: [README.zh-CN.md](README.zh-CN.md)

This is the Flutter Desktop client for KeyLessPass. Users use it to enroll a local profile, create a USB factor package, add credential records, derive passwords, rotate records, and recover factors.

## Run Locally

Build the Rust core first:

```bash
cd ../rust_core
cargo build
```

Run Flutter:

```bash
cd ../flutter_app
flutter pub get
flutter run -d macos
```

If macOS cannot find `xcodebuild`:

```bash
DEVELOPER_DIR=/Applications/Xcode.app/Contents/Developer flutter run -d macos
```

## Check

```bash
flutter analyze
flutter test
```
