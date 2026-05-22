<p align="center">
  <img src="docs/assets/logo.png" alt="KeyLessPass" width="96" height="96" />
</p>

<h1 align="center">KeyLessPass</h1>

<p align="center">
  <strong>仅本地运行的桌面密码管理原型 — 按需派生口令，不保存口令库。</strong>
</p>

<p align="center">
  <a href="https://github.com/ferrarif1/KeyLessPass">GitHub</a>
  ·
  <a href="README.md">English</a>
  ·
  <a href="docs/DESIGN.md">设计</a>
  ·
  <a href="docs/SECURITY.md">安全</a>
</p>

---

KeyLessPass 是一个**仅本地运行**的桌面端密码管理原型：不维护传统意义上的“加密密码库”，也不在磁盘上持久化目标系统的明文口令。应用在需要时根据主密钥、本机因子与 USB 因子等材料**确定性派生**服务口令；本地 SQLite 仅保存 **CDR（Credential Derivation Record，凭据派生记录）** 元数据与完整性信息，不存储派生出的密码。

克隆本仓库：

```bash
git clone https://github.com/ferrarif1/KeyLessPass.git
cd KeyLessPass
```

**English documentation:** [README.md](README.md)

## 特性

- **无口令库**：不保存目标系统明文密码；助记词不入库持久化（见 [`docs/SECURITY.md`](docs/SECURITY.md)）。
- **多因子派生**：注册时组合助记词、本机平台因子与 USB 因子包；派生时校验因子完整性（HMAC 等）。
- **CDR 管理**：通过 SQLite 管理 `recordSeq`、`recordId`、`version`、`salt`、`encodingDescriptor` 等稳定派生字段；`displayName`、`serviceHint`、`accountHint` 等展示字段不参与派生（见 [`docs/DESIGN.md`](docs/DESIGN.md)）。
- **桌面 UI（Flutter）**：服务列表、新增凭据、注册、恢复、安全状态与设置；支持快捷键（如派生、轮换）。
- **密码轮换**：修改 `encodingDescriptor` 视为新版本/轮换流程，需确认步骤完成。
- **恢复**：支持通过 USB 因子包或本机材料进行本地/USB 恢复路径。
- **跨平台桌面**：目标平台为 **macOS、Windows、Linux**；各平台使用对应的因子保护实现（macOS Keychain、Windows DPAPI 扩展点、Linux 本地 AEAD + 文件权限；不可用时回退并提示降低安全状态）。
- **JSON FFI**：Flutter 通过 C ABI 调用 Rust Core（`keylesspass_ffi_json` / `keylesspass_ffi_free`）。

**明确不在范围内**：Web 服务、云同步、浏览器插件、浏览器自动填充、以 WebView 为主界面的实现。

## 技术栈

| 组件 | 技术 |
|------|------|
| UI | Flutter Desktop（`keylesspass_desktop`） |
| 核心逻辑与密码学 | Rust 库 `keylesspass_core`（`rlib` / `cdylib` / `staticlib`） |
| 本地元数据 | SQLite（`rusqlite`，bundled） |
| 密码学依赖 | HKDF-SHA256、HMAC-SHA256、AES-GCM 等 |

## 环境要求

- **Rust**：**≥ 1.70**（已在 **rustc 1.70.0** 上验证）；可选用 [`rust-toolchain.toml`](rust-toolchain.toml) 中的 stable 通道（不会覆盖你已安装的编译器版本）
- **Flutter**：SDK **≥ 3.3.0**（见 [`flutter_app/pubspec.yaml`](flutter_app/pubspec.yaml)）
- **桌面平台 SDK**：
  - macOS：Xcode / macOS 桌面支持
  - Windows：Visual Studio 构建工具 + Windows 桌面支持
  - Linux：GTK 等 Flutter Linux 桌面依赖
- **可选**：U 盘路径（注册/恢复时写入或读取 USB 因子包）

首次克隆后若缺少 `macos/`、`windows/`、`linux/` 等平台目录，可运行 [`tools/init_flutter_desktop.sh`](tools/init_flutter_desktop.sh) 生成 Flutter 多桌面工程文件。

## 安装与构建

### 1. 构建 Rust Core

```bash
./tools/build_rust_core.sh
```

或在 `rust_core/` 目录执行：

```bash
cargo build          # 调试库，供 flutter run 开发时加载
cargo build --release # 发布构建
```

### 2. 获取 Flutter 依赖

```bash
cd flutter_app
flutter pub get
```

### 3. 开发运行

先确保 **debug** 版 `libkeylesspass_core`（或 Windows 的 `keylesspass_core.dll`）已构建。Flutter 会按平台在可执行文件旁、`../rust_core/target/debug/` 等路径查找动态库（见 [`flutter_app/lib/ffi/rust_core.dart`](flutter_app/lib/ffi/rust_core.dart)）。

```bash
cd flutter_app
flutter run -d macos    # 或 windows / linux
```

### 4. 发布打包（需手动拷贝动态库）

