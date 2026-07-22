# KeyLessPass 授权后台

English: [README.md](README.md)

这是 KeyLessPass 商业设备授权的内网管理后台。它用于创建组织、导入设备请求、签发离线 `.klp-license-bundle` 授权包，也支持客户端在线激活。

后台只保存商业元数据：

- 组织、方案、席位数、到期时间和功能列表；
- 桌面客户端导出的设备授权请求；
- 已签发授权包历史；
- 授权吊销记录；
- 追加式管理审计事件。

后台绝不能接收或保存助记短语、`Kmaster`、`deviceSecret`、`usbSecret`、服务密码、派生密码、CDR secret 或恢复 wrapper key。

## 一键内网部署

前提：内网服务器已安装 Docker 和 Docker Compose。

```bash
cd admin_backend
./scripts/intranet_deploy.sh
```

脚本首次运行会创建 `.env`，生成随机 Admin token 和 Ed25519 签名种子，构建容器并启动服务。

打开脚本输出的地址，默认是：

```text
http://127.0.0.1:8787
```

如果给局域网电脑访问，把 `127.0.0.1` 换成服务器 IP，并确保端口只在内网开放。

在线激活用于真实部署时必须使用 HTTPS。本机测试可以用 `http://127.0.0.1:8787`。

## 手动本地运行

```bash
cd admin_backend
cargo run -- generate-key
export KEYLESSPASS_ADMIN_TOKEN="$(openssl rand -hex 32)"
export KEYLESSPASS_LICENSE_SIGNING_KEY_B64="<上一步生成的 seed>"
export KEYLESSPASS_LICENSE_KEY_ID="keylesspass-license-2026-q3"
export KEYLESSPASS_ADMIN_DB="./keylesspass-admin.sqlite3"
cargo run
```

## 客户端公钥

后台用 Ed25519 私钥签名授权包，桌面客户端只嵌入公钥验签。

部署后台后：

1. 登录管理页。
2. 复制 `publicKeyB64`。
3. 构建商业客户端时设置同一个 `KEYLESSPASS_LICENSE_KEY_ID` 和该公钥。
4. 不要把私钥或签名 seed 放进客户端、浏览器前端或公开仓库。

## 离线授权流程

1. 在后台创建组织，设置席位数、方案、功能和有效期。
2. 在每台桌面客户端点击“复制设备请求”。
3. 在后台 `Import device request` 粘贴请求并分配组织。
4. 在设备表勾选设备。
5. 点击 `Issue bundle` 签发授权包。
6. 复制或下载授权包。
7. 回到桌面客户端点击“导入授权包”。

## 在线激活流程

1. 在后台创建组织。
2. 在 `Organization activation` 中复制该组织激活码。
3. 把 HTTPS 授权服务地址和激活码发给用户。
4. 用户在桌面客户端点击“在线激活”。
5. 用户输入服务地址、激活码和席位标签。
6. 后台自动占用一个席位并返回绑定该设备的签名授权包。

本机测试可用：

```text
http://127.0.0.1:8787
```

普通内网 IP 的 HTTP 地址会被客户端拒绝；真实环境请上 HTTPS。

## 角色和审计

`KEYLESSPASS_ADMIN_TOKEN` 是单管理员 token。需要多人账号时，设置 `KEYLESSPASS_ADMIN_USERS_JSON`：

```json
[
  {"name":"admin","role":"admin","token":"..."},
  {"name":"license-operator","role":"operator","token":"..."},
  {"name":"audit-reader","role":"auditor","token":"..."}
]
```

- `admin`：创建组织和吊销授权；
- `operator`：导入设备、批量 CSV、签发授权包、查看激活码；
- `auditor`：查看状态、导出设备和审计 CSV。

审计记录包含操作者、角色、动作、目标和时间，不记录密码秘密或管理员 token。

## CSV 批量导入

CSV 必须是 UTF-8，并包含：

```csv
requestJson,organizationId,seatLabel
"{""schemaVersion"":1,""requestId"":""req-..."",...}",org-acme,Finance laptop 01
```

`requestJson` 是 JSON，里面有逗号和双引号。请用标准 CSV 工具生成，不要手写拼字符串。

## 签名密钥轮换

1. 运行 `cargo run -- generate-key` 生成新 seed。
2. 使用新的 `KEYLESSPASS_LICENSE_KEY_ID` 部署后台。
3. 客户端构建时用 `KEYLESSPASS_LICENSE_TRUSTED_KEYS_JSON` 同时信任旧公钥和新公钥。
4. 所有受支持客户端都信任新公钥后，再切后台签名 seed。
5. 私钥永远只留在后台服务器。

## API

管理接口需要：

```text
Authorization: Bearer <KEYLESSPASS_ADMIN_TOKEN>
```

主要接口：

- `GET /api/status`
- `GET /api/snapshot`
- `GET /api/organizations`
- `POST /api/organizations`
- `POST /api/device-requests/import`
- `POST /api/device-requests/import.csv`
- `GET /api/devices`
- `GET /api/devices.csv`
- `POST /api/licenses/issue`
- `POST /api/grants/{grantId}/revoke`
- `GET /api/audit.csv`

`POST /api/activation/activate` 使用组织激活码，不使用 Admin token。`GET /healthz` 不需要 token。
