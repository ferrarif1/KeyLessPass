# 设备批量授权实现与使用指南

[English implementation notes](device-batch-authorization-implementation.md)

本文说明如何实际使用 KeyLessPass 的商业设备授权功能。推荐按本文顺序完成：部署授权后台、构建商业客户端、创建组织，然后选择在线激活或离线批量授权。

> 商业授权只控制产品使用权，不参与密码派生，也不会接触助记短语、主密钥、设备因子、U 盘因子、CDR secret、服务密码或派生密码。

## 1. 项目中各部分的作用

```text
KeyLessPass/
├── admin_backend/                         # 内网授权后台：组织、席位、签发和吊销
├── flutter_app/                           # 桌面客户端：激活、导入授权包、查看状态
├── rust_core/                             # 本地验签、设备绑定和授权强制检查
├── tools/commercial/
│   └── build_commercial_release.sh        # macOS/Linux 商业构建入口
└── packaging/                             # 各平台安装包构建脚本
```

授权后台持有 Ed25519 签名私钥，商业客户端只嵌入公钥。客户端在本机验证授权包，并用 `commercialDeviceId + deviceFingerprint` 确认授权是否属于当前设备。

## 2. 完整使用流程

首次部署按以下顺序操作：

1. 在可信内网主机部署 `admin_backend`。
2. 登录后台，复制 `publicKeyB64`。
3. 用该公钥构建强制授权的商业客户端。
4. 在后台创建组织，设置席位数、有效期和功能。
5. 将商业客户端安装到目标电脑。
6. 选择在线激活，或收集设备请求后批量签发离线授权包。
7. 在客户端“设置 → 商业授权”确认状态为“已授权”。

已有源码只做开发调试时，可直接按根目录 [README.zh-CN.md](../../README.zh-CN.md) 的“快速开始”运行。只有通过商业构建入口编译的客户端才会强制检查授权。

## 3. 部署授权后台

### 3.1 环境要求

- 一台由组织控制的内网主机；
- Docker；
- Docker Compose；
- OpenSSL；
- 正式提供在线激活时，还需要 HTTPS 反向代理和访问频率限制。

### 3.2 一键启动

在仓库根目录执行：

```bash
cd admin_backend
./scripts/intranet_deploy.sh
```

首次运行会：

- 创建权限为 `600` 的 `admin_backend/.env`；
- 生成管理员 token；
- 生成 Ed25519 签名种子；
- 构建并启动 Docker Compose 服务；
- 使用 Docker volume 保存 SQLite 数据库。

终端会输出类似内容：

```text
URL:   http://127.0.0.1:8787
Token: <随机管理员 token>
```

打开该地址，将 token 粘贴到 **Admin token**，点击 **Connect**。如果从另一台内网电脑访问，把 `127.0.0.1` 换成服务器的内网 IP。

检查服务状态：

```bash
curl http://127.0.0.1:8787/healthz
docker compose ps
```

停止和再次启动：

```bash
cd admin_backend
docker compose stop
docker compose up -d
```

不要执行 `docker compose down -v`，除非明确要删除授权数据库。

### 3.3 必须备份的内容

至少备份：

- `admin_backend/.env` 中的签名种子和管理员配置；
- Docker volume `keylesspass-admin-data` 中的 SQLite 数据库；
- 当前使用的 `KEYLESSPASS_LICENSE_KEY_ID`；
- 已发布客户端对应的公钥和版本记录。

签名种子不能进入客户端、浏览器前端、公开仓库或普通日志。丢失私钥后，旧客户端仍能验证已有授权包，但无法再用同一密钥签发新包。

## 4. 构建商业客户端

### 4.1 获取验证公钥

登录授权后台，在 **Signing key** 卡片中复制 `publicKeyB64`。同时记下页面显示的 key ID，默认是：

```text
keylesspass-license-2026-q3
```

后台的 `KEYLESSPASS_LICENSE_KEY_ID` 必须与客户端构建使用的 ID 一致。

### 4.2 macOS

先准备 Flutter、Rust、Xcode，以及两个 Rust 架构目标：

```bash
rustup target add x86_64-apple-darwin aarch64-apple-darwin
```

在仓库根目录构建：

```bash
KEYLESSPASS_LICENSE_PUBLIC_KEY_B64='<后台复制的 publicKeyB64>' \
KEYLESSPASS_LICENSE_KEY_ID='keylesspass-license-2026-q3' \
tools/commercial/build_commercial_release.sh macos
```

默认输出：

```text
flutter_app/build/macos/Build/Products/Release/KeyLessPass.app
dist/macos/KeyLessPass-<版本>-macos.dmg
```

脚本会在编译期设置：

```text
KEYLESSPASS_REQUIRE_LICENSE=1
KEYLESSPASS_BUILD_CHANNEL=commercial
KEYLESSPASS_APP_MAJOR_VERSION=1
```

公开分发前，仍需使用 Developer ID 签名、notarize 并 staple DMG；默认的临时签名只适合内部验证。

