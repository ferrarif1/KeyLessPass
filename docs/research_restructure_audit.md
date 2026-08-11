# Research Restructure Audit

审计日期：2026-08-08  
审计对象：`rust_core`、现有 deterministic-password/lifecycle 稿件、Factor-Preserving Peer Recovery 稿件、TLA+ 模型、测试向量与实验脚本。

## 1. 结论

当前项目不是需要推倒重来，而是需要更换研究主轴并补齐一个严格版本化的算法层：保留随机 256-bit Root Key、Shamir 2-of-3、认证恢复对象、CDR、远端证据约束轮换和复制/新鲜度语义；将 EncoderV2 固定为兼容算法，新建 `derivationVersion = 3`、`encoderVersion = 3` 的 exact policy-space 路径。

仓库中的 `research::psppd` 已验证“有限 DFA + BigUint 动态规划 + rank/unrank”的最小可行性，但仍是研究原型，不是可以直接写进论文的完成方案。尤其是：它只接收手工构造 DFA；策略覆盖范围很窄；没有 `DomainPermutation` 抽象；FF1-on-binary-superset 的 cycle-walking 仅是带边界的原型；没有 CDR v3 接入、`policyEpoch`、跨 epoch 历史排除、迁移或固定向量。

因此，主论文应围绕“可精确计数的策略语言 + rank/unrank + 有限域排列假设下的确定性轮换”展开。P2P recovery 只保留为第三 Shamir share 的 Network Recovery Profile。

## 2. 已正确实现、应保留的部分

### 2.1 Root Key 与恢复

- Root Key 是随机 256 bit 值，而不是从低熵人类因素直接派生。
- `vsss-rs` 在 GF(256) 上实现标准 2-of-3 Shamir；三个逻辑因素有独立的 `factor_type`、`factor_id` 和 generation。
- `RecoveryManifest` 明确绑定 `vaultID`、`rootGeneration`、`shareSetID`、阈值、份额数和 KCV。
- 每个 `ShareEnvelope` 的 MAC 在候选 Root Key 重构后验证；KCV 先排除错误候选，验证顺序不存在秘密依赖循环。
- 三个份额均存在时，`recover_root_key_from_available` 尝试三种 pair，能标记唯一失败 pair 所排除的疑似损坏因素，并拒绝不同 pair 恢复出不同 Root Key 的情况。
- 网络第三份额和离线恢复份额已经具有不同数据类型；`D + U` 本地恢复路径不依赖网络模块。

这些是成熟支撑模块，不是论文的密码学创新。

### 2.2 CDR、密钥分离与固定向量

- CDR 使用 RFC 8785 canonical JSON；MAC 覆盖 lifecycle 和展示字段，密码派生输入只包含明确选定的身份、generation、salt 和策略摘要。
- `derive_vault_subkey` 将 `vaultID`、`rootGeneration`、crypto-suite version 和 purpose 纳入 HKDF 上下文。
- 服务和账户使用非空 UUID；credential salt 固定为 128 bit；策略描述经 canonical serialization 后取 SHA-256。
- `recordID`、`recordSeq` 等管理字段被有意排除在新式服务密码 seed 输入之外，避免管理迁移改变密码。
- CDR、password derivation、recovery phrase 已有跨平台固定向量。

需修正的只是命名：代码函数名 `password_derivation_input_v3` / `derive_service_secret_v3` 中的 “v3” 指 CDR schema v3，而字段 `derivationVersion` 实际仍为 2。新算法接入前必须消除这层歧义。

### 2.3 轮换与复制语义

- 轮换显式表示 `PREPARED`、`UPDATE_SENT`、`UNKNOWN_OUTCOME`、reconciliation、confirmed、committed、aborted、ambiguous 和 rollback-required。
- 远端证据不是单一布尔值，而是 old/new 各自的 success、conclusive failure 或 indeterminate，并绑定 endpoint 和 observation time。
- 状态机能够区分 old-only、new-only、both、neither；只有证据充分且契约允许时才提交。
- pending CDR 保留父记录 hash、operation ID、Lamport clock 和 replica epoch。

这些语义应继续服务于“候选 generation 何时成为 committed generation”，而不扩大为浏览器自动改密或真实服务端事务协议。

### 2.4 Network Recovery Profile 中可复用的部分

- encrypted network share；
- fixed-size opaque object；
- view/data authority separation；
- 3-of-5 erasure fragments；
- capability、approval、freshness、threshold release 的合取释放条件；
- `shareSetID`、`rootGeneration`、`objectEpoch` 的一致绑定；
- 恢复后的 share refresh 语义；
- factor-collapse 作为部署/状态安全性质；
- 现有本地 cryptographic baseline 和 TLA+ 模型可作为 supporting evidence。

