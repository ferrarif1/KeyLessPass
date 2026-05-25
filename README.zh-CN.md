# KeyLessPass

> Keypassless is source-available, not open-source.
> The code is provided for evaluation, security review, learning, and non-commercial testing only.
> Commercial use, enterprise deployment, redistribution, OEM integration, white-label use, managed service use, or channel resale requires a separate written commercial license.

> Keypassless 采用“源码可见但非开源”的授权模式。
> 本仓库代码仅供评估、安全审查、学习和非商业测试使用。
> 企业部署、商业使用、二次分发、OEM 集成、白标使用、托管服务或渠道销售，均需另行取得书面商业授权。

<p align="center">
  <img src="docs/readme-assets/logo.png" width="112" alt="KeyLessPass logo" />
</p>

<p align="center">
  <strong>面向企业桌面场景的 storage-free 本机密码派生客户端。</strong>
</p>

<p align="center">
  <a href="README.md">English</a>
  ·
  <a href="SECURITY.md">Security</a>
  ·
  <a href="COMMERCIAL.md">Commercial</a>
  ·
  <a href="PRIVACY.md">Privacy</a>
  ·
  <a href="RELEASE.md">Release</a>
</p>

KeyLessPass 是一个真正的本机桌面客户端，用于企业内部仍依赖文本密码登录的遗留系统、运维控制台、厂商门户、数据库网关和网络设备。它按需派生服务密码，而不是保存服务密码库。

KeyLessPass 不是 Web 应用，不是浏览器插件，不是云密码管理器，也不是传统 vault。它不保存目标系统明文密码，不维护加密服务密码库，也不保存助记短语。

## 界面预览

| 初始化 | 记录 |
| --- | --- |
| ![初始化](docs/readme-assets/screenshots/01-enrollment.png) | ![记录](docs/readme-assets/screenshots/02-records.png) |

| 派生密码 | 轮换 |
| --- | --- |
| ![派生密码](docs/readme-assets/screenshots/03-derive-password.png) | ![轮换](docs/readme-assets/screenshots/04-rotation.png) |

| U 盘设备与恢复 |
| --- |
| ![U 盘设备与恢复](docs/readme-assets/screenshots/05-usb-recovery.png) |

## 核心能力

- Flutter Desktop 原生桌面 UI + Rust 安全核心。
- SQLite 仅保存非秘密 CDR 元数据和完整性标签。
- 普通 U 盘作为 USB 因子包载体。
- 初始化时随机生成每用户主密钥。
- 支持本机生成英文或简体中文助记短语，生成后不保存。
- 派生路径基于稳定的 `recordSeq`、`recordId`、`version`、`salt` 和 `encodingDescriptor`。
- `displayName`、`serviceHint`、`accountHint`、备注仅用于展示和检索，修改后不改变密码。
- 支持 pending / commit / cancel 的两阶段密码轮换。
- 支持使用两个可用因子重建缺失的 U 盘包或本机包。
- 支持 U 盘路径选择、包校验和 U 盘包重建。
- 支持不含敏感数据的诊断信息导出。
- 预留 macOS、Windows、Linux 平台因子适配层。
- 支持英文和简体中文界面。

## 保存什么，不保存什么

| 本机保存 | 不保存 |
| --- | --- |
| 受保护的本机因子包 | 目标系统明文密码 |
| 用户选择的 U 盘因子包 | 加密服务密码库 |
| CDR 元数据、盐值、版本和 MAC 标签 | 助记短语 |
| 本地恢复元数据 | 云账号、同步状态、分析数据 |

## 简要工作原理

初始化时，KeyLessPass 生成随机 256-bit 每用户主密钥。助记短语不是服务密码根种子，也不会被保存。它会先通过 KDF 形成独立的助记短语因子 `F_M`，再和本机因子 `F_C`、U 盘因子 `F_U` 一起参与本地 2-of-3 解封装/恢复模型，用于恢复 `Kmaster`。服务密码派生阶段使用恢复出的 `Kmaster` 和稳定的 CDR 路径字段。

每条记录只保存非秘密 CDR 元数据。显示名称、服务提示、账号提示和备注可搜索、可编辑，但不参与派生路径。修改密码规则必须创建新版本，并被视为一次密码轮换。

