# 商业设备授权设计

English: [device-batch-authorization.md](device-batch-authorization.md)

实际部署和操作请看 [设备授权实现与项目使用指南](device-batch-authorization-implementation.zh-CN.md)。

## 目标与边界

授权系统控制客户、终端数量、功能、版本和有效期，但不参与密码派生。授权后台和授权文件绝不能包含助记短语、`Kmaster`、`deviceSecret`、`usbSecret`、CDR secret、服务密码、派生密码或恢复密钥。授权失败不得删除密码数据。

## 信任模型

```text
厂商根公钥（编译进客户端）
  验证 customer entitlement
    委托客户现场公钥
    限制数量/日期/功能/版本
    批准 deviceKeyId 白名单
      验证客户现场签发的设备 bundle
        匹配本机设备私钥、公钥和受保护指纹
```

厂商根私钥始终离线保管。客户只持有现场私钥。客户端不会因为现场私钥能正确签名就直接信任它；现场公钥必须由厂商授权，grant 的设备 key ID 还必须在厂商白名单内。

这一设计修复了“客户拿到服务端私钥后可自行把 `maxSeats` 改大或分叉签发”的根本问题。

## 授权对象

- Customer entitlement：客户、现场公钥、授权序列号、三类数量限制、最大离线宽限天数、有效期、功能、版本和批准设备 key ID。
- Organization license：客户内部组织、方案、席位、功能、版本、宽限期。
- Device request：设备公钥、key ID、平台信息和设备私钥签名证明。
- Device grant：绑定设备 key ID、公钥、商业设备 ID、HMAC 指纹、有效期和功能。
- Bundle：厂商 entitlement、组织授权、设备 grant、吊销列表和现场签名。
- Security state：已见最大 entitlement serial、最新 bundle 时间、最大本机时间和独立存放的受保护历史标记；清除本地授权时仍保留。

当前 schema version 为 `2`。

## 设备身份

设备身份以 Ed25519 私钥为主，公开 UUID 只作辅助。Windows 使用 DPAPI 保护设备密钥和授权安全状态；macOS 使用 Keychain 保护；Linux 使用现有平台 provider 降级。设备请求证明持有私钥，后台拒绝同一个 key ID 注册成多个设备，也禁止现有设备静默换 key。

更高安全等级应把接口替换为 TPM 2.0、Secure Enclave 或 PKCS#11/HSM 非导出密钥，并增加 attestation。

## 席位和吊销

`seat_allocations` 单独记录 active、expired 和 revoked 状态。签发在 SQLite `BEGIN IMMEDIATE` 事务中完成清理、计数、分配、bundle/grant 写入，避免并发超发。吊销设备时会吊销其全部活动 grant 并禁止原身份再次激活。

自动内网客户端每 30 分钟续签默认 24 小时租约，并带 1 天默认宽限，吊销延迟有上限但不是即时。手工导出的静态离线授权仍只能在收到新包或原授权到期后失效。

## 严格节点授权与浮动授权

当前安全默认值是厂商批准设备白名单，适合严格控制“授权几个终端”。客户管理员仍能收集请求和发包，但新增终端必须经厂商提高 serial 后批准。

如果客户完全控制内网服务器、代码和现场私钥，单凭一个厂商签名的 `maxConcurrentDevices` 数字无法严格控制浮动并发，因为恶意服务器可以分叉状态。严格浮动授权必须选择以下之一：

- 厂商在线短租约和 heartbeat；
- TPM/HSM 固定的内网授权服务器；
- Sentinel LDK、CodeMeter 等硬件授权系统。

## 无法绝对解决的风险

- 机器所有者 patch 客户端跳过检查；
- 完整克隆离线虚拟机及其受保护系统状态；
- 删除全部本地状态后以新设备重新申请；
- 永不联网的设备无法即时吊销。

平台代码签名、官方校验和、签名更新、厂商签发账本、客户水印、支持准入和合同审计仍然必要。普通 U 盘不是不可复制授权加密狗。