这些机制只能说明网络份额不会因单一设备因素失陷而顺带释放。它们不证明匿名性、抗流量分析、抗 Sybil、生产级 Byzantine robustness 或新 threshold-OPRF 原语。

## 3. EncoderV2 的数学缺陷

`crypto::encoder::encode_password` 使用 HMAC 确定性字节流，并通过 rejection sampling 实现每个 `sample_below(n)` 的无 modulo bias；Fisher--Yates 中的索引抽样也没有取模偏差。这些局部性质是正确的。

但最终算法依次执行：随机排列空位、按类补齐最小数量、再填充其余字符、最后验证。一个合法字符串可能由不同数量的内部选择路径产生；在约束重叠、min/max count、fixed position、首尾限制和 no-repeat/no-sequential 限制下，不同合法字符串的前像数量通常不同。因此：

- 它没有在完整合法语言上定义均匀分布；
- `password_space_upper_bound_log2` 只返回未固定位置的 `L log2 |alphabet|`，不是 `log2 |L_P|`；
- 最大尝试数意味着部分非空策略可能确定性失败；
- 不同 credential generations 由独立 seed 再编码，无法从构造上保证密码不重复；
- 无法直接证明企业 history 5/10/24 的同 epoch 零重复性质。

因此 EncoderV2 只能描述为“使用无模偏的局部抽样并最终验证策略的兼容编码器”，不能描述为“在全部合法密码上 unbiased/uniform”。

## 4. 已有 PSPPD 原型的可复用程度

`research::psppd` 中以下部分可直接提炼：

- canonical alphabet order；
- 对每个有限长度构建 `counts[position][state]` 的 BigUint DP；
- 多长度空间按长度升序拼接；
- `rank` / `unrank` 的字典序双射；
- 精确 `total_count` 和显示用 `log2(total_count)`；
- `{a,b0,...,b9}` 偏置示例；
- 一个 required-class bitset product DFA 的最小构造器。

以下部分不能原样升级为主实现：

- 策略必须手工提供 transition table，尚无 Policy IR/compiler；
- required-class helper 只处理固定长度和“至少出现一次”，不支持 min/max count、边缘限制、重复 run、顺序 run、固定 prefix/suffix 或 forbidden substring；
- 只有四个普通单元测试，没有 proptest 的全域双射性质测试；
- FF1 直接作用于最小二进制超集后 cycle-walk，使用固定的 1024 次运行上限；上限错误会使理想排列变成可能失败的部分算法；
- `MIN_FF1_DOMAIN_SIZE = 1,000,000` 只是 FF1 标准最小 domain 约束的代码化边界，不足以证明任意 BigUint domain 的小域/多用户安全性；
- 研究模块没有稳定 serialization、policy hash、fixed vector 或 backward compatibility contract。

## 5. 应删除或降级的论述

### 5.1 从主论文移除

- 浏览器插件、DOM 识别、自动填充、剪贴板 UX、自动 Web 改密；
- P2P 匿名性、DHT、Sybil resistance、广义 distributed storage；
- 把 Shamir、HKDF、HMAC、JCS、DFA、rank/unrank、FPE、OPRF 或 erasure coding 本身列为创新；
- 把 100,000 个内存对象扫描或本地椭圆曲线运算当作生产网络性能；
- 108-word “recovery phrase” 的可用性暗示，应改称 offline/paper recovery package；
- 任何未测量的 Windows/Linux、真实 adapter、真实用户可用性或生产 freshness 服务结论。

### 5.2 降为 supporting profile / appendix

- Factor-Preserving Opaque Peer Recovery 的完整协议；
- threshold OPRF arithmetic 和 fixed-size opaque object 格式；
- recovery TLA+ 的 7,296-state 有界检查；
- evidence-bounded remote rotation 的详细 contract taxonomy；
- 旧 pairwise-wrapper 到 authenticated Shamir 的历史迁移。

### 5.3 现有两篇稿件的处理

- deterministic/lifecycle 稿件不再继续润色；其 recovery、CDR、rotation 章节作为新稿件素材库。
- Factor-Preserving 稿件保留为独立备份/后续研究，不并入主论文主线。
- 当前主稿的 Table 1、摘要、贡献列表和结论均不得在 Phase 10 之前改写，避免在算法与查新未定时反复包装 novelty。

## 6. 新算法需要修改的接口

### 6.1 新增 Policy IR 与 compiler

建议新增独立 `policy` 模块，提供：

```text
PolicyDescriptorV3 -> canonical Policy IR -> finite-state machine
count_policy_space
rank
unrank
```

compiler 必须给出明确的支持集合和拒绝原因。计数全程使用 `BigUint`，不能将 `N_P` 压缩为 u64/u128。