### 4.3 Linux

必须在 Linux 主机执行：

```bash
KEYLESSPASS_LICENSE_PUBLIC_KEY_B64='<后台复制的 publicKeyB64>' \
KEYLESSPASS_LICENSE_KEY_ID='keylesspass-license-2026-q3' \
tools/commercial/build_commercial_release.sh linux
```

主要输出位于：

```text
flutter_app/build/linux/x64/release/bundle/
dist/linux/
```

### 4.4 Windows

必须在已安装 Flutter、Rust、Visual Studio Build Tools 和 Inno Setup 的 Windows 主机上使用 PowerShell：

```powershell
$env:KEYLESSPASS_REQUIRE_LICENSE="1"
$env:KEYLESSPASS_BUILD_CHANNEL="commercial"
$env:KEYLESSPASS_LICENSE_KEY_ID="keylesspass-license-2026-q3"
$env:KEYLESSPASS_LICENSE_PUBLIC_KEY_B64="<后台复制的 publicKeyB64>"
$env:KEYLESSPASS_APP_MAJOR_VERSION="1"
$env:KEYLESSPASS_MANAGED_LICENSE_FILE="C:\ProgramData\KeyLessPass\license-bundle.json"
packaging\windows\build_installer.ps1
```

正式分发前使用组织的 Authenticode 证书签名安装包。

### 4.5 验证构建是否正确

安装并启动客户端，打开 **设置 → 商业授权**：

- 未导入授权前应显示“未授权”；
- 初始化、添加记录、派生、轮换、恢复和 U 盘 CDR 同步应被授权守卫拦截；
- “在线激活”“复制设备请求”“导入授权包”和“清除本机授权”仍然可用。

如果未授权时密码功能仍可使用，通常说明运行的是普通评估构建，而不是通过商业构建入口生成的客户端。

## 5. 在后台创建组织

登录管理页，在 **Create organization** 填写：

| 字段 | 用途 | 建议值 |
| --- | --- | --- |
| Organization name | 客户或部门名称 | `某某集团` |
| Plan | 套餐标识 | `enterprise` |
| Max seats | 可激活的最大设备数 | 按合同席位数填写 |
| Valid days | 授权有效天数 | `365` |
| Offline grace days | 到期后的离线宽限天数 | `14` |
| Features | 客户端允许的功能和渠道 | `desktop-client, channel:commercial` |

点击 **Create organization**。系统会自动生成组织 ID、license ID 和激活码。当前管理页面创建的组织默认允许应用主版本 `1`。

`desktop-client` 是桌面密码功能所需权益；商业构建还要求与构建渠道匹配的 `channel:commercial`。不要删除这两个值。

## 6. 在线激活

在线激活适合目标电脑能够访问授权服务的场景。

### 6.1 服务端准备

1. 把授权后台放在组织的 HTTPS 反向代理之后。
2. 对 `/api/activation/activate` 设置速率限制和访问日志脱敏。
3. 在后台 **Organization activation** 中复制目标组织的激活码。
4. 通过组织认可的安全渠道把服务器地址和激活码交给用户。

客户端只接受：

- `https://` 地址；
- 或同一台电脑上的 `http://localhost`、`http://127.0.0.1`、`http://[::1]`。

普通内网 IP 的明文 HTTP 地址会被拒绝。

### 6.2 客户端操作

1. 打开 **设置 → 商业授权**。
2. 点击 **在线激活**。
3. 输入激活服务器地址，例如 `https://license.example.com`。
4. 输入组织激活码。
5. 填写便于管理员识别的席位名称，例如“财务部笔记本 01”。
6. 再次点击 **在线激活**。

客户端会自动生成设备请求，服务端占用一个席位并返回只绑定当前设备的签名授权包。成功后状态应变为“已授权”。

## 7. 离线单台或批量授权

离线授权适合隔离网、不能访问授权服务器或需要管理员统一审批的场景。

### 7.1 从客户端导出设备请求

在每台目标电脑上：

1. 打开 **设置 → 商业授权**。
2. 点击 **复制设备请求**。
3. 可填写组织 ID 和席位名称；不知道组织 ID 时可以留空，由管理员导入时选择。
4. 点击复制，将 JSON 保存到受控工单、文本文件或标准 CSV 中。

设备请求只包含授权标识、平台、应用版本、构建渠道和设备指纹，不包含任何密码秘密。

### 7.2 在后台导入设备

单台导入：

1. 在 **Import device request** 选择组织。
2. 填写席位名称。
3. 将完整设备请求 JSON 粘贴到 **Request JSON**。
4. 点击 **Import request**。

批量导入时，在 **Devices** 区域选择 UTF-8 CSV。表头必须是：

```csv
requestJson,organizationId,seatLabel
"{""schemaVersion"":1,""requestId"":""req-..."",...}",org-acme,财务部笔记本01
```

`requestJson` 内含逗号和双引号，必须使用合规 CSV 工具进行转义，不要简单用字符串拼接生成。

