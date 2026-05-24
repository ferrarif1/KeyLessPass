# KeyLessPass

KeyLessPass 是一个仅本地运行的跨平台桌面客户端，用于企业内部遗留系统、运维控制台、厂商入口、数据库网关等仍依赖文本密码的场景。它按需派生服务密码，不保存目标系统明文密码，不维护加密服务密码库，也不保存助记词。

## 核心能力

- Flutter Desktop 原生桌面 UI + Rust Core 安全核心。
- SQLite 仅保存 CDR 元数据和完整性标签。
- 普通 U 盘作为 USB 因子包载体。
- 初始化时随机生成每用户主密钥。
- 支持本机生成英文或简体中文助记短语，生成后不保存。
- 派生路径基于 `recordSeq`、`recordId`、`version`、`salt` 和 `encodingDescriptor`。
- `displayName`、`serviceHint`、`accountHint`、备注仅用于展示和检索，修改后不会改变密码。
- 支持 pending / commit / cancel 的两阶段密码轮换。
- 支持中英文界面。

## 明确不做

- 不做 Web 应用。
- 不做云同步。
- 不做浏览器插件或自动填充。
- 不做远程后台。
- 不上传密码、助记词、因子包或 CDR。

## 构建

```bash
cd rust_core
cargo test

cd ../flutter_app
flutter pub get
flutter analyze
flutter test
flutter run -d macos
```

macOS 发布构建可参考：

```bash
DEVELOPER_DIR=/Applications/Xcode.app/Contents/Developer flutter build macos --release
```

更多说明见英文 [README.md](README.md)、[PRIVACY.md](PRIVACY.md)、[SECURITY.md](SECURITY.md) 和 [RELEASE.md](RELEASE.md)。
