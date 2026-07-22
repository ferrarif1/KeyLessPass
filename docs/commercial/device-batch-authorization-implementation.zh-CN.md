# 设备授权实现与项目使用指南

English: [device-batch-authorization-implementation.md](device-batch-authorization-implementation.md)

本文是商业版的主操作手册，说明厂商、客户管理员和终端用户分别要做什么。

安全结论和剩余边界见 [商业授权安全审计报告](authorization-security-audit.zh-CN.md)。

## 1. 项目组件

```text
admin_backend/       客户内网授权服务、公开下载页、管理后台
flutter_app/         macOS/Windows/Linux 桌面客户端
rust_core/           设备身份、授权验签、权限守卫和密码核心
tools/commercial/    强制授权商业构建入口
packaging/           安装包和平台签名脚本
```

下载应用不需要登录：访问 `http(s)://服务器/download`。创建组织、导入设备、签发和吊销需要打开服务器根地址并输入 Admin token。

## 2. 商业授权怎样防止客户超量

客户端验证两层 Ed25519 签名：

1. 内置厂商根公钥，验证 `customer-entitlement.json`；
2. 从该授权中取得客户现场公钥，再验证现场签发的设备包。

厂商授权同时限制客户、期限、功能、版本、最大数量和批准的 `deviceKeyId`。客户即使控制现场服务端，也不能为白名单之外的新设备制作客户端可接受的授权包。

每台终端首次运行还会生成设备身份私钥。授权绑定其公钥、key ID 和受平台保护的设备指纹。Windows 使用 DPAPI 保护软件密钥；macOS 使用 Keychain 保护。复制普通授权 JSON、公开 UUID 或数据库不能直接克隆授权。

## 3. 厂商首次准备

厂商离线生成并保管根密钥。根私钥不得发送给客户、写入客户服务器或放进客户端：

```bash
cd admin_backend
cargo run -- generate-key
```

安全保存输出的 seed，把根公钥和根 key ID用于商业客户端构建。正式环境建议把根签发工作站离线，使用独立签发记录和双人复核。

## 4. 客户部署内网平台

```bash
cd admin_backend
./scripts/intranet_deploy.sh
```

脚本首次运行生成 Admin token 和客户现场密钥，然后输出现场 key ID 与公钥并暂停。客户把这两个值、公司名称、合同数量和有效期发给厂商。

厂商先签发一个可为空设备白名单的客户授权。客户安装：

```text
admin_backend/.env
  KEYLESSPASS_VENDOR_KEY_ID=<厂商根 key ID>
  KEYLESSPASS_VENDOR_PUBLIC_KEY_B64=<厂商根公钥>

admin_backend/license/customer-entitlement.json
  <厂商返回的完整 JSON>
```

再次运行 `./scripts/intranet_deploy.sh`。访问：

```text
/download   任何内网用户可下载应用，不需要 token
/           管理后台，需要 Admin token
/healthz    健康检查，不需要 token
```

要让其他内网电脑访问，调整 Compose 的端口绑定或使用 HTTPS 反向代理；不要直接把管理端口暴露到互联网。

## 5. 构建和放置客户端

客户端必须嵌入厂商根公钥：

```bash
KEYLESSPASS_LICENSE_KEY_ID='keylesspass-vendor-root-2026' \
KEYLESSPASS_LICENSE_PUBLIC_KEY_B64='<厂商根公钥>' \
CODESIGN_IDENTITY='Developer ID Application: 你的公司 (TEAMID)' \
tools/commercial/build_commercial_release.sh macos
```

Linux 使用参数 `linux` 并设置 `KEYLESSPASS_LINUX_GPG_KEY_ID`。Windows 在 Windows 主机按 `packaging/windows/build_installer.ps1` 的提示设置相同根 key ID、公钥和 Authenticode 证书指纹。

将签名后的 DMG、EXE/MSIX 或 Linux 包复制到 `admin_backend/downloads/`。平台自动计算 SHA-256，并在 `/download` 提供下载。

商业构建必须满足：

- `KEYLESSPASS_REQUIRE_LICENSE=1`；
- 非 evaluation 渠道；
- 厂商根公钥有效；
- macOS/Windows 正式包完成平台代码签名。

## 6. 首次批准设备

### 终端用户