### 7.3 签发授权包

1. 在 **Devices** 表格勾选需要授权的设备。
2. 在 **Issue license bundle** 选择同一个组织。
3. 如无特殊需要，`Valid days override` 留空，沿用组织有效期。
4. 点击 **Issue bundle**。
5. 点击 **Copy bundle** 或 **Download bundle**。

一个授权包可以包含多台设备的 grant，但每台客户端只会接受与自身 `commercialDeviceId + deviceFingerprint` 匹配的授权书。复制同一授权包到未登记设备不会获得授权。

### 7.4 在客户端导入

1. 用文本编辑器打开下载的 `.klp-license-bundle` 文件并复制完整 JSON，或直接使用后台的 **Copy bundle**。
2. 打开客户端 **设置 → 商业授权**。
3. 点击 **导入授权包**。
4. 粘贴完整 JSON 并确认导入。
5. 检查状态、组织、席位、授权 ID、设备授权 ID 和有效期。

状态显示“已授权”后即可正常使用受保护功能。

## 8. 使用 MDM 自动下发授权包

商业构建会监听平台托管路径：

| 平台 | 默认路径 |
| --- | --- |
| macOS | `/Library/Application Support/KeyLessPass/license-bundle.json` |
| Windows | `C:\ProgramData\KeyLessPass\license-bundle.json` |
| Linux | `/etc/keylesspass/license-bundle.json` |

MDM 或配置管理系统可把完整授权包 JSON 写入对应路径。客户端读取授权状态时会自动导入并验签。

注意：授权包仍然按设备绑定。若给多台设备下发同一个文件，该文件必须包含这些设备各自的 grant；文件权限应允许客户端读取，但不应允许普通用户随意覆盖。

## 9. 续期、吊销和换机

### 续期

在后台为原组织和设备重新签发有效期更晚的授权包，随后通过客户端导入或 MDM 覆盖托管文件。客户端不需要重新初始化密码数据。

### 吊销

在后台的 grant 历史中吊销目标授权，然后签发包含最新吊销列表的新授权包并分发。离线客户端只有在收到新包后才能知道新的吊销状态。

### 换机

新电脑会产生新的商业设备 ID 和指纹。应在新电脑重新导出设备请求并签发新 grant；不要把旧电脑的本地授权状态复制到新电脑。

### 清除本机授权

客户端 **设置 → 商业授权 → 清除本机授权** 只删除本机商业授权，不会删除或修改因子包、CDR、恢复包装或密码记录。

## 10. 常见问题

| 现象 | 常见原因 | 处理方法 |
| --- | --- | --- |
| 始终显示“未授权” | 没有导入授权包 | 在线激活或导入后台签发的完整 JSON |
| 显示“无效” | 客户端公钥或 key ID 与后台不一致，或文件被修改 | 用同一后台公钥重新构建客户端并重新签发 |
| 显示“不适用于此设备” | 授权包中没有当前设备 grant | 重新复制本机设备请求并导入后台签发 |
| 显示“当前应用版本未获授权” | 客户端主版本不在组织允许列表 | 通过 API 新建允许该主版本的组织授权，或使用被允许的客户端主版本 |
| 在线激活失败 | 使用了非本机 HTTP、证书失败、地址或激活码错误 | 改用有效 HTTPS，检查反向代理和激活码 |
| 提示席位已满 | 已激活设备达到 Max seats | 吊销/回收旧设备席位，或按合同增加席位 |
| 授权包复制到另一台电脑无效 | 设备绑定生效 | 为新电脑单独导入设备请求并签发 |
| 未授权时仍可派生密码 | 使用了评估构建 | 用 `tools/commercial/build_commercial_release.sh` 重新构建 |

## 11. 上线前检查清单

- [ ] 后台部署在组织控制的主机上，`.env` 权限和备份已确认。
- [ ] 签名私钥没有进入客户端、代码仓库或普通日志。
- [ ] 客户端公钥和 key ID 与后台完全一致。
- [ ] 商业客户端通过商业构建入口生成，并完成平台代码签名。
- [ ] 组织席位数、有效期、宽限期和 features 已核对。
- [ ] 在线激活使用 HTTPS，并对激活接口限流。
- [ ] 已分别验证在线激活、离线导入、错误设备拒绝和到期状态。
- [ ] 已建立数据库、签名密钥、授权包和审计记录备份。
- [ ] 已制定续期、吊销、换机和签名密钥轮换流程。

## 12. 实现边界

本实现能防止直接复制安装包、把单设备授权复制给其他电脑、离线部署超出已签发席位，以及被篡改授权包的使用。它不承诺阻止攻击者修改本地二进制、完整克隆机器状态或修改源码移除检查。

生产环境仍需由部署方完成 TLS、限流、签名私钥保护、数据库备份、平台代码签名与公证，以及签名密钥轮换管理。这些属于上线运维控制，不是客户端授权流程中的缺失功能。
