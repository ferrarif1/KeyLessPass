# macOS 安装与构建说明

English: [MACOS_INSTALL.en.md](MACOS_INSTALL.en.md)

本文面向需要在 macOS 上构建和测试 KeyLessPass 桌面客户端的开发者或 PoC 测试人员。流程从安装 Flutter 开始，最终输出 `.app`，也可生成本地 DMG。

## 1. 系统要求

- macOS 13 或更高版本建议用于开发；
- Apple Silicon 或 Intel Mac 均可；
- 需要完整 Xcode，而不是仅安装 Command Line Tools；
- 需要可写 U 盘用于 USB factor package 测试。

## 2. 安装 Xcode

1. 从 Mac App Store 或 Apple Developer 下载并安装 Xcode。
2. 首次打开 Xcode，接受许可协议并安装附加组件。
3. 确认 Xcode 路径：

```bash
ls /Applications/Xcode.app
```

4. 如果系统当前选择的是 Command Line Tools，构建 Flutter macOS app 时请临时指定：

```bash
export DEVELOPER_DIR=/Applications/Xcode.app/Contents/Developer
```

也可以全局切换，但这会影响本机其他构建环境：

```bash
sudo xcode-select -s /Applications/Xcode.app/Contents/Developer
```

## 3. 安装 Flutter SDK

1. 打开 Flutter 官方 macOS 桌面开发说明：
   <https://docs.flutter.dev/platform-integration/macos/setup>
2. 下载 Flutter SDK。
3. 建议解压到用户目录下的稳定路径，例如：

```bash
mkdir -p "$HOME/development"
cd "$HOME/development"
# 将下载的 Flutter SDK 解压为 $HOME/development/flutter
```

4. 把 Flutter 加入 shell PATH。以 `zsh` 为例：

```bash
echo 'export PATH="$HOME/development/flutter/bin:$PATH"' >> ~/.zshrc
source ~/.zshrc
```

5. 检查 Flutter：

```bash
flutter --version
flutter doctor -v
```

如果 `flutter doctor` 提示 Xcode 问题，优先确认 `DEVELOPER_DIR` 是否指向完整 Xcode。

## 4. 安装 Rust

1. 打开 Rust 官方安装页：
   <https://www.rust-lang.org/tools/install>
2. 安装 `rustup`。
3. 重新打开终端，检查：

```bash
rustc --version
cargo --version
```

Apple Silicon 和 Intel Mac 通常使用系统默认 stable toolchain 即可。

## 5. 获取项目代码

如果已经有仓库目录，可以跳过本步骤。否则：

```bash
git clone <repository-url> KeyLessPass
cd KeyLessPass
```

## 6. 安装 Flutter 依赖

```bash
cd flutter_app
flutter pub get
flutter analyze
flutter test
cd ..
```

## 7. 测试 Rust Core

```bash
cd rust_core
cargo test
cd ..
```

## 8. 本地运行 macOS 桌面客户端

```bash
cd flutter_app
DEVELOPER_DIR=/Applications/Xcode.app/Contents/Developer flutter run -d macos
cd ..
```

测试 U 盘时，建议插入一个可写 U 盘，例如路径 `/Volumes/WD`。如果应用能看到 U 盘但不能写入，请在界面中使用文件夹按钮选择 U 盘根目录，触发 macOS 用户授权。

## 9. 构建 macOS 分发包

在仓库根目录运行：

```bash
DEVELOPER_DIR=/Applications/Xcode.app/Contents/Developer \
FLUTTER_BIN="$HOME/development/flutter/bin/flutter" \
CODESIGN_IDENTITY="-" \
packaging/macos/build_dmg.sh
```

打包脚本默认生成 Intel 与 Apple Silicon 通用版本，首次打包前安装两个 Rust target：

```bash
rustup target add x86_64-apple-darwin aarch64-apple-darwin
```

脚本会执行：

1. 分别为 `x86_64-apple-darwin` 和 `aarch64-apple-darwin` 构建 Rust Core，并用 `lipo` 合并
2. `flutter build macos --release`
3. 把 Rust Core dylib 复制到 `.app/Contents/Frameworks/`
4. 使用指定证书或 ad-hoc 签名重新签名 `.app`
5. 生成包含 `KeyLessPass.app` 和 `Applications` 快捷方式的 DMG

`.app` 输出路径：

```text
flutter_app/build/macos/Build/Products/Release/KeyLessPass.app
```

DMG 分发包输出路径：

```text
dist/macos/KeyLessPass-0.1.0-macos.dmg
```

小范围 PoC 测试时，把这个 DMG 发给测试电脑即可。测试者打开 DMG 后把 `KeyLessPass.app` 拖到 `Applications`。本地 unsigned/ad-hoc DMG 仅用于测试，不适合公开发布。

## 10. 正式分发准备

正式发布 `.dmg` 前需要：

- Apple Developer 账号；
- Developer ID Application 证书；
- 正确的 bundle identifier；
- entitlements 检查，尤其是 removable media 和 user-selected read/write；
- DMG 签名；
- notarization；
- stapler 票据装订；
- macOS 13/14/15 真机验证；
- U 盘创建、校验、恢复流程验证。

## 11. 常见问题

### xcodebuild 不可用

如果出现：

```text
xcrun: error: unable to find utility "xcodebuild"
```

说明当前系统选择的是 Command Line Tools。临时解决：

```bash
DEVELOPER_DIR=/Applications/Xcode.app/Contents/Developer flutter build macos --release
```

### U 盘可见但不能写入

在界面中点击 U 盘路径旁的文件夹按钮，选择 U 盘根目录，例如 `/Volumes/WD`。这会通过系统文件选择器授予用户选择目录的读写权限。

### 双击 app 被 Gatekeeper 拦截

本地 ad-hoc 签名构建可能被 Gatekeeper 提示。PoC 测试可以右键打开；正式分发需要 Developer ID 签名和 notarization。

### 重新签名后 U 盘权限失效

不要丢失 `flutter_app/macos/Runner/Release.entitlements`。重新签名时必须带 entitlements，否则 removable media 或用户选择文件权限可能失效。
