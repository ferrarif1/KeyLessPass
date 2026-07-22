# KeyLessPass 桌面端设计

English: [DESIGN.md](DESIGN.md)

## 架构

KeyLessPass 是桌面客户端，不是 Web 应用：

- Flutter Desktop 渲染界面；
- Rust Core 实现密码学、CDR 存储、U 盘因子包、恢复和平台因子 provider；
- SQLite 只保存 CDR 元数据，不保存派生密码；
- Flutter 通过很小的 C ABI JSON FFI 调用 Rust。

没有 Web server、云同步、浏览器自动填充、浏览器插件或 WebView 主界面。

## 派生边界

这些展示字段可以改，且不影响派生密码：

- `displayName`
- `serviceHint`
- `accountHint`

这些稳定字段参与派生：

- `recordSeq`
- `recordId`
- `version`
- `salt`
- `encodingDescriptor`

如果要改密码规则，必须创建新版本，这就是密码轮换。

## 正常使用

日常派生只需要：

1. 当前电脑；
2. 助记短语。

Rust Core 通过 `W_MC` 恢复 `Kmaster`。U 盘日常可以离线保存，只在初始化、恢复、更换因子或重置助记短语时使用。

## 恢复路径

- 助记短语 + 本机：解 `W_MC`，可重建 U 盘包。
- 助记短语 + U 盘：解 `W_MU`，可重建本机因子。
- 本机 + U 盘：解 `W_CU`，可在不知道旧助记短语时重置助记短语。

本机和 U 盘 payload 都不保存明文 `Kmaster`。U 盘是普通可复制存储，应按“可复制因子容器”理解。

## 平台 provider

Rust provider 接口：

```rust
pub trait PlatformFactorProvider {
    fn platform_name(&self) -> String;
    fn get_or_create_device_id(&self) -> Result<String>;
    fn get_or_create_device_secret(&self) -> Result<SecretBytes>;
    fn protect_local_package(&self, plaintext: &[u8]) -> Result<Vec<u8>>;
    fn unprotect_local_package(&self, protected: &[u8]) -> Result<Vec<u8>>;
}
```

macOS 优先使用 Keychain。Windows 预留 DPAPI 路径。Linux/UOS/麒麟第一版使用本地 AEAD 包保护和文件权限；如果安全性降低，UI 会显示对应状态。

## FFI

Rust 导出：

```c
char *keylesspass_ffi_json(const char *request_json);
void keylesspass_ffi_free(char *response_json);
```

请求：

```json
{"op":"derivePassword","payload":{...}}
```

成功响应：

```json
{"ok":true,"data":{...}}
```

失败响应：

```json
{"ok":false,"error":"safe user-facing error"}
```

派生和恢复失败在 FFI 边界会转成安全的用户提示，不暴露内部秘密。
