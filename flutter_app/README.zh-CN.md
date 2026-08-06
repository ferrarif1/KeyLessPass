# KeyLessPass 桌面客户端

English: [README.md](README.md)

这是 KeyLessPass 的 Flutter Desktop 客户端。普通用户通过它初始化本机、创建 U 盘因子、添加记录、派生密码、做轮换和恢复。

## 本地运行

先构建 Rust Core：

```bash
cd ../rust_core
cargo build
```

再运行 Flutter：

```bash
cd ../flutter_app
flutter pub get
flutter run -d macos
```

如果 macOS 提示找不到 `xcodebuild`：

```bash
DEVELOPER_DIR=/Applications/Xcode.app/Contents/Developer flutter run -d macos
```

## 检查

```bash
flutter analyze
flutter test
```
