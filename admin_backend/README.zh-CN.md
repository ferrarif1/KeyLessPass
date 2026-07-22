# KeyLessPass 纯内网授权服务

English: [README.md](README.md)

推荐使用“自动收集 + 厂商一次离线批准”模式。终端用户只下载、安装、启动应用，不需要服务器地址、激活码或 Admin token。

## 客户怎么做

### 1. 一键启动

要求 Docker、Docker Compose、OpenSSL：

```bash
cd admin_backend
./scripts/intranet_deploy.sh
```

首次运行生成现场 Ed25519 密钥和部署维护 token，并输出现场 key ID、公钥。客户把它们连同公司名称、购买终端数、有效期发给厂商。厂商返回根公钥和初始 `license/customer-entitlement.json` 后，再运行同一脚本。正式销售交付时，厂商应预先完成这一步，把每个客户独立的现场密钥配置和初始 entitlement 放进交付包；客户收到后只需运行一次脚本。若交付配置没有有效维护 token，脚本会在客户机器本地生成，厂商不会掌握该 token。

脚本自动检测服务器内网 IP。多网卡、NAT、固定域名或反向代理环境请在 `.env` 明确设置：

```dotenv
KEYLESSPASS_PUBLIC_BASE_URL=http://10.20.30.40:8787
```

### 2. 用户下载

访问脚本输出的 `http://服务器IP:8787/download`，不需要登录。把正式安装包放到 `admin_backend/downloads/` 即可出现在下载页。

用户点击任一客户端安装包时，下载页会同时下载本服务器动态生成的 `keylesspass-client-config.json`。客户端按以下顺序寻找服务器：

1. 受管配置目录或用户下载目录中的 `keylesspass-client-config.json`（包括浏览器重名后的最新文件）；
2. 配置缺失时才用 UDP 端口 `8788` 广播发现；
3. 都失败时保持未授权并等待 IT 修复，不要求用户手输激活码。

正常用户不需要移动文件。浏览器若阻止一次下载多个文件，请允许该内网站点下载；批量部署时，IT 也可把配置放到任一固定位置：

| 平台 | 配置位置 |
| --- | --- |
| macOS | `/Library/Application Support/KeyLessPass/keylesspass-client-config.json` 或用户目录同名路径 |
| Windows | `%PROGRAMDATA%\KeyLessPass\keylesspass-client-config.json` 或 `%APPDATA%` 同名路径 |
| Linux | `/etc/keylesspass/client-config.json` 或 `~/.config/keylesspass/client-config.json` |
| 通用 | 应用可执行文件旁，或用 `KEYLESSPASS_CLIENT_CONFIG` 指定绝对路径 |

该配置只告诉客户端服务器地址，不授予权限。最终授权仍必须通过客户端内置厂商根公钥验证 Ed25519 信任链，因此伪造配置不能生成免费授权。

### 3. 收集并批准设备

所有目标终端启动一次后，会自动登记并显示等待厂商批准。客户维护人员打开服务器根地址，输入部署脚本生成的 token：

1. 点击“Export batch request”；
2. 把 `keylesspass-offline-approval-request.json` 发给厂商；
3. 收到更高序列号的 `customer-entitlement.json` 后，点击“Import vendor approval”；
4. 服务自动重启；客户端轮询后自动变成已授权。

这个 token 只用于离线批准文件的导入、导出和故障维护。普通下载、设备登记、自动授权都不发送它。

## 厂商怎么批准

根私钥只保存在厂商受控离线工作站：

```bash
cd admin_backend
export KEYLESSPASS_VENDOR_SIGNING_KEY_B64='<厂商根私钥 seed>'
export KEYLESSPASS_VENDOR_KEY_ID='keylesspass-vendor-root-2026'
export KEYLESSPASS_DEVICE_BATCH_REQUEST_FILE='/path/keylesspass-offline-approval-request.json'
export KEYLESSPASS_CUSTOMER_VALID_UNTIL='2027-12-31T23:59:59Z'
cargo run -- issue-customer-entitlement > customer-entitlement.json
```

批量文件携带上一份厂商签名 entitlement。命令先验签，并从中读取客户、现场公钥、当前序列号、购买额度、期限和功能；客户篡改批量文件中的额度会被拒绝。命令把序列号加 1，请求设备超过已签额度时签发失败；续费或扩容必须由厂商显式设置新合同参数。如需只批准一部分设备，设置 `KEYLESSPASS_AUTHORIZED_DEVICE_KEY_IDS` 为核准后的逗号分隔列表。

初始授权可使用传统环境变量签发，并把 `KEYLESSPASS_AUTHORIZED_DEVICE_KEY_IDS` 留空。空白名单只能启动服务器和收集设备，不能签发任何终端授权。

## 端口和防火墙

| 端口 | 用途 | 是否必须 |
| --- | --- | --- |
| TCP 8787 | 下载、配置、自动登记和维护页面 | 必须 |
| UDP 8788 | 配置丢失时的兜底发现 | 可选 |

只向客户内网开放。跨不可信网段时用 HTTPS 反向代理，并限制根路径及 `/api/offline-approval/*` 的访问；不要暴露到互联网。

## 安全边界

- 客户现场私钥只能在厂商签名额度及设备白名单内签发；
- 商业客户端只内置厂商根公钥，不直接信任现场公钥；
- 每个设备请求必须证明持有其 Ed25519 设备私钥；
- 席位分配使用数据库事务，终端授权与设备 key、指纹、版本和期限绑定；
- 服务只处理授权元数据，不得接收助记词、`Kmaster`、因子秘密、服务密码或派生密码。

必须备份 `.env`、SQLite volume、当前 entitlement 和厂商签发台账。纯离线软件不能即时吊销已发出的长期包，也不能绝对识别完整虚拟机克隆；高对抗场景应缩短有效期或改用厂商在线/TPM/HSM/硬件授权。
