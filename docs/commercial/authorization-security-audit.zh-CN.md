# 商业授权安全审计报告

English: [authorization-security-audit.md](authorization-security-audit.md)

审计日期：2026-07-22

范围：`rust_core` 授权校验与功能守卫、Flutter 动态库加载、`admin_backend` 信任链/认证/席位/吊销、商业构建与三平台打包脚本、授权部署文档。

## 结论

当前代码已从“客户后台就是客户端信任根”改为“厂商根授权客户现场公钥，并逐设备批准 key ID”。在不修改官方客户端的前提下，拿到客户后台、数据库和现场私钥的人不能把未被厂商批准的新终端变成有效授权终端。

这解决了本次审计中最直接的收入风险，但不等于本地应用不可破解。掌握终端管理员权限的人仍可尝试 patch 客户端；纯离线虚拟机完整克隆和即时吊销也没有纯软件绝对解法。

## 已修复问题

| 风险 | 修复 |
| --- | --- |
| 运行时环境变量可注入攻击者公钥 | 可信公钥环只在编译期读取；商业包嵌入厂商根公钥 |
| 客户后台私钥可无限签发 | 厂商签名 customer entitlement 委托现场公钥，并要求 grant 的 `deviceKeyId` 在厂商白名单 |
| 复制公开 UUID/应用目录可克隆指纹 | 设备 Ed25519 私钥、持有证明、HMAC 设备指纹；Windows DPAPI machine scope、macOS Keychain 严格保护 |
| 同一设备 key 可伪装成多台设备 | 后台唯一索引、身份不可换 key、跨组织/跨身份冲突拒绝 |
| 并发激活超发 | `seat_allocations` + SQLite `BEGIN IMMEDIATE` 原子清理、计数、分配和写入 |
| 过期 grant 永久占席位 | 到期 allocation 自动标记 expired |
| 吊销后重新激活 | 吊销设备全部活动 grant，释放/标记席位，原身份禁止重激活 |
| 客户现场扩大功能/版本/期限/宽限 | 客户端同时校验厂商、组织、grant 三层约束；厂商授权限制最大离线宽限 |
| 系统时间和旧包回滚 | 保存受保护的最大时间、最大 entitlement serial 和最新 bundle 时间；清除授权不删除该状态 |
| 商业客户端加载调试动态库 | Release 只加载安装包内固定动态库路径 |
| 无授权配置误打正式包 | 商业打包强制授权公钥；macOS/Windows 要求平台签名，Linux 要求 GPG 签名清单；仅本机测试可显式允许 unsigned |
| 未授权仍能调用辅助功能 | 核心写操作、派生、恢复、列表、U 盘工具和助记词生成均受商业授权守卫；只保留状态、激活/导入、清除授权和重置应用数据 |
| Admin token 长期写入浏览器 | 改为 tab 级 `sessionStorage`；服务端使用恒定时间摘要比较和角色/组织范围 |
| 下载应用也要求登录 | `/download`、`/api/downloads`、`/downloads/*` 公开；管理 API 仍要求 token |

## 验证证据

- Rust Core：49 项测试通过；包括错误设备、复制身份、伪造签名、厂商白名单、功能/宽限越权、旧包回滚和商业编译强制阻断。
- Admin Backend：6 项测试通过；包括设备证明、跨组织身份、重复设备 key 和席位超发拒绝。
- Flutter：11 项测试通过，`flutter analyze` 无问题。
- 实际进程烟雾测试：`/download` 与 `/healthz` 返回 200；管理 API 无 token 返回 401，带正确 token 返回 200。
- Shell 脚本语法和 `git diff --check` 通过。

## 仍然存在的边界

### 高风险但无法由纯离线软件消除

1. 终端所有者可以修改二进制或替换整个客户端逻辑。必须依赖平台代码签名、官方校验和、签名更新、客户水印和合同审计提高成本。
2. 完整复制离线虚拟机、虚拟 TPM/系统密钥、磁盘和快照时，两个永不连接共同服务器的副本无法被纯软件可靠区分。
3. 客户完全控制内网浮动授权服务器时，可以修改代码并分叉并发状态。严格并发控制需要厂商在线短租约、TPM/HSM 授权服务器或成熟硬件授权产品。
4. 离线吊销只能在客户端收到更新包或旧授权到期后生效。

### 平台差异

- Windows：当前为 DPAPI machine-scope 软件密钥；建议高安全版升级 TPM 2.0 CNG 非导出密钥和 attestation。
- macOS：当前为 Keychain 保护的软件 Ed25519；建议 Apple Silicon 高安全版升级 Secure Enclave P-256。
- Linux：当前为文件权限和本地 AEAD，能够随完整系统镜像复制；高安全销售必须要求 TPM2/PKCS#11/HSM 或只提供在线/内网租约模式。

## 正式销售前必须执行

- 厂商根私钥离线保存，不进入客户服务器、CI 普通变量、客户端或仓库；建立双人签发和备份恢复流程。
- 每次批准设备提高 `entitlementSerial`，保留客户、合同、设备 key、旧/新 serial 和签发人账本。
- 客户现场服务使用 HTTPS 反向代理、激活接口限流、管理网段限制、定期备份和日志脱敏。
- macOS 使用 Developer ID、hardened runtime、notarize 和 staple；Windows 使用 Authenticode；Linux发布签名校验清单。
- 不向高对抗客户承诺“纯软件绝对防破解”或“普通 U 盘不可复制”。这类客户应销售 TPM/HSM 或 Sentinel/CodeMeter 高安全版本。
