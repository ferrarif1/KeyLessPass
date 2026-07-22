# 发布说明

English: [RELEASE.md](RELEASE.md)

## 商业授权构建

商业包必须通过 `tools/commercial/build_commercial_release.sh` 或等价 CI 构建。构建时要：

- 启用 `KEYLESSPASS_REQUIRE_LICENSE=1`；
- 设置非 evaluation 发布渠道；
- 设置托管授权包路径；
- 嵌入授权后台对应的公钥。

密钥轮换时，使用 [docs/commercial/commercial-release-hardening.zh-CN.md](docs/commercial/commercial-release-hardening.zh-CN.md) 中的多公钥信任配置。

发布前必须验证：

- HTTPS 在线激活；
- 离线授权包导入；
- MDM/托管授权包刷新；
- 到期和宽限期；
- 吊销刷新；
- 席位耗尽；
- 干净安装后的未授权状态。

不要分发本地 ad-hoc 签名的授权测试包。

## macOS

完整说明看：[docs/MACOS_INSTALL.md](docs/MACOS_INSTALL.md)

最小构建命令：

```bash
cd rust_core
cargo build --release

cd ../flutter_app
DEVELOPER_DIR=/Applications/Xcode.app/Contents/Developer flutter build macos --release
```

打包前安装两个 Rust target：

```bash
rustup target add x86_64-apple-darwin aarch64-apple-darwin
```

本地测试 DMG：

```bash
DEVELOPER_DIR=/Applications/Xcode.app/Contents/Developer \
FLUTTER_BIN=/Users/zhangyuanyi/development/flutter/bin/flutter \
CODESIGN_IDENTITY="-" CREATE_DMG=1 packaging/macos/build_dmg.sh
```

正式发布需要 Developer ID 签名、notarization 和 staple。

重新签名时必须带 entitlements，特别是 removable media、user-selected read/write 和网络 client 权限，否则 U 盘或在线激活会失败。

## Windows

完整说明看：[docs/WINDOWS_INSTALL.md](docs/WINDOWS_INSTALL.md)

```powershell
powershell -ExecutionPolicy Bypass -File packaging\windows\build_installer.ps1
```

输出：

```text
dist\windows\KeyLessPass-Setup-0.1.0.exe
```

正式发布还需要代码签名证书、安装包签名、SmartScreen 验证、DPAPI 真机验证，以及安装/升级/卸载测试。

## Linux

完整说明看：[docs/LINUX_INSTALL.md](docs/LINUX_INSTALL.md)

```bash
packaging/linux/build_packages.sh
```

常见输出：

```text
dist/linux/KeyLessPass-linux-x64-0.1.0.tar.gz
dist/linux/keylesspass_0.1.0_amd64.deb
dist/linux/KeyLessPass-linux-x64-0.1.0.AppImage
```

AppImage 只有在 `appimagetool` 可用时生成。

正式发布还需要签名或校验和、桌面入口验证、Ubuntu/Debian/UOS/麒麟权限验证，以及可选 Secret Service/libsecret 验证。
