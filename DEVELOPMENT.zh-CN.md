# 开发说明

English: [DEVELOPMENT.md](DEVELOPMENT.md)

## 仓库结构

- `flutter_app/`：Flutter Desktop UI 和 FFI 绑定。
- `rust_core/`：密码学、CDR 存储、因子包、恢复和 JSON FFI。
- `admin_backend/`：内网商业设备授权后台。
- `packaging/`：macOS、Windows、Linux 打包入口。
- `docs/`：架构、产品化、发布和使用说明。

## 本地启动

```bash
cd rust_core
cargo build

cd ../flutter_app
flutter pub get
flutter run -d macos
```

如果 macOS 构建失败，并提示当前选中的是 Command Line Tools：

```bash
DEVELOPER_DIR=/Applications/Xcode.app/Contents/Developer flutter run -d macos
```

## 授权后台

```bash
cd admin_backend
./scripts/intranet_deploy.sh
```

打开脚本输出的地址，输入脚本输出的 Admin token。给设备授权的流程看 [admin_backend/README.zh-CN.md](admin_backend/README.zh-CN.md) 和 [docs/commercial/device-batch-authorization-implementation.zh-CN.md](docs/commercial/device-batch-authorization-implementation.zh-CN.md)。

## i18n

修改 `flutter_app/lib/l10n/` 下的 ARB 文件后重新生成：

```bash
cd flutter_app
flutter gen-l10n
flutter test test/i18n_test.dart
```

## macOS U 盘测试

使用可写移动卷，例如 `/Volumes/WD`。KeyLessPass 会在 U 盘根目录写入 `keylesspass-usb-factor.json`。

如果自动扫描能看到 U 盘但不能写入，请在界面中点击 U 盘路径旁的文件夹按钮，选择 U 盘根目录。macOS 会通过 `NSOpenPanel` 授予本次运行的读写权限。
