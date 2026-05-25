# Linux 安装与构建说明

本文面向需要在 Linux 桌面环境构建和测试 KeyLessPass 的开发者或 PoC 测试人员，覆盖 Ubuntu/Debian 系发行版，以及统信 UOS、麒麟等常见国产 Linux / 信创环境的构建要点。

## 1. 系统要求

- 64-bit Linux 桌面环境；
- GTK 3 桌面依赖；
- 可写 U 盘用于 USB factor package 测试；
- 有 sudo 权限安装构建依赖；
- 目标平台建议优先使用 Ubuntu/Debian 系验证，再扩展到 UOS/麒麟等环境。

## 2. 安装 Flutter SDK

1. 打开 Flutter 官方 Linux 桌面开发说明：
   <https://docs.flutter.dev/platform-integration/linux/setup>
2. 下载 Flutter SDK。
3. 建议解压到用户目录下的稳定路径：

```bash
mkdir -p "$HOME/development"
cd "$HOME/development"
# 将下载的 Flutter SDK 解压为 $HOME/development/flutter
```

4. 把 Flutter 加入 PATH。以 bash 为例：

```bash
echo 'export PATH="$HOME/development/flutter/bin:$PATH"' >> ~/.bashrc
source ~/.bashrc
```

5. 检查 Flutter：

```bash
flutter --version
flutter doctor -v
```

## 3. 安装 Linux 桌面构建依赖

Ubuntu/Debian 可使用：

```bash
sudo apt update
sudo apt install -y \
  clang \
  cmake \
  ninja-build \
  pkg-config \
  libgtk-3-dev \
  liblzma-dev \
  libstdc++-12-dev
```

不同发行版包名可能略有差异。若 `flutter doctor -v` 提示缺少 Linux desktop 依赖，请按提示补齐。

统信 UOS、麒麟等 Debian 系环境通常也可以从以上依赖开始；如果仓库源包名不同，请使用系统软件源中对应的 GTK 3、CMake、Ninja、Clang 和 pkg-config 包。

## 4. 启用 Flutter Linux Desktop

较新的 Flutter 版本通常默认支持 desktop。仍可显式启用一次：

```bash
flutter config --enable-linux-desktop
flutter devices
```

确认设备列表中包含：

```text
linux
```

## 5. 安装 Rust

1. 打开 Rust 官方安装页：
   <https://www.rust-lang.org/tools/install>
2. 安装 `rustup`。
3. 重新打开终端，检查：

```bash
rustc --version
cargo --version
```

通常使用默认 stable GNU toolchain 即可。

## 6. 获取项目代码

如果已经有仓库目录，可以跳过本步骤。否则：

```bash
git clone <repository-url> KeyLessPass
cd KeyLessPass
```

仓库采用 source-available license。企业生产部署、商业使用、二次分发、OEM、渠道销售或托管服务需要单独书面授权。

## 7. 安装 Flutter 依赖

```bash
cd flutter_app
flutter pub get
flutter analyze
flutter test
cd ..
```

## 8. 测试 Rust Core

```bash
cd rust_core
cargo test
cd ..
```

## 9. 本地运行 Linux 桌面客户端

```bash
cd flutter_app
flutter run -d linux
cd ..
```

如果 `flutter run -d linux` 找不到 Linux 设备，请先运行：

```bash
flutter doctor -v
flutter devices
```

并根据提示补齐系统依赖。

## 10. 构建 Linux Release Bundle

在仓库根目录运行：

```bash
FLUTTER_BIN="$HOME/development/flutter/bin/flutter" \
packaging/linux/build_packages.sh
```

脚本会执行：

1. `cargo build --release`
2. `flutter build linux --release`
3. 把 Rust Core `.so` 复制到 Flutter Linux release bundle

输出目录：

```text
flutter_app/build/linux/x64/release/bundle
```

该目录中应包含：

```text
keylesspass_desktop
lib/libkeylesspass_core.so
data/
```

可以从 bundle 目录直接运行：

```bash
cd flutter_app/build/linux/x64/release/bundle
./keylesspass_desktop
```

## 11. 打包为 deb/rpm/AppImage

当前脚本先输出 release bundle。正式分发时还需要接入发行版打包工具，例如：

- deb：`dpkg-deb`、`fpm` 或 Debian packaging；
- rpm：`rpmbuild`、`fpm`；
- AppImage：`linuxdeploy` / `appimagetool`；
- 国产 Linux / 信创环境：可按客户环境制作 deb/rpm 或离线安装包。

正式包建议包含：

- desktop entry；
- 应用图标；
- 安装路径规范；
- 卸载脚本；
- 依赖声明；
- U 盘访问验证；
- 企业内网离线安装说明。

## 12. 国产 Linux / 信创环境注意事项

统信 UOS、麒麟等环境通常需要额外确认：

- CPU 架构是 x86_64、ARM64 还是其他企业定制架构；
- Flutter Linux desktop 是否支持该架构；
- 系统 GTK 版本；
- 企业终端管控策略是否限制 U 盘访问；
- 目标环境是否允许用户选择可移动卷；
- 是否需要离线依赖包；
- 是否需要国产化适配测试报告。

第一版建议优先输出 x86_64 deb/rpm，并在真实 UOS/麒麟机器上验证：

- 初始化；
- 创建 U 盘因子包；
- 添加记录；
- 派生密码；
- U 盘 CDR 备份同步；
- 恢复和重建 U 盘包。

## 13. 常见问题

### flutter doctor 提示缺少 GTK 或 CMake

安装 Linux 桌面依赖：

```bash
sudo apt install -y clang cmake ninja-build pkg-config libgtk-3-dev liblzma-dev
```

### 运行时找不到 libkeylesspass_core.so

请使用仓库根目录脚本构建：

```bash
packaging/linux/build_packages.sh
```

不要只运行 `flutter build linux`，否则 Rust Core `.so` 可能没有复制到 bundle。

### U 盘无法写入

检查 U 盘挂载点权限和文件系统：

```bash
mount | grep media
ls -ld /media/$USER
```

如果是只读挂载或企业策略限制，需要先在系统层面解除限制。

### 离线环境无法安装依赖

请在联网机器上准备对应发行版和架构的离线依赖包，或使用企业内部软件源。Flutter SDK、Rust toolchain 和系统开发包都需要纳入离线交付清单。
