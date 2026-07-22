# 上架准备清单

English: [STORE_READINESS_CHECKLIST.md](STORE_READINESS_CHECKLIST.md)

## macOS

### 已完成

- App 名称为 KeyLessPass。
- Bundle identifier 为 `com.keylesspass.desktop`。
- Release entitlements 包含可移动媒体、用户选择文件读写和网络 client 权限。
- 打包脚本可构建 Rust Core、构建 Flutter macOS release、复制 Rust 动态库、签名并生成 DMG。
- 已有隐私、安全、发布和支持占位文档。
- 已实现 macOS 原生 U 盘目录选择，不依赖第三方插件。
- 已实现设置页带确认词的本地数据重置。
- 已实现 U 盘 CDR 元数据备份同步和恢复。

### 部分完成

- App icon 资源存在，但正式 DMG/App Store 图标仍需最终审核。
- Sandbox 能力已准备，但仍需 notarized 包真机测试。
- U 盘 CDR 备份冲突文案还需要最后一轮 UX 审核。

### 需要人工配置

- Apple Developer Team ID。
- Developer ID Application 证书。
- Notarization 凭据。
- 公开隐私政策 URL。
- 支持邮箱。
- 官网或产品页。

## Windows

### 已完成

- Rust Core 有 Windows provider 抽象和统一 trait smoke test。
- Windows 打包脚本会构建 Flutter release 并复制 `keylesspass_core.dll`。

### 部分完成

- DPAPI 生产验证已在架构层预留。
- 安装器工具已文档化，但正式配置仍需完善。

### 需要人工配置

- 代码签名证书。
- MSI/EXE 安装器配置。
- Windows 10/11 真机验证。
- 安装、升级、卸载测试。

## Linux / UOS / 麒麟

### 已完成

- Linux provider 抽象存在，并有 trait smoke test。
- Linux 打包脚本会构建 Flutter release 并复制 Rust `.so`。
- 架构不依赖云或浏览器。

### 部分完成

- 第一版使用本地 AEAD 和文件权限保护。
- Secret Service/libsecret 作为后续硬化项保留。

### 需要人工配置

- deb/rpm/AppImage 打包。
- 桌面入口验证。
- Ubuntu、Debian、UOS、麒麟发行版 QA。
- 企业离线安装包。
