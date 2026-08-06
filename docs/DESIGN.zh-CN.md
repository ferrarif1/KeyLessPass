# KeyLessPass v3 设计说明

KeyLessPass 定位为仍依赖文本密码的企业遗留系统生命周期架构，不提出新的 KDF、AEAD、秘密共享算法或密码策略语言。研究对象是恢复代际、凭据代际、远端轮换不确定结果、副本冲突及可选新鲜度锚点之间的统一状态协议。

v3 使用 256 位随机 Root Key，并由 `vsss-rs` 实现的 GF(256) Shamir 2-of-3 方案拆分为恢复份额、本机份额和 U 盘份额。恢复份额编码为带校验和的 108 个英文单词；本机份额由平台能力保护；普通 U 盘只作为可复制的份额文件载体。份额封装绑定 Vault、Root Key 代际、share set、因素类型/标识/代际及密码套件。重建后使用 Root-Key 派生的 HMAC 校验封装，并用 KCV 验证 Root Key。

旧 `W_MC`、`W_MU`、`W_CU` 是 v2 成对完整密钥 wrapper，不是 secret share，仅保留迁移读取。迁移验证三条旧路径得到相同 Root Key，再写入并验证 v3 份额，最后提交 manifest，因此不改变已有服务密码。

CDR v3 使用 RFC 8785 JSON 规范化，显式记录 Vault、服务、账号、凭据代际、Root Key 代际、策略/编码器版本、轮换状态、操作 ID、父哈希及副本元数据。编码器使用 HMAC 确定性流、拒绝采样和无偏 Fisher--Yates，支持字符类数量、首尾限制、禁重复/连续字符、矛盾策略拒绝及最大尝试次数。

密码轮换不是 two-phase commit，而是持久化的 pending-confirm-reconcile 状态协议。请求结果未知时必须进入 reconciliation；新密码验证成功才提交，旧密码验证成功则中止，两者均失败则转人工处理。

本地模式只能检测篡改和部分副本不一致，不能检测所有合法副本整体回滚。企业锚定模式提供 Root Key 代际、CDR epoch 和 digest 的 CAS 新鲜度接口；仓库只包含内存测试实现。

当前限制：Flutter 初始化仍生成 v2 数据，v3 需通过 Rust 迁移接口启用；非空 Vault 的 Root Key 全量轮换、生产 freshness 服务、目标系统适配器和完整 v3 桌面恢复界面尚未交付。详见 [LIMITATIONS.md](LIMITATIONS.md)。
