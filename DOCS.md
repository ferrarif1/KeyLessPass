# KeyLessPass Documentation

Chinese: [DOCS.zh-CN.md](DOCS.zh-CN.md)

Use this page to find the shortest path for the job you want to finish.

## Start Here

| Goal | English | Chinese |
| --- | --- | --- |
| Understand the product and run it locally | [README.md](README.md) | [README.zh-CN.md](README.zh-CN.md) |
| Build or package on macOS | [docs/MACOS_INSTALL.en.md](docs/MACOS_INSTALL.en.md) | [docs/MACOS_INSTALL.md](docs/MACOS_INSTALL.md) |
| Build or package on Windows | [docs/WINDOWS_INSTALL.en.md](docs/WINDOWS_INSTALL.en.md) | [docs/WINDOWS_INSTALL.md](docs/WINDOWS_INSTALL.md) |
| Build or package on Linux | [docs/LINUX_INSTALL.en.md](docs/LINUX_INSTALL.en.md) | [docs/LINUX_INSTALL.md](docs/LINUX_INSTALL.md) |

## Product And Operations

| Topic | English | Chinese |
| --- | --- | --- |
| Privacy | [PRIVACY.md](PRIVACY.md) | [PRIVACY.zh-CN.md](PRIVACY.zh-CN.md) |
| Security policy | [SECURITY.md](SECURITY.md) | [SECURITY.zh-CN.md](SECURITY.zh-CN.md) |
| Release checklist | [RELEASE.md](RELEASE.md) | [RELEASE.zh-CN.md](RELEASE.zh-CN.md) |
| Changelog | [CHANGELOG.md](CHANGELOG.md) | [CHANGELOG.zh-CN.md](CHANGELOG.zh-CN.md) |
| Development | [DEVELOPMENT.md](DEVELOPMENT.md) | [DEVELOPMENT.zh-CN.md](DEVELOPMENT.zh-CN.md) |
| Contributing | [CONTRIBUTING.md](CONTRIBUTING.md) | [CONTRIBUTING.zh-CN.md](CONTRIBUTING.zh-CN.md) |

## Design And Readiness

| Topic | English | Chinese |
| --- | --- | --- |
| App metadata | [docs/APP_METADATA.md](docs/APP_METADATA.md) | [docs/APP_METADATA.zh-CN.md](docs/APP_METADATA.zh-CN.md) |
| Desktop design | [docs/DESIGN.md](docs/DESIGN.md) | [docs/DESIGN.zh-CN.md](docs/DESIGN.zh-CN.md) |
| Productization report | [docs/PRODUCTIZATION_REPORT.md](docs/PRODUCTIZATION_REPORT.md) | [docs/PRODUCTIZATION_REPORT.zh-CN.md](docs/PRODUCTIZATION_REPORT.zh-CN.md) |
| Security notes | [docs/SECURITY.md](docs/SECURITY.md) | [docs/SECURITY.zh-CN.md](docs/SECURITY.zh-CN.md) |
| Store readiness | [docs/STORE_READINESS_CHECKLIST.md](docs/STORE_READINESS_CHECKLIST.md) | [docs/STORE_READINESS_CHECKLIST.zh-CN.md](docs/STORE_READINESS_CHECKLIST.zh-CN.md) |
| Recovery implementation notes | [docs/security/2-of-3-recovery-implementation-notes.md](docs/security/2-of-3-recovery-implementation-notes.md) | [docs/security/2-of-3-recovery-implementation-notes.zh-CN.md](docs/security/2-of-3-recovery-implementation-notes.zh-CN.md) |

## Local Quick Commands

Run the Rust core checks:

```bash
cd rust_core
cargo test
```

Run the Flutter checks:

```bash
cd flutter_app
flutter pub get
flutter analyze
flutter test
```

Run the macOS desktop app:

```bash
cd flutter_app
DEVELOPER_DIR=/Applications/Xcode.app/Contents/Developer flutter run -d macos
```
