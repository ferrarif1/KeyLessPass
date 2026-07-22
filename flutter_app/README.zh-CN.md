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

然后在客户端：

1. 打开“安全”。
2. 在线方式：点“在线激活”，输入 HTTPS 授权服务地址和组织激活码。
3. 离线方式：点“复制设备请求”，交给后台导入并签发授权包，再点“导入授权包”。

本机测试时，在线激活地址可以使用：

```text
http://127.0.0.1:8787
```

真实内网或公网环境必须使用 HTTPS。
