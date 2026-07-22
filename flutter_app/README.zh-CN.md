# KeyLessPass 桌面客户端

English: [README.md](README.md)

这是 KeyLessPass 的 Flutter Desktop 客户端。普通用户通过它初始化本机、创建 U 盘因子、添加记录、派生密码、做轮换和恢复，也可以导入商业授权。

## 本地运行

先构建 Rust Core：

```bash
cd ../rust_core
cargo build
```

再运行 Flutter：

```bash
cd ../flutter_app
flutter pub get
flutter run -d macos
```

如果 macOS 提示找不到 `xcodebuild`：

```bash
DEVELOPER_DIR=/Applications/Xcode.app/Contents/Developer flutter run -d macos
```

## 检查

```bash
flutter analyze
flutter test
```

## 授权测试

先启动后台：

```bash
cd ../admin_backend
./scripts/intranet_deploy.sh
```

商业客户端启动后会自动授权：

1. 优先读取受管目录或用户下载目录中最新的 `keylesspass-client-config.json`；下载页在用户点击安装包时会同时生成并下载它；
2. 没有配置时才通过 UDP 8788 发现内网服务器；
3. 自动登记设备，等待厂商批量批准；
4. 批准文件导入服务端后，客户端轮询并自动导入本机授权。

普通用户无需进入“安全”页，也不需要服务器地址、激活码、Admin token 或手动移动配置文件。浏览器阻止多个下载时需要允许该内网站点下载；受管配置位置见 [服务端中文说明](../admin_backend/README.zh-CN.md)。

以下仅用于开发测试和故障备用：

1. 打开“安全”。
2. 在线方式：点“在线激活”，输入 HTTPS 授权服务地址和组织激活码。
3. 离线方式：点“复制设备请求”，交给后台导入并签发授权包，再点“导入授权包”。

本机测试时，在线激活地址可以使用：

```text
http://127.0.0.1:8787
```

手动输入地址的真实内网或公网环境必须使用 HTTPS；自动发现和受管配置可在纯内网使用 HTTP，因为最终授权仍由厂商根签名校验。
