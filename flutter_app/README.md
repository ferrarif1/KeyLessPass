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

Commercial clients authorize automatically:

1. read the newest `keylesspass-client-config.json` from a managed path or the user's Downloads directory; the download page delivers it together with the selected installer;
2. use UDP 8788 only when no config is available;
3. register the device and wait for vendor batch approval;
4. poll and import the device grant after customer IT installs the approval.

End users do not open Security, enter a server URL/activation code/Admin token, or move the config manually. If the browser blocks multiple downloads, allow downloads for the intranet site. Managed paths are documented in the [backend guide](../admin_backend/README.md).

The following is only for development and troubleshooting fallback:

1. Open `Security`.
2. Online path: click `Activate online`, then enter the HTTPS service URL and organization activation code.
3. Offline path: click `Copy device request`, import it in the backend, issue a bundle, then click `Import license bundle`.

For local testing, online activation may use:

```text
http://127.0.0.1:8787
```

Manually entered non-loopback addresses require HTTPS. Automatic discovery and managed configuration may use HTTP on an isolated intranet because the resulting license is still verified against the embedded vendor root.
