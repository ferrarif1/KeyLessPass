# Windows 安装与构建说明

本文面向需要在 Windows 10/11 上构建 KeyLessPass 桌面客户端的开发者或测试人员。流程从安装 Flutter 开始，最终输出可运行的 Windows release 目录。

## 1. 安装 Flutter SDK

1. 打开 Flutter 官方 Windows 桌面开发说明：
   <https://docs.flutter.dev/platform-integration/windows/setup>
2. 下载 Flutter SDK for Windows。
3. 建议解压到一个不需要管理员权限、路径中不含中文和空格的位置，例如：

```powershell
C:\src\flutter
```

4. 把 Flutter 加入当前用户的 `PATH`：

```powershell
C:\src\flutter\bin
```

5. 重新打开 PowerShell，确认 Flutter 可用：

```powershell
flutter --version
flutter doctor -v
```

如果 `flutter` 命令不可用，先检查 `PATH` 是否包含 `C:\src\flutter\bin`。

## 2. 安装 Visual Studio 2022 C++ 桌面工具

Flutter Windows 桌面构建需要 Microsoft C++ 桌面工具链。

1. 安装 Visual Studio 2022 Community 或 Visual Studio 2022 Build Tools。
2. 在 Visual Studio Installer 中选择 workload：

```text
Desktop development with C++
```

3. 确认包含以下组件：

- MSVC v143 x64/x86 build tools
- Windows 10 SDK 或 Windows 11 SDK
- C++ CMake tools for Windows

4. 重新打开 PowerShell，检查 Flutter 是否识别到 Windows 工具链：

```powershell
flutter doctor -v
```

正常情况下应看到类似：

```text
[✓] Visual Studio - develop Windows apps
```

## 3. 启用 Flutter Windows Desktop

较新的 Flutter 版本通常默认支持 desktop。仍可显式启用一次：

```powershell
flutter config --enable-windows-desktop
flutter devices
```

确认设备列表中包含：

```text
windows
```

## 4. 安装 Rust

KeyLessPass 的密码学核心使用 Rust 编译为 Windows DLL。

1. 打开 Rust 官方安装页：
   <https://www.rust-lang.org/tools/install>
2. 安装 `rustup`。
3. 安装后重新打开 PowerShell，确认 Rust 可用：

```powershell
rustc --version
cargo --version
```

4. 确认使用 MSVC toolchain：

```powershell
rustup default stable-x86_64-pc-windows-msvc
rustup show
```

## 5. 获取项目代码

如果已经有仓库目录，可以跳过本步骤。否则：

```powershell
git clone <repository-url> KeyLessPass
cd KeyLessPass
```

仓库采用 source-available license。企业生产部署、商业使用、二次分发、OEM、渠道销售或托管服务需要单独书面授权。

## 6. 安装 Flutter 依赖

```powershell
cd flutter_app
flutter pub get
flutter analyze
flutter test
cd ..
```

## 7. 测试 Rust Core

```powershell
cd rust_core
cargo test
cd ..
```

## 8. 本地运行 Windows 桌面客户端

```powershell
cd flutter_app
flutter run -d windows
cd ..
```

如果启动失败，优先重新运行：

```powershell
flutter doctor -v
```

并修复 `Windows version` 或 `Visual Studio - develop Windows apps` 下的错误。

## 9. 构建 Windows Release

在仓库根目录运行：

```powershell
powershell -ExecutionPolicy Bypass -File packaging\windows\build_installer.ps1
```

脚本会执行：

1. `cargo build --release`
2. `flutter build windows --release`
3. 把 Rust Core DLL 复制到 Flutter Windows release 目录

输出目录为：

```text
flutter_app\build\windows\x64\runner\Release
```

该目录中应包含：

```text
KeyLessPass.exe
keylesspass_core.dll
data\
```

可以直接双击 `KeyLessPass.exe` 做本机测试。

## 10. 生成安装包

当前仓库的 Windows 脚本先输出 release 运行目录。用于分发时，还需要接入安装包工具，例如：

- WiX Toolset 生成 MSI；
- Inno Setup 生成 EXE installer；
- 企业内部软件分发系统打包。

正式发布前还需要：

- Windows 代码签名证书；
- Windows 10/11 真机验证；
- 安装、升级、卸载测试；
- U 盘读写和 DPAPI 行为验证；
- 日志和诊断信息脱敏检查。

## 11. 常见问题

### flutter doctor 提示 Visual Studio 不可用

重新打开 Visual Studio Installer，确认安装了 `Desktop development with C++` workload，并包含 Windows SDK 和 CMake tools。

### 找不到 keylesspass_core.dll

请使用仓库根目录的打包脚本：

```powershell
powershell -ExecutionPolicy Bypass -File packaging\windows\build_installer.ps1
```

不要只运行 `flutter build windows`，否则 Rust Core DLL 可能没有被复制到 release 目录。

### 路径中有空格或中文导致构建异常

建议将 Flutter SDK 和项目目录放到简单路径，例如：

```text
C:\src\flutter
C:\work\KeyLessPass
```

### Windows Defender 或企业终端安全软件拦截

本地未签名构建可能被拦截。PoC 测试时可使用企业允许名单；正式分发应使用代码签名证书和可信安装包。
