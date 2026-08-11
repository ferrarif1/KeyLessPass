# 因素保持恢复方案最新改造报告

日期：2026-08-09

## 结论

旧的 Data Key / View Key、threshold OPRF、固定隐匿对象和纠删编码方案已
从主动实现与新论文中移除。当前方案只保留解决“因素坍缩”所需的最小
结构：顶层 D/U/N Shamir 2-of-3、网络 S_N 的 3-of-5 Shamir 片段、两个
独立 Ed25519 审批者、会话绑定的 X25519/AES-GCM 释放，以及代次与
freshness 检查。

## 本轮实现

- 引入 `shareSetGeneration`，区分普通重新分片和 Root-Key 替换；
- 票据绑定 vault、Root/share-set 代次、opID、恢复会话公钥、有效期、
  操作目的和节点集合；
- D 只有请求能力，A 只有签名审批能力且不保存 Root share；
- 节点防重放账本保证同票据幂等、异票据同 opID 拒绝；
- freshness 同时绑定 Root Key、share set、CDR、policy epoch、credential
  generation 和 credential lineage；
- Kcred 暴露通过更换 credential salt 开启新 lineage；Kroot 暴露必须
  替换 Root Key 并轮换所有远端凭据。

## 证据边界

全部 85 个 Rust 测试和严格 Clippy 通过；恢复 TLA+ 检查 852,704 个
distinct states，集成 freshness 模型检查 40,292 个 distinct states。
本地 3 节点释放与重构平均 6.036 ms，但没有测网络 RTT 或人工审批时间。

因此当前可主张的是“能力闭包下的因素保持分析与生命周期协议研究
原型”，不能主张新 Shamir、生产级网络恢复、匿名性或 Byzantine 容错。
