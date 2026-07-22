# 贡献说明

English: [CONTRIBUTING.md](CONTRIBUTING.md)

感谢改进 KeyLessPass。提交前请保持改动符合“本地密码派生，不保存服务密码”的产品边界。

## 规则

- 不要增加云同步、云账号、浏览器自动填充或 Web 后端。
- 不要保存目标系统明文密码。
- 不要保存助记短语。
- 不要把助记短语做成服务密码根种子。
- 不要记录敏感材料到日志。
- 派生逻辑必须基于稳定 CDR 字段，不基于可编辑展示字段。

## 提交前检查

```bash
cd rust_core
cargo test

cd ../flutter_app
flutter analyze
flutter test
```

发布前还要做敏感词扫描，并人工复核结果。
