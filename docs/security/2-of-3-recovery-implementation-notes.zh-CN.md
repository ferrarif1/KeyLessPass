# KeyLessPass 2-of-3 本地恢复实现说明

English: [2-of-3-recovery-implementation-notes.md](2-of-3-recovery-implementation-notes.md)

本文记录严格成对 wrapper 恢复 schema 的安全边界，供 Rust Core 和 UI 文案实现使用。

## macOS 本机因子来源

- macOS Keychain item 继续使用：
  - service/location：`com.keylesspass.local-factor`
  - account：`keylesspass`
- 该 item 不是 `Kmaster`，也不是助记短语。
- 它是平台保护的本机因子来源，对应论文中的 `deviceSecret` 输入。
- 它只能用于派生本机因子：

```text
FC = KDF(deviceSecret || deviceID || userID || saltC)
```

- 它不能用于解密一个持久化明文 `Kmaster` 的本机 payload。

## 持久化包边界

- 本机因子包不能保存明文 `Kmaster`。
- 本机因子包不能保存 `usbSecret`。
- U 盘因子包不能保存明文 `Kmaster`。
- U 盘因子包不能保存 `deviceSecret`。
- 助记短语永不落盘。
- 助记短语校验器只能用于校验，不能成为恢复 `Kmaster` 的唯一条件。

## V2 `encryptedPayload`

- `encryptedPayload` 保留在 JSON schema 中用于兼容。
- 在 V2 中，它是历史字段名，保存 base64 编码因子 payload，不是助记短语加密 vault，也不是平台加密 vault。
- 本机 encoded payload 不能包含明文 `Kmaster` 或 `usbSecret`。
- U 盘 encoded payload 不能包含明文 `Kmaster` 或 `deviceSecret`。
- `Kmaster` 在 V2 包中只能出现在 `W_MC`、`W_MU`、`W_CU` wrapper 密文里。

## 成对 wrapper

`Kmaster` 是随机 256-bit root secret。落盘时只能作为以下 wrapper 密文：

```text
K_MC = HKDF(FM || FC, "KeyLessPass/wrap/MC")
K_MU = HKDF(FM || FU, "KeyLessPass/wrap/MU")
K_CU = HKDF(FC || FU, "KeyLessPass/wrap/CU")

W_MC = AES-256-GCM(K_MC, Kmaster)
W_MU = AES-256-GCM(K_MU, Kmaster)
W_CU = AES-256-GCM(K_CU, Kmaster)
```

每个 wrapper 必须带足够元数据用于认证解密，包括 wrapper 类型、版本、nonce、ciphertext、tag，以及 AAD 或可精确重建 AAD 的稳定字段。

## 恢复不变量

任意两个因子可以恢复同一个 `Kmaster`：

- 助记短语 + 本机：派生 `FM` 和 `FC`，解 `W_MC`。
- 助记短语 + U 盘：派生 `FM` 和 `FU`，解 `W_MU`。
- 本机 + U 盘：派生 `FC` 和 `FU`，解 `W_CU`。

任意单个因子必须失败：

- 只有助记短语：失败。
- 只有本机/本机包：失败。
- 只有 U 盘包：失败。

## U 盘因子边界

- U 盘是普通可复制存储，不是不可复制硬件密钥。
- 复制 U 盘包就是复制 U 盘因子。
- 复制出来的 U 盘因子单独不能恢复 `Kmaster`，仍需要助记短语因子或匹配的本机因子。
- U 盘包不能整体只用助记短语派生出的 key 加密。
- U 盘包是 USB 因子容器，可保存 `usbId`、`saltU`、`usbSecret` 或等价 U 盘因子材料、`W_MU`、`W_CU`、wrapper 元数据、schema version 和完整性元数据。

## 本机因子边界

- 本机包保存 `userId`、`deviceId`、`saltC`、`mnemonicSalt`、助记短语校验器、`W_MC`、可选 `W_CU`、schema version、recovery generation 和密码派生算法。
- 本机包不保存明文 `Kmaster`。
- 本机包不保存 `usbSecret`。
- `com.keylesspass.local-factor` 平台 secret 保持在 JSON payload 外，仅作为 `FC` 来源。

## 旧 schema

保存 master-key payload 或用助记短语加密整个 U 盘 payload 的旧包不满足该模型。如果无法自动迁移，应返回清晰错误：

```text
legacy factor package stores master-key payload and does not support strict pairwise-wrapper recovery; please migrate with the old mnemonic available.
```
