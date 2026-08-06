# 产品化报告

English: [PRODUCTIZATION_REPORT.md](PRODUCTIZATION_REPORT.md)

## 摘要

KeyLessPass 已从早期桌面原型升级为 Flutter Desktop + Rust Core 的产品化应用。macOS 是主要验证平台，Windows 和 Linux 架构入口保留。

## 已完成

- 重做桌面外壳：首页、初始化、记录、U 盘设备、安全、设置、关于。
- 记录页支持搜索、过滤、详情、派生、轮换和元数据编辑。
- 增加备注字段，且不影响派生路径。
- 简化轮换：新版本成为当前版本，旧版本仍可用于回滚检查。
- 增加 macOS 可写移动卷检测。
- 增加 macOS 原生目录选择，用于授予 U 盘根目录读写权限。
- U 盘页支持选择路径、校验 U 盘包、重建 U 盘包、同步/恢复 CDR 备份。
- 初始化支持英文和简体中文助记短语，本地生成后不保存。
- 实现论文对齐的 2-of-3 成对 wrapper 恢复模型。
- 支持“本机 + U 盘”重置助记短语，不改变已有派生密码。
- 因子包和恢复元数据增加 schema/version 字段。
- 新初始化增加助记短语校验，避免错误助记短语生成另一套派生结果。
- 设置页支持带确认词的本地数据重置。
- 诊断导出已替换为脱敏诊断。
- Rust Core 阻止已有本机状态时重复初始化。
- 增加英文和简体中文 UI 资源及 i18n 测试。
- macOS bundle 名称和 identifier 已产品化。
- 增加隐私、安全、发布、贡献、开发和就绪清单文档。

## UI 结果

- 左侧导航适合桌面窗口，窄窗口可滚动。
- 首页展示记录数、U 盘状态、完整性状态和快捷操作。
- 记录页负责添加、派生、轮换和编辑。
- 派生页默认遮罩密码，派生后清理助记短语输入。
- 设置页包含语言、主题、剪贴板清理时间、默认密码长度、诊断和本地重置。
- U 盘页包含结构校验、因子重建、CDR 备份、三种恢复路径和助记短语重置。

## 安全加固

- UI 不显示内部因子 secret、主密钥、原始 CDR secret 或历史派生密码。
- 派生密码默认遮罩，并在超时后清空剪贴板。
- Rust Core 测试覆盖派生稳定性、元数据不可变边界、路径字段敏感性、篡改失败、缺失因子、轮换行为和恢复流程。
- 本机和 U 盘 payload 不再保存明文 `Kmaster`。
- 本机 payload 不保存 `usbSecret`，U 盘 payload 不保存 `deviceSecret`。
- V2 `encryptedPayload` 是历史字段名，承载 base64 因子 payload，不是助记短语加密 vault。
- U 盘发现不记录挂载细节。

## 发布准备状态

- macOS entitlements 已包含可移动媒体和用户选择文件读写权限。
- macOS 打包脚本支持 ad-hoc 或 Developer ID 签名，并可生成 DMG。
- Windows/Linux 打包入口保留，待真实平台硬化。

## 待完成

- Apple Developer ID 签名、notarization 和真实隐私政策 URL。
- Windows/Linux 原生 U 盘路径选择器补齐。
- Windows DPAPI 真机验证。
- Linux Secret Service/libsecret 可选集成和 UOS/麒麟打包验证。
- CDR 备份冲突提示的最终 UX 文案。
