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

下载应用不需要登录：访问 `http(s)://服务器/download`。推荐模式下终端会自动找到服务器、登记并领取已批准授权；部署维护 token 只给客户 IT 导入/导出厂商文件，不给终端用户。

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
/           批量批准和故障维护，需要部署维护 token
/healthz    健康检查，不需要 token
```

脚本默认把 TCP 8787 和 UDP 8788 开放到内网，并自动生成 `KEYLESSPASS_PUBLIC_BASE_URL`。多网卡或 NAT 环境要在 `.env` 写成终端实际能访问的地址。不要把端口暴露到互联网。

用户点击 `/download` 中任一安装包时，页面会同时下载动态生成的 `keylesspass-client-config.json`。客户端自动读取下载目录中最新的同名配置，UDP 只在配置缺失时兜底。该文件只定位服务器；授权仍必须通过客户端内置厂商根公钥的 Ed25519 验签。

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

## 6. 推荐：纯内网自动批量批准

1. 用户从 `/download` 点击安装包；页面同时下载客户端和服务器配置。安装并启动一次，不输入任何授权信息。
2. 客户端读取下载目录或受管目录中的最新配置；只有配置丢失时才用 UDP 8788 发现服务器。
3. 客户端向 `/api/automatic/activate` 提交带设备私钥证明的请求。未获批准时服务端只登记并返回等待状态。
4. 客户 IT 用部署维护 token 打开 `/`，点击“Export batch request”，把 JSON 发给厂商。
5. 厂商核对合同数量，在离线工作站执行：

```bash
export KEYLESSPASS_VENDOR_SIGNING_KEY_B64='<厂商根私钥>'
export KEYLESSPASS_VENDOR_KEY_ID='keylesspass-vendor-root-2026'
export KEYLESSPASS_DEVICE_BATCH_REQUEST_FILE='<客户批量请求 JSON>'
export KEYLESSPASS_CUSTOMER_VALID_UNTIL='2027-12-31T23:59:59Z'
cargo run -- issue-customer-entitlement > customer-entitlement.json
```

6. 客户在同一页面导入返回文件。服务自动重启。
7. 客户端最多 20 秒后再次请求，取得只属于本机的签名授权并自动导入。

批量文件内嵌上一份厂商签名 entitlement；签发命令先验签并继承其中的合同额度、客户、现场公钥、期限和功能。客户修改 JSON 中的额度会被拒绝。请求数超过已签额度时命令拒绝签发；扩容/续费必须由厂商显式覆盖合同参数。若收集设备多于购买数，厂商必须设置 `KEYLESSPASS_AUTHORIZED_DEVICE_KEY_IDS` 明确选择获批设备。

## 7. 自动附带配置与受管配置位置

| 平台 | 路径 |
| --- | --- |
| macOS | `/Library/Application Support/KeyLessPass/keylesspass-client-config.json` 或用户目录同名位置 |
| Windows | `%PROGRAMDATA%\KeyLessPass\keylesspass-client-config.json` 或 `%APPDATA%` 同名路径 |
| Linux | `/etc/keylesspass/client-config.json` 或 `~/.config/keylesspass/client-config.json` |
| 通用 | 可执行文件旁、当前工作目录，或 `KEYLESSPASS_CLIENT_CONFIG` 指定位置 |

TCP 8787 仍必须允许终端访问；如果连 TCP 也被禁止，就不存在客户端访问内网授权服务的网络路径，必须由网络管理员放行或通过 MDM 下发离线授权包。

## 8. 备用：手动批准设备

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

## 9. 备用：手动在线激活

在线激活只适用于已经被厂商批准的设备：

1. 管理员在后台复制组织激活码。
2. 用户在客户端点击“在线激活”。
3. 输入 HTTPS 服务地址、组织激活码和席位标签。
4. 服务端验证设备证明、厂商白名单、组织权限和剩余席位，在 SQLite `BEGIN IMMEDIATE` 事务中占用席位并签发单设备包。

本机测试可使用 `http://127.0.0.1:8787`；普通内网地址必须使用 HTTPS。点按钮后状态不变时，先查看弹出的错误；常见原因是设备尚未加入厂商白名单、激活码错误或席位已满。

## 10. 备用：手动导入离线授权

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

## 11. 席位、吊销和换机

- 席位在独立 `seat_allocations` 表中事务分配；过期席位自动变为 expired。
- 吊销一个设备会吊销该设备全部活动 grant，并禁止使用旧身份重新在线激活。
- 离线吊销只有在终端收到更新包或原授权到期后生效。
- 换机必须生成新设备密钥，由厂商把新 key ID 加入更高序列授权；不要复制旧应用目录。
- 清除本机授权不会删除密码记录，但防回滚安全状态会保留。

## 12. 故障排查

| 现象 | 原因和处理 |
| --- | --- |
| 无法启动后台 | 检查根公钥、现场私钥和 `license/customer-entitlement.json` 是否匹配 |
| 客户端找不到服务器 | 确认浏览器同时下载了配置，或由 IT 下发配置；确认 TCP 8787 可达和 `KEYLESSPASS_PUBLIC_BASE_URL` 正确。UDP 8788 只是兜底 |
| 一直等待批准 | 在维护页导出最新批量请求；确认导入 entitlement 的序列号更高且包含该设备 key ID |
| 在线激活无变化 | 设备未获厂商批准、HTTPS/激活码错误或席位已满；查看客户端提示和后台日志 |
| “exceeds vendor entitlement” | 组织或设备超出厂商签名范围，不能在客户后台自行扩大 |
| “not for this device” | 包不属于当前设备，重新复制本机 schema v2 请求 |
| “rollback detected” | 导入了旧 entitlement serial 或旧 bundle，使用最新文件 |
| 未授权仍可使用 | 运行的是开发/评估构建，重新使用商业构建入口 |

## 13. 不能消除的边界

本实现显著提高普通复制、改后台超发和回滚旧授权的成本，但本地软件不是不可破解 DRM：拥有机器控制权的人仍可能 patch 客户端；完整复制离线虚拟机、设备密钥和所有受保护系统状态也无法由纯软件绝对识别。

需要严格控制浮动并发或对抗恶意客户时，选择厂商在线逐设备/短租约签发、TPM/HSM 授权服务器，或 Sentinel LDK、CodeMeter 等成熟硬件授权方案。不要把普通 U 盘当作不可复制加密狗。
