# KeyLessPass 文档导航

English: [DOCS.md](DOCS.md)

不知道先看哪里时，从这张表开始。每个文档都配有英文和简体中文版本。

## 先看这里

| 你要做什么 | 英文 | 中文 |
| --- | --- | --- |
| 了解项目并本地运行 | [README.md](README.md) | [README.zh-CN.md](README.zh-CN.md) |
| 在 macOS 构建或打包 | [docs/MACOS_INSTALL.en.md](docs/MACOS_INSTALL.en.md) | [docs/MACOS_INSTALL.md](docs/MACOS_INSTALL.md) |
| 在 Windows 构建或打包 | [docs/WINDOWS_INSTALL.en.md](docs/WINDOWS_INSTALL.en.md) | [docs/WINDOWS_INSTALL.md](docs/WINDOWS_INSTALL.md) |
| 在 Linux 构建或打包 | [docs/LINUX_INSTALL.en.md](docs/LINUX_INSTALL.en.md) | [docs/LINUX_INSTALL.md](docs/LINUX_INSTALL.md) |

## 产品和运维

| 主题 | 英文 | 中文 |
| --- | --- | --- |
| 隐私说明 | [PRIVACY.md](PRIVACY.md) | [PRIVACY.zh-CN.md](PRIVACY.zh-CN.md) |
| 安全政策 | [SECURITY.md](SECURITY.md) | [SECURITY.zh-CN.md](SECURITY.zh-CN.md) |
| 发布说明 | [RELEASE.md](RELEASE.md) | [RELEASE.zh-CN.md](RELEASE.zh-CN.md) |
| 更新日志 | [CHANGELOG.md](CHANGELOG.md) | [CHANGELOG.zh-CN.md](CHANGELOG.zh-CN.md) |
| 开发说明 | [DEVELOPMENT.md](DEVELOPMENT.md) | [DEVELOPMENT.zh-CN.md](DEVELOPMENT.zh-CN.md) |
| 贡献规则 | [CONTRIBUTING.md](CONTRIBUTING.md) | [CONTRIBUTING.zh-CN.md](CONTRIBUTING.zh-CN.md) |

## 设计和验收

| 主题 | 英文 | 中文 |
| --- | --- | --- |
| 应用商店元数据 | [docs/APP_METADATA.md](docs/APP_METADATA.md) | [docs/APP_METADATA.zh-CN.md](docs/APP_METADATA.zh-CN.md) |
| 桌面端设计 | [docs/DESIGN.md](docs/DESIGN.md) | [docs/DESIGN.zh-CN.md](docs/DESIGN.zh-CN.md) |
| 产品化报告 | [docs/PRODUCTIZATION_REPORT.md](docs/PRODUCTIZATION_REPORT.md) | [docs/PRODUCTIZATION_REPORT.zh-CN.md](docs/PRODUCTIZATION_REPORT.zh-CN.md) |
| 安全说明 | [docs/SECURITY.md](docs/SECURITY.md) | [docs/SECURITY.zh-CN.md](docs/SECURITY.zh-CN.md) |
| 上架准备清单 | [docs/STORE_READINESS_CHECKLIST.md](docs/STORE_READINESS_CHECKLIST.md) | [docs/STORE_READINESS_CHECKLIST.zh-CN.md](docs/STORE_READINESS_CHECKLIST.zh-CN.md) |
| 2-of-3 恢复实现说明 | [docs/security/2-of-3-recovery-implementation-notes.md](docs/security/2-of-3-recovery-implementation-notes.md) | [docs/security/2-of-3-recovery-implementation-notes.zh-CN.md](docs/security/2-of-3-recovery-implementation-notes.zh-CN.md) |
| ASTER 实现和声明边界 | [docs/ASTER_IMPLEMENTATION_PROFILE.md](docs/ASTER_IMPLEMENTATION_PROFILE.md) | 同一份双语边界文档 |
| ASTER 可复现研究工件 | [research/aster/README.md](research/aster/README.md) | 同一份工件说明 |
| ASTER 证据局限 | [research/aster/LIMITATIONS.md](research/aster/LIMITATIONS.md) | 同一份工件说明 |

## 本地常用命令

测试 Rust Core：

```bash
cd rust_core
cargo test
```

测试 Flutter：

```bash
cd flutter_app
flutter pub get
flutter analyze
flutter test
```

运行 macOS 客户端：

```bash
cd flutter_app
DEVELOPER_DIR=/Applications/Xcode.app/Contents/Developer flutter run -d macos
```