### 6.2 新增有限域排列抽象

```rust
trait DomainPermutation {
    fn permute(key, tweak, domain_size, input) -> Result<BigUint>;
    fn invert(key, tweak, domain_size, input) -> Result<BigUint>;
}
```

论文定理只依赖此接口的 keyed-permutation 假设。FF1 cycle-walking 可作为 prototype backend，但必须单独标注适用 domain、失败边界、标准依赖和与理想 arbitrary-domain primitive 的差距。

### 6.3 CDR 与派生上下文

新增并认证：

- `policyEpoch: u64`；
- v3 canonical policy 表示或稳定 policy hash；
- 明确的 history window；
- 必要时记录 `subGeneration`/exclusion counter，但其语义必须稳定。

`credentialGeneration` 作为 permutation 输入 `x = g`，不得再同时作为 tweak 的一部分；否则每个 generation 对应不同排列，不能由同一排列的单射性推出无碰撞。tweak 绑定 vault/service/account/salt/root generation/policy identity/policy epoch/algorithm versions。

必须检查 `g < N_P`；不得使用 `g % N_P`。空间耗尽时只能显式 rollover policy epoch、变更策略或变更 credential domain。

### 6.4 derive dispatcher

当前 `service::derive` 只按 `schemaVersion` 和 `derivationVersion >= 2` 区分新旧 seed，随后无条件调用 EncoderV2。需改为精确 match：

```text
(derivationVersion=1, legacy encoder)
(derivationVersion=2, encoderVersion=2)
(derivationVersion=3, encoderVersion=3)
otherwise -> reject
```

不能使用 `>=` 接受未知未来版本。

## 7. 固定向量影响

若直接修改 `EncodingDescriptor`、CDR 默认字段、canonical serialization 或 EncoderV2 行为，会破坏：

- `cdr-v3-rfc8785.json`；
- `password-derivation-v2.json`；
- 现有 CDR MAC；
- 已部署 record 的密码再生；
- 研究实验基线。

正确做法是：

1. 完全冻结 EncoderV2 代码路径和 v2 固定向量；
2. 新增 v3 policy descriptor/IR、canonical vector、derivation context vector、permutation vector 和 end-to-end password vector；
3. 旧 CDR 反序列化时不得通过默认字段偷偷进入 v3；
4. v3 仅由显式远端轮换创建，候选密码经远端证据确认后才 commit。

## 8. Migration 与版本策略

### 8.1 不允许原地升级

软件升级、策略解析器升级或 policy compiler 优化不得改变现有 credential 的密码。旧 record 永远按其记录的 derivation/encoder/policy canonicalization version 再生。

### 8.2 显式 v2 -> v3 轮换

迁移流程应为：

1. 读取并验证 active v2 CDR；
2. 编译目标 v3 policy，计算 `N_P`，验证 `credentialGeneration < N_P`；
3. 创建 pending v3 CDR，设置 `derivationVersion = 3`、`encoderVersion = 3`、`policyEpoch = 1`；
4. 用 v3 算法派生 candidate password；
5. 通过已有 remote-evidence 接口确认 `new succeeds && old conclusively fails`；
6. 原子提交 v3 CDR，保留 v2 predecessor 以支持 reconciliation/audit；
7. 失败或不确定时仍以 v2 committed record 为当前状态。

迁移不能复用当前 `service::migration` 的名称和语义：现有模块处理 pairwise recovery wrapper -> Shamir share set，而不是 password encoder v2 -> v3。两者应拆分命名。

### 8.3 policy epoch

- 同一 policy epoch 内 generation 增加，依靠同一 domain permutation 的单射性保证不重复。
- 实质策略变化创建新 epoch；跨 epoch 可能存在语言交集，必须重新派生 history window 内的旧密码并执行 deterministic exclusion。
- `rootGeneration`、`credentialGeneration`、`policyEpoch`、`shareSetGeneration` 分别演化，不得互相替代。

## 9. Phase 1 后的准入条件

只有同时满足以下条件，才能进入论文重写：

- 查新未发现“同一构造 + 同一应用 + 同一性质”的前置工作；
- Policy IR 的支持范围与状态复杂度已写清；
- BigUint exact count 和 rank/unrank 通过 property tests；
- DomainPermutation 的论文假设与 prototype backend 边界分离；
- v2 fixed vectors 原样通过，v3 新向量独立加入；
- policy epoch、history 和 domain exhaustion 有可执行语义；
- 实验由脚本生成原始数据，结论服从数据。

## 10. 研究定位

后续所有设计均以此为准：

> This work studies how a threshold-protected random Root Key can deterministically generate independent, high-entropy, policy-compliant, versioned passwords for many legacy enterprise accounts without storing per-service password values.

