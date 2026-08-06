# 安全策略与 v3 声明边界

安全问题请私下发送至 `revanton@icloud.com`，并提供受影响提交、复现步骤、影响和建议修复。不要提交真实生产凭据。

v3 使用成熟 Shamir 2-of-3 拆分随机 Root Key；一个份额不足以通过恢复接口。份额封装绑定 Vault、Root Key/share set/因素代际、因素角色和套件，并通过重建后的 HMAC 与 KCV 验证。CDR MAC、父哈希和持久化轮换状态用于检测篡改、部分副本不一致和远端结果未知。

以下不属于安全承诺：普通 U 盘不可复制；受感染终端上的份额独立性；进程内存中的 Root Key/服务密码保密；标准 Shamir 不重建完整密钥；仅靠本地 HMAC/哈希检测全部副本整体回滚；仅刷新 share set 即撤销已泄露的两份旧份额。

当前 Rust Core 已实现 v3 创建与恢复、份额刷新、因素替换、空 Vault Root Key 轮换、v2 迁移、CDR v3、无偏编码、轮换 reconciliation、冲突分类和 freshness 接口。Flutter 新建流程仍为 v2；非空 Vault Root Key 全量迁移、目标系统适配器、生产 freshness 服务和完整 v3 UI 尚未交付，不能作为当前安全声明。
