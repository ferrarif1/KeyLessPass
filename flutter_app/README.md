# KeyLessPass Desktop Client

Chinese: [README.zh-CN.md](README.zh-CN.md)

This is the Flutter Desktop client for KeyLessPass. Users use it to enroll a local profile, create a USB factor package, add credential records, derive passwords, rotate records, recover factors, and import commercial authorization.

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

## Authorization Testing

Start the backend first:

```bash
cd ../admin_backend
./scripts/intranet_deploy.sh
```

Then in the desktop client:

1. Open `Security`.
2. Online path: click `Activate online`, then enter the HTTPS service URL and organization activation code.
3. Offline path: click `Copy device request`, import it in the backend, issue a bundle, then click `Import license bundle`.

For local testing, online activation may use:

```text
http://127.0.0.1:8787
```

Real LAN or internet deployments must use HTTPS.
