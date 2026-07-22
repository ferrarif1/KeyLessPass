# 安全政策

English: [SECURITY.md](SECURITY.md)

KeyLessPass 是本地密码派生工具。它不保存服务密码，但用户仍需要保护助记短语、本机设备、U 盘因子、恢复材料和操作系统环境。

## 报告漏洞

如果发现安全漏洞，请先不要公开披露。请发送邮件到 revanton@icloud.com，并包含：

- 问题描述；
- 受影响版本或 commit；
- 复现步骤；
- 潜在影响；
- 可选的修复建议。

不要在漏洞报告中包含真实生产密码、企业秘密、客户凭据、私钥或敏感业务数据。

## 评估和 PoC 安全

- 只使用测试账号和测试数据。
- 未经明确授权，不要使用真实企业生产凭据。
- 未取得商业授权和正式批准前，不要部署为生产凭据管理系统。
- 在敏感环境使用前，先验证安全模型。

## 安全边界

KeyLessPass 通过不保存服务密码来降低 vault 风险，但安全仍依赖：

- 助记短语强度和保密性；
- U 盘因子的保护；
- 本机设备安全；
- 应用二进制完整性；
- 备份和恢复流程；
- 用户操作纪律；
- 企业终端和访问控制策略。

## 2-of-3 本地恢复模型

Rust Core 使用三组因子：

```text
F_M = KDF(Normalize(mnemonic), saltM)
F_C = KDF(deviceSecret || deviceID || userID, saltC)
F_U = KDF(usbSecret || usbID || userID, saltU)
```

同一个随机生成的 `Kmaster` 只以 `W_MC`、`W_MU`、`W_CU` 三个包装密文形式落盘：

```text
W_MC = AES-256-GCM(HKDF(F_M || F_C, "KeyLessPass/wrap/MC"), Kmaster)
W_MU = AES-256-GCM(HKDF(F_M || F_U, "KeyLessPass/wrap/MU"), Kmaster)
W_CU = AES-256-GCM(HKDF(F_C || F_U, "KeyLessPass/wrap/CU"), Kmaster)
```

恢复路径：

- 助记短语 + 本机：通过 `W_MC` 恢复，并可重建 U 盘包。
- 助记短语 + U 盘：通过 `W_MU` 恢复，并可重建本机因子。
- 本机 + U 盘：通过 `W_CU` 重置助记短语，不需要旧助记短语。

单个因子不能恢复 `Kmaster`。U 盘是普通可复制存储，不是不可复制硬件密钥。

## 存储边界

- 本机因子包不保存明文 `Kmaster` 或 `usbSecret`。
- U 盘因子包不保存明文 `Kmaster` 或 `deviceSecret`。
- CDR 备份只保存元数据和 MAC，不保存服务密码。
- macOS `com.keylesspass.local-factor` 是平台保护的 `deviceSecret` 来源，不是 `Kmaster`，也不是助记短语。

V2 schema 中的 `encryptedPayload` 是历史字段名，现在承载 base64 编码的因子 payload，不是助记短语加密 vault，也不包含明文 `Kmaster`。
