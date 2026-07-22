# 商业发布加固

English: [commercial-release-hardening.md](commercial-release-hardening.md)

本文定义 KeyLessPass 商业分发控制。目标不是假装本地桌面二进制不可破解；坚定攻击者可以 patch 客户端检查。成熟方案是分层控制：单设备签名授权、编译期强制授权、签名安装包、客户可识别授权包、吊销、以及把支持/更新绑定到有效授权。

## 原则

- 绝不把授权私钥或共享激活 secret 放进客户端。
- 商业客户端只嵌入厂商根公钥，不直接信任客户后台公钥。
- 商业客户端必须用 `KEYLESSPASS_REQUIRE_LICENSE=1` 构建。
- 授权包同时验证厂商授权和现场签名，并绑定设备身份私钥、`deviceKeyId` 与设备指纹。
- 复制授权包不能授权另一台电脑，除非包里包含那台电脑的 grant。
- 授权元数据必须和密码安全材料分离，不能包含助记短语、`Kmaster`、`deviceSecret`、`usbSecret`、服务密码、派生密码、CDR secret 或 wrapper key。
- 普通 U 盘仍然是可复制介质，不要宣传成不可复制硬件。

## 发布流程

1. 厂商离线签发客户授权池证书，委托客户现场公钥并列出批准的 `deviceKeyId`。
2. 客户安装授权证书后部署 `admin_backend`。
3. 使用厂商根公钥构建强制授权商业客户端：

   ```bash
   KEYLESSPASS_LICENSE_KEY_ID='keylesspass-vendor-root-2026' \
   KEYLESSPASS_LICENSE_PUBLIC_KEY_B64='<厂商根公钥>' \
   CODESIGN_IDENTITY='Developer ID Application: 你的公司 (TEAMID)' \
   tools/commercial/build_commercial_release.sh macos
   ```

4. 签名生成的 app 或安装器：
   - macOS：Developer ID Application 签名、notarization、staple；
   - Windows：Authenticode 签名二进制和安装器；
   - Linux：签名仓库元数据或发布校验和清单。

商业 macOS/Windows 构建默认拒绝缺少平台签名证书，Linux 默认要求 `KEYLESSPASS_LINUX_GPG_KEY_ID` 并生成 `SHA256SUMS.asc`。`KEYLESSPASS_ALLOW_UNSIGNED=1` 只能用于本机测试，不能发布。
5. 将安装包放入 `admin_backend/downloads/` 后，在厂商离线工作站设置 `KEYLESSPASS_RELEASE_DIRECTORY` 并执行 `cargo run -- issue-release-manifest > downloads/release-manifest.json`。后台拒绝列出未被厂商清单签名或哈希不匹配的文件。
6. 通过客户专属渠道分发，并把 release artifact、`licenseId`、`organizationId`、`keyId` 和合同记录关联保存。
7. 商业支持和更新要求客户提供有效授权状态。

## 反滥用模型

该设计提高未授权分发成本：

- 编译强制授权后，没有有效 grant 的共享二进制保持未授权。
- 复制授权包会被设备身份和指纹限制。
- 被修改的客户后台不能为厂商白名单之外的设备签发可用授权。
- 授权包包含组织、license 和 grant 标识，便于审计。
- 后续离线授权包携带吊销列表。
- 支持和更新可以要求有效授权状态。
- 签名安装器让篡改构建和官方构建可区分。

这不能阻止有人 patch 二进制并传播修改版。对应措施包括客户水印、官方校验和、签名更新通道、客户授权记录和商业合同执行。

## 构建期控制

Rust Core 从编译期环境变量读取授权公钥：

```text
KEYLESSPASS_LICENSE_KEY_ID
KEYLESSPASS_LICENSE_PUBLIC_KEY_B64
KEYLESSPASS_REQUIRE_LICENSE=1
KEYLESSPASS_BUILD_CHANNEL=commercial
KEYLESSPASS_APP_MAJOR_VERSION=1
KEYLESSPASS_MANAGED_LICENSE_FILE=<managed bundle path>
```

以上 key ID 和公钥指厂商根。厂商根轮换时，增加重叠公钥映射：

```text
KEYLESSPASS_LICENSE_TRUSTED_KEYS_JSON={"old-key-id":"old-public-key","new-key-id":"new-public-key"}
```

评估/源码构建保留非阻断默认值，便于审查。商业发布必须使用 `tools/commercial/build_commercial_release.sh` 或设置同等变量的 CI；该入口要求显式提供根 key ID 和公钥，禁止默认 ID。

运行时变量可以用于测试更严格检查，但商业构建不能依赖运行时开关来启用授权强制。

构建脚本会给不同平台设置托管授权包默认路径，例如 `/Library/Application Support/KeyLessPass`、`/etc/keylesspass` 或 `C:\ProgramData\KeyLessPass`。客户端评估授权状态时会验签并刷新本地授权存储。