1. 安装并打开 KeyLessPass。
2. 进入“安全 → 商业授权”。
3. 点击“复制设备请求”，填写席位标签。
4. 把完整 JSON 交给客户管理员。

请求为 schema v2，包含设备公钥、`deviceKeyId` 和持有私钥的签名证明，不包含任何密码秘密。

### 客户管理员

1. 用 Admin token 登录后台。
2. 创建组织；席位、功能、版本和期限不得超过厂商授权。
3. 导入设备请求，可单台粘贴或使用标准 UTF-8 CSV。
4. 导出设备 CSV，把需要购买/新增的 `deviceKeyId` 发给厂商。

### 厂商

核对合同数量，把批准的 key ID 以英文逗号分隔写入 `KEYLESSPASS_AUTHORIZED_DEVICE_KEY_IDS`，将 `KEYLESSPASS_ENTITLEMENT_SERIAL` 增加 1，再执行：

```bash
cargo run -- issue-customer-entitlement > customer-entitlement.json
```

客户替换文件并执行：

```bash
cd admin_backend
docker compose restart keylesspass-admin
```

旧序列授权和旧签发时间包会被客户端防回滚状态拒绝。

## 7. 在线激活

在线激活只适用于已经被厂商批准的设备：

1. 管理员在后台复制组织激活码。
2. 用户在客户端点击“在线激活”。
3. 输入 HTTPS 服务地址、组织激活码和席位标签。
4. 服务端验证设备证明、厂商白名单、组织权限和剩余席位，在 SQLite `BEGIN IMMEDIATE` 事务中占用席位并签发单设备包。

本机测试可使用 `http://127.0.0.1:8787`；普通内网地址必须使用 HTTPS。点按钮后状态不变时，先查看弹出的错误；常见原因是设备尚未加入厂商白名单、激活码错误或席位已满。

## 8. 离线授权

1. 管理员导入并完成厂商审批。
2. 在后台选择同一组织下的设备。
3. 点击签发授权包并下载完整 JSON。
4. 用户在“安全 → 商业授权 → 导入授权包”粘贴或选择该包。
5. 确认状态为“已授权”。

授权包可由 MDM 放到：

| 平台 | 路径 |
| --- | --- |
| macOS | `/Library/Application Support/KeyLessPass/license-bundle.json` |
| Windows | `C:\ProgramData\KeyLessPass\license-bundle.json` |
| Linux | `/etc/keylesspass/license-bundle.json` |

## 9. 席位、吊销和换机

- 席位在独立 `seat_allocations` 表中事务分配；过期席位自动变为 expired。
- 吊销一个设备会吊销该设备全部活动 grant，并禁止使用旧身份重新在线激活。
- 离线吊销只有在终端收到更新包或原授权到期后生效。
- 换机必须生成新设备密钥，由厂商把新 key ID 加入更高序列授权；不要复制旧应用目录。
- 清除本机授权不会删除密码记录，但防回滚安全状态会保留。

## 10. 故障排查

| 现象 | 原因和处理 |
| --- | --- |
| 无法启动后台 | 检查根公钥、现场私钥和 `license/customer-entitlement.json` 是否匹配 |
| 在线激活无变化 | 设备未获厂商批准、HTTPS/激活码错误或席位已满；查看客户端提示和后台日志 |
| “exceeds vendor entitlement” | 组织或设备超出厂商签名范围，不能在客户后台自行扩大 |
| “not for this device” | 包不属于当前设备，重新复制本机 schema v2 请求 |
| “rollback detected” | 导入了旧 entitlement serial 或旧 bundle，使用最新文件 |
| 未授权仍可使用 | 运行的是开发/评估构建，重新使用商业构建入口 |

## 11. 不能消除的边界

本实现显著提高普通复制、改后台超发和回滚旧授权的成本，但本地软件不是不可破解 DRM：拥有机器控制权的人仍可能 patch 客户端；完整复制离线虚拟机、设备密钥和所有受保护系统状态也无法由纯软件绝对识别。

需要严格控制浮动并发或对抗恶意客户时，选择厂商在线逐设备/短租约签发、TPM/HSM 授权服务器，或 Sentinel LDK、CodeMeter 等成熟硬件授权方案。不要把普通 U 盘当作不可复制加密狗。