```mermaid
flowchart LR
    M["助记短语<br/>不保存"] --> KDF["KDF"]
    KDF --> FM["助记短语因子 F_M"]
    FC["本机因子 F_C<br/>平台保护"] --> R["2-of-3 解封装 / 恢复"]
    FM --> R
    FU["U 盘因子 F_U<br/>U 盘因子包"] --> R
    R --> KM["恢复出的 Kmaster"]
    KM --> D["HKDF + 确定性编码"]
    C["CDR 稳定字段<br/>recordSeq + recordId + version + salt + Rule"] --> D
    D --> P["服务密码<br/>短暂显示 / 自动清剪贴板"]
    FU --> U["U 盘保存<br/>U 盘因子包<br/>可选 CDR 副本<br/>无明文密码<br/>无助记短语<br/>无明文 Kmaster"]
    C --> U
```

## 安全模型

- 不把目标系统明文密码写入磁盘。
- 不维护加密服务密码库。
- 不保存助记短语。
- 不包含云同步、远程后台、浏览器自动填充或账号登录体系。
- 随机数来自操作系统 CSPRNG。
- 使用前校验 CDR 和因子包完整性。
- 派生密码默认遮罩显示，并在配置时间后清空剪贴板。
- 日志不得包含助记短语、主密钥、因子秘密、HKDF 原始输出、AEAD key、HMAC key 或派生密码。

第一版客户端只承诺本地和 U 盘元数据的一致性/完整性检查。若需要更强回滚检测，可接入外部版本摘要、append-only 审计日志或可信单调状态。

## 桌面导航

当前主导航按稳定对象组织：

- 首页
- 初始化
- 记录
- U 盘设备
- 安全
- 设置
- 关于

添加记录、派生密码和轮换从“记录”进入；恢复工具放在“U 盘设备”中。

## 项目结构

```text
KeyLessPass
├── flutter_app/          # Flutter Desktop UI
├── rust_core/            # Rust 密码学、存储、恢复和 FFI 核心
├── packaging/            # macOS、Windows、Linux 打包脚本
├── docs/                 # 产品化、安全、发布和设计文档
└── releases/             # 本地发布产物，git 忽略
```

## 快速开始

### 环境要求

- Flutter Desktop SDK
- Rust toolchain
- macOS: Xcode
- Windows: Visual Studio Build Tools
- Linux: Flutter Linux desktop dependencies

### 测试 Rust Core

```bash
cd rust_core
cargo test
```

### 运行桌面客户端

```bash
cd flutter_app
flutter pub get
flutter analyze
flutter test
flutter run -d macos
```

### macOS 发布构建

```bash
DEVELOPER_DIR=/Applications/Xcode.app/Contents/Developer \
FLUTTER_BIN=/path/to/flutter \
CODESIGN_IDENTITY='-' \
packaging/macos/build_dmg.sh
```

分发前需要使用 Developer ID Application 证书签名并 notarize。详见 [RELEASE.md](RELEASE.md)。

## 当前状态

macOS 是当前主要验证平台。Windows 和 Linux 的架构、平台因子接口和打包入口已经预留，后续需要在真实系统上完成硬化和发布验证。

## 文档

- [SECURITY.md](SECURITY.md)
- [PRIVACY.md](PRIVACY.md)
- [RELEASE.md](RELEASE.md)
- [CONTRIBUTING.md](CONTRIBUTING.md)
- [DEVELOPMENT.md](DEVELOPMENT.md)
- [docs/PRODUCTIZATION_REPORT.md](docs/PRODUCTIZATION_REPORT.md)
- [docs/STORE_READINESS_CHECKLIST.md](docs/STORE_READINESS_CHECKLIST.md)

## License

Keypassless 采用源码可见但非开源的授权模式。详见 [LICENSE](LICENSE)、[NOTICE](NOTICE) 和 [COMMERCIAL.md](COMMERCIAL.md)。

允许个人学习、评估、安全审查和非商业测试。企业生产部署、商业使用、二次分发、OEM 或白标集成、托管服务、安全服务打包、渠道销售，以及处理真实生产凭据，均需另行取得书面商业授权。