| 平台 | 脚本 |
|------|------|
| macOS | [`packaging/macos/build_dmg.sh`](packaging/macos/build_dmg.sh) |
| Linux | [`packaging/linux/build_packages.sh`](packaging/linux/build_packages.sh) |
| Windows | [`packaging/windows/build_installer.ps1`](packaging/windows/build_installer.ps1) |

脚本会执行 `cargo build --release` 与 `flutter build <platform> --release`，并将 `libkeylesspass_core` 复制到应用 bundle/输出目录。DMG、DEB/RPM/AppImage、MSI/EXE 安装包需在本机另行配置签名与安装器工具（脚本内均有提示）。

## 使用说明

1. **首次注册（Enroll）**：输入助记词并选择可写的 USB 路径，生成本机因子包与 USB 因子包；助记词不会写入磁盘。
2. **添加凭据**：配置编码规则（`encodingDescriptor`）与稳定派生字段，写入 CDR。
3. **派生密码**：选中凭据后派生；口令仅在界面短暂显示，可复制到剪贴板，**约 30 秒后自动清空剪贴板**，且不写入日志或本地存储。
4. **轮换**：更新编码描述或版本后走轮换与确认流程。
5. **恢复**：在丢失部分因子时，通过 USB 或本机恢复流程重建（详见应用内“恢复”视图）。
6. **安全状态**：查看平台因子保护级别（如 Keychain / DPAPI / 回退模式）。

已在同一设备完成注册时，普通注册流程会被阻止，以避免覆盖主密钥相关恢复材料；需使用恢复流程或显式出厂重置（当前为原型能力，生产环境需另行设计）。

## 配置

### 数据目录

默认应用数据目录（可通过环境变量覆盖）：

| 平台 | 默认路径 |
|------|----------|
| macOS | `~/Library/Application Support/KeylessPass/` |
| Windows | `%APPDATA%\KeylessPass\` |
| Linux | `$XDG_DATA_HOME/keylesspass` 或 `~/.local/share/keylesspass` |

目录内典型文件：

- `keylesspass-config.json` — 应用配置
- `cdr.sqlite3` — CDR 数据库
- `local-factor-package.json` — 本机因子包
- `recovery-metadata.json` — 恢复元数据

### 环境变量

| 变量 | 说明 |
|------|------|
| `KEYLESSPASS_HOME` | 若设置，则作为应用数据根目录（覆盖各平台默认路径） |

## 测试与验证

**Rust 单元/集成测试：**

```bash
cd rust_core
cargo test
```

**Flutter 测试：**

```bash
cd flutter_app
flutter test
```

**功能与性能证据示例**（临时目录 + 模拟 USB，输出 JSON 结果）：

```bash
cd rust_core
cargo run --example evidence_harness
cargo run --example seed_ui_state
```

## 项目结构

```
KeyLessPass/
├── rust_core/          # keylesspass_core：密码学、CDR、因子包、FFI、平台适配
│   ├── src/
│   │   ├── crypto/     # KDF、AEAD、MAC、编码与恢复相关算法
│   │   ├── domain/     # CDR、因子、配置等领域模型
│   │   ├── service/    # 注册、派生、轮换、恢复等业务
│   │   ├── storage/    # SQLite、因子包、USB 存储
│   │   ├── platform/   # macOS / Windows / Linux 因子提供者
│   │   └── ffi.rs      # JSON FFI 入口
│   └── examples/       # evidence_harness、seed_ui_state
├── flutter_app/        # keylesspass_desktop：Flutter 桌面 UI 与 FFI 绑定
├── docs/
│   ├── DESIGN.md       # 架构与派生边界说明（英文）
│   └── SECURITY.md     # 安全注意事项（英文）
├── tools/              # build_rust_core.sh、init_flutter_desktop.sh
├── packaging/          # 各平台 release 构建脚本
└── rust-toolchain.toml
```

## FFI 操作一览

请求 JSON 形如 `{"op":"<操作名>","payload":{...}}`，响应为 `{"ok":true,"data":...}` 或 `{"ok":false,"error":"..."}`。

| `op` | 说明 |
|------|------|
| `getAppStatus` | 是否已注册、配置与安全状态 |
| `getSecurityStatus` | 平台安全状态 |
| `listCredentials` | 列出 CDR |
| `listUsbCandidates` | 枚举可用 USB 路径 |
| `enroll` | 首次注册 |
| `addCredential` | 新增凭据记录 |
| `updateCredentialDisplay` | 更新展示字段 |
| `derivePassword` | 派生口令 |
| `rotateCredential` / `confirmRotation` | 轮换与确认 |
| `recoverUsb` / `recoverLocal` | USB / 本机恢复 |

敏感派生/恢复失败时，FFI 层会返回泛化错误信息，避免泄露内部细节（见 [`docs/DESIGN.md`](docs/DESIGN.md)）。

## 文档

- [DESIGN.md](docs/DESIGN.md) — 架构、派生字段边界、平台 `PlatformFactorProvider`、FFI 约定
- [SECURITY.md](docs/SECURITY.md) — 随机源、持久化边界、日志与 MVP 回滚检测范围

## 许可证

本项目采用 [MIT 许可证](LICENSE)。

## 相关链接

- 公开仓库：<https://github.com/ferrarif1/KeyLessPass>
