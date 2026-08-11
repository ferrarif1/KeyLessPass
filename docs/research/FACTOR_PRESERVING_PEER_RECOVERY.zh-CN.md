# 因素保持的异构 Root-Key 网络恢复方案

## 1. 研究边界

本方案保留顶层 Root Key 的标准 Shamir 2-of-3 结构：

```text
(S_D, S_U, S_N) <- ShamirSplit(2, 3, K_root)
```

其中 `D` 为受管设备，`U` 为离线或可移动介质，`N` 为独立管理的网络
恢复域。当前主动实现不使用 Data Key、View Key、threshold OPRF、隐匿
对象扫描或纠删编码密文。上述设计属于已废弃实验，不进入新论文和主动
artifact。

研究问题不是重新发明 Shamir，而是判断部署后的能力是否让逻辑 2-of-3
退化为实际 1-of-n。

## 2. 能力闭包与因素坍缩

对任一受保护的失陷域 `X`，定义 `Closure(X)` 包含攻击者从该域获得的
文件、安全存储内容、API/TLS 凭据、Cookie、请求签名能力、可自动调用
的协议，以及诚实服务按这些能力返回的响应。因素保持条件为：

```text
|Closure(X) intersect {S_D, S_U, S_N}| < 2
```

若设备同时持有 `S_D`，又能凭设备自身凭据无人审批地取得 `S_N`，则
Shamir 多项式仍是 2-of-3，但部署后的访问结构已坍缩。

## 3. 当前协议

1. 将经过认证的网络 `ShareEnvelope` 再用标准 Shamir 分为 3-of-5
   网络片段；它们只承载一个顶层份额 `S_N`。
2. 恢复票据绑定 `vaultID`、`rootGeneration`、`shareSetID`、
   `shareSetGeneration`、随机 `opID`、临时 X25519 公钥、签发/过期时间、
   操作目的和授权节点集合。
3. 至少两个不同、独立管理的审批者用 Ed25519 对完整票据签名。设备只有
   请求能力，没有审批签名能力。
4. 节点验证票据、代次、节点范围、新鲜度、双人审批和防重放账本后，才
   释放自身片段。
5. 节点通过临时 X25519、HKDF-SHA256 和 AES-GCM 将片段只加密给本次
   会话；票据摘要和节点身份作为关联数据。
6. `(nodeID, opID, ticketHash)` 保证幂等；相同 `opID` 搭配不同票据会被
   拒绝。
7. 客户端收集任意三个有效片段重构 `S_N`，仍必须结合 `S_D` 或 `S_U`
   才能恢复 Root Key，并通过顶层 KCV 与 envelope MAC 验证。

## 4. 生命周期

普通重新分片保留 `K_root` 和 `rootGeneration`，只递增
`shareSetGeneration`。如果怀疑旧门限已经泄露，则必须替换 Root Key，
同时递增 Root-Key 和 share-set 代次。仅重新分片不能撤销攻击者已经得到
的两个旧顶层份额。

节点按 Root-Key 代次、share-set 标识和 share-set 代次拒绝旧票据。
统一 freshness checkpoint 还记录每个 credential 的 `policyEpoch`、
`credentialGeneration` 和 lineage，用于区分回滚与同代分叉。

## 5. 已有证据与限制

测试覆盖全部十种 3-of-5 组合，以及审批不足/重复、错误节点、过期票据、
旧 share-set、混合代次、防重放、密文篡改和单域能力闭包。TLA+ 有界模型
检查授权、新鲜度和因素不坍缩不变量。

当前性能只代表本地密码学基线，不包含网络 RTT、人工审批等待、硬件签名、
跨地域一致性和真实传输故障。该 feature 未接入桌面产品，也不宣称新的
Shamir 原语、生产级恢复服务、匿名性或 Byzantine 节点容错。
