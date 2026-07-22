# KeyLessPass 授权后台

English: [README.md](README.md)

该服务用于客户内网中的应用下载、设备登记、席位分配、在线激活、离线授权和吊销。

- `/download`、`/api/downloads` 和 `/downloads/*` 公开，不需要登录；
- 管理页面和管理 API 必须使用 Admin token；
- 客户现场私钥只能签发厂商授权范围内的包；
- 商业客户端只内置厂商根公钥，不信任客户现场公钥本身；
- 厂商授权中没有列出的设备密钥，客户后台无法自行激活。

后台只处理授权元数据，不得接收助记短语、`Kmaster`、设备因子、U 盘因子、CDR secret、服务密码或派生密码。

## 信任链

```text
厂商离线根私钥（只在厂商保管）
  └─签名→ customer-entitlement.json
             ├─客户、期限、功能和版本
             ├─最大登记/并发/离线数量
             ├─客户现场公钥
             └─厂商批准的 deviceKeyId 白名单
                    └─约束→ 客户现场签发的设备授权包
```

仅设置 `maxSeats` 不能防止被修改的客户后台分叉签发。严格模式因此要求设备的 `deviceKeyId` 同时出现在厂商签名白名单中。

## 客户现场部署

要求 Docker、Docker Compose 和 OpenSSL：

```bash
cd admin_backend
./scripts/intranet_deploy.sh
```

首次运行会生成：

- 随机 Admin token；
- 客户现场 Ed25519 私钥；
- 唯一现场 key ID；
- 权限为 `600` 的 `.env`。

首次运行停在“等待厂商授权”是正常的。把脚本输出的 `KEYLESSPASS_SITE_KEY_ID`、`KEYLESSPASS_SITE_PUBLIC_KEY_B64`、购买数量和客户信息发给厂商。厂商返回：

1. `KEYLESSPASS_VENDOR_KEY_ID` 和厂商根公钥；
2. 厂商签名的 `license/customer-entitlement.json`。

把根公钥写入 `.env`，把授权文件放到指定目录，再运行脚本。成功后：

```text
下载页：http://127.0.0.1:8787/download   （无需登录）
管理页：http://127.0.0.1:8787/            （需要 Admin token）
健康检查：http://127.0.0.1:8787/healthz
```

从其他内网电脑访问时，用服务器内网 IP，并通过防火墙限制管理入口。在线激活除本机测试外必须置于 HTTPS 反向代理之后。

## 厂商签发客户授权

以下命令只允许在厂商受控工作站执行，根私钥绝不能交给客户：

```bash
cd admin_backend
export KEYLESSPASS_VENDOR_SIGNING_KEY_B64='<厂商离线根 seed>'
export KEYLESSPASS_VENDOR_KEY_ID='keylesspass-vendor-root-2026'
export KEYLESSPASS_CUSTOMER_ID='customer-001'
export KEYLESSPASS_CUSTOMER_NAME='某某公司'
export KEYLESSPASS_SITE_KEY_ID='<客户脚本输出的现场 key ID>'
export KEYLESSPASS_SITE_PUBLIC_KEY_B64='<客户脚本输出的现场公钥>'
export KEYLESSPASS_MAX_REGISTERED_DEVICES='50'
export KEYLESSPASS_MAX_CONCURRENT_DEVICES='50'
export KEYLESSPASS_MAX_OFFLINE_BORROWED='0'
export KEYLESSPASS_MAX_OFFLINE_GRACE_DAYS='14'
export KEYLESSPASS_CUSTOMER_FEATURES='desktop-client,channel:commercial'
export KEYLESSPASS_ALLOWED_MAJOR_VERSIONS='1'
export KEYLESSPASS_ENTITLEMENT_SERIAL='1'
export KEYLESSPASS_CUSTOMER_VALID_UNTIL='2027-12-31T23:59:59Z'
export KEYLESSPASS_AUTHORIZED_DEVICE_KEY_IDS=''
cargo run -- issue-customer-entitlement > customer-entitlement.json
```

空设备白名单适合首次启动和收集请求，但不能签发设备授权。管理员导入设备请求并导出设备 CSV 后，厂商核对购买数量，将获批的 `deviceKeyId` 以英文逗号分隔写入 `KEYLESSPASS_AUTHORIZED_DEVICE_KEY_IDS`，提高 `KEYLESSPASS_ENTITLEMENT_SERIAL`，重新签发文件。客户替换文件并重启：

```bash
docker compose restart keylesspass-admin
```

降低序列号、替换成更旧的授权包或加入超过购买数量的设备都会被拒绝。

## 构建商业客户端

客户端嵌入厂商根公钥，而不是管理页显示的现场公钥：

```bash
cd ..
KEYLESSPASS_LICENSE_KEY_ID='keylesspass-vendor-root-2026' \
KEYLESSPASS_LICENSE_PUBLIC_KEY_B64='<厂商根公钥>' \
CODESIGN_IDENTITY='Developer ID Application: 你的公司 (TEAMID)' \
tools/commercial/build_commercial_release.sh macos
```

根私钥和客户现场私钥都不能进入客户端。正式包还必须做平台代码签名；构建脚本会拒绝无授权配置的商业打包。

## 管理员实际操作

1. 把安装包放入 `admin_backend/downloads/`，用户从 `/download` 直接下载。
2. 管理员打开 `/`，输入 Admin token 登录。
3. 创建组织，席位、功能、版本和有效期不得超过厂商授权。
4. 客户端在“安全 → 商业授权”复制设备请求。
5. 管理员导入请求；首次新增设备需由厂商加入白名单并重新签发客户授权。
6. 厂商批准并重启后台后，管理员选择设备并签发离线授权包，或让设备使用 HTTPS 在线激活。
7. 客户端导入授权包后显示“已授权”。

在线激活只会成功激活厂商已批准的设备。组织激活码不是 Admin token；它只能为对应组织申请设备授权。

## 角色和 API

单管理员使用 `KEYLESSPASS_ADMIN_TOKEN`。多人时使用 `KEYLESSPASS_ADMIN_USERS_JSON`：

```json
[
  {"name":"admin","role":"admin","token":"至少 24 个字符"},
  {"name":"operator","role":"operator","token":"另一个长 token"},
  {"name":"auditor","role":"auditor","token":"另一个长 token"}
]
```

- `admin`：组织和吊销管理；
- `operator`：设备导入和授权签发；
- `auditor`：只读状态和导出。

管理接口使用 `Authorization: Bearer <token>`。`POST /api/activation/activate` 使用组织激活码；下载、健康检查和在线激活不接收 Admin token。

## 备份和安全边界

必须备份 `.env`、SQLite volume、当前客户授权文件和厂商侧签发记录。客户现场密钥泄露后，攻击者仍不能增加未被厂商白名单批准的设备，但可以为已批准设备重签包，因此仍需轮换现场密钥并提高 entitlement serial。

纯离线软件无法可靠判断完整虚拟机快照是否被复制，也不能即时收到吊销。高对抗客户应使用厂商在线逐设备签发、TPM/HSM 授权服务器或 CodeMeter/Sentinel 等硬件方案。
