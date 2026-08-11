# Novelty and Prior-Art Audit

检索日期：2026-08-08  
目的：界定 exact policy-space derivation、有限域排列、确定性轮换和 threshold recovery 的可主张贡献。本文档是先行研究审计，不是专利法律意见，也不能替代投稿前由作者和图书馆数据库完成的最终检索。

## 1. 最重要的结论

### 1.1 “PSPPD” 名称不是已知标准术语，但总体构造不能宣称为全新数学原语

当前建议的组合：

```text
finite policy language
-> exact count and rank/unrank
-> rank-then-encipher / arbitrary-domain permutation
-> unrank into the same language
```

与 Bellare、Ristenpart、Rogaway、Stegers 在 2009 年 FPE 工作中明确研究的 **rank-then-encipher** 范式高度重合。任意有限域上的 cipher 也至少由 Black 与 Rogaway 在 2002 年正式研究。正则语言的计数、ranking/unranking 和按补全数加权的均匀生成更早已有理论。因此：

- 不能声称发明了有限语言、DFA、exact counting、rank/unrank、uniform generation、FPE、cycle walking 或 rank-then-encipher；
- 不能把“generation 作为 permutation 输入，因此同域内不重复”包装为新的密码学定理；它是 permutation 单射性的直接推论；
- `PSPPD` 最多是本文系统中这一实例化协议的工作名称，不应称为新的密码原语。

可防守的研究贡献应收缩为：

> 将既有 rank-then-encipher 方法专门实例化为不存逐服务密码值的企业凭据生命周期协议，给出可执行的策略 IR、精确空间度量、版本化 domain、同 policy epoch 的无重复轮换、跨 epoch 的 deterministic history exclusion，并与 threshold-protected random Root Key、远端提交证据和 backward-compatible migration 共同实现和评估。

这是**应用算法与系统协议贡献**，不是基础密码学发明。它可能适合 JISA 一类应用安全/系统期刊；若以“全新数学构造”投稿密码学顶会，当前 novelty 不足。

### 1.2 与 MFDPG 的真实区别已得到源码级确认

MFDPG 论文提出 regex -> NFA -> DFA -> Xeger-style random traversal，并用确定性 PRNG 使同一输入再生同一密码。论文证明/讨论的是 determinism、policy acceptance 和伪随机源，没有给出合法语言 cardinality、rank/unrank、合法输出等概率或跨轮换零碰撞保证。

其公开实现（`multifactor/mfdpg`，审计 commit `6c26096dd22ff2b18aa5d8e4c3d5b0caf7b45bb7`）实际使用 `randexp@0.5.3`。`RandExp` 对 regex alternation 的 options 等概率选择，并对 repetition 长度等概率选择，而不是按每个分支的合法补全字符串数量加权。因此对 `/a|b[0-9]/`：

```text
Pr[a] = 1/2
Pr[b0] = ... = Pr[b9] = 1/20
```

而合法语言上的均匀概率应为 `1/11`。这不是对“所有 Xeger 实现”的泛化指控，而是对 MFDPG 论文算法边界和其公开 artifact 的可复现实证。

### 1.3 有限域 permutation 是当前最大的密码学实现风险

NIST FF1 处理 radix 字符串域，而不是无条件提供任意 BigUint `[0,N)` 的 API。2025 年 SP 800-38G Rev.1 第二公开草案将 FF1 最小 domain size 提高到 1,000,000，并移除 FF3；small-domain attacks 说明不能把“是 permutation”与“具有足够 PRP security”混为一谈。

当前原型把 `[0,N)` 嵌入最小二进制超集，用 radix-2 FF1 cycle-walk 回子集。这在数学上可形成 permutation（无限制 walk 时），但固定 `max_walks = 1024` 会引入可失败输入，且 FF1 的标准/安全边界、tweak 长度和 multi-user query security 必须独立分析。论文应以 `DomainPermutation` 为假设；FF1 cycle-walking 只能称为 bounded prototype backend。

## 2. Deterministic password generation

| 工作 | 已解决 | 未解决/与本文边界 | exact uniform / rank | collision-free rotation |
|---|---|---|---|---|
| [PwdHash, USENIX Security 2005](https://www.usenix.org/legacy/event/sec05/tech/full_papers/ross/ross.pdf) | master password + site data 生成 site-specific password；无需服务端改造 | 浏览器扩展、低熵 master-secret 暴露面、复杂策略和 lifecycle 非主线 | 未提供 | 未提供构造保证 |
| [AutoPass, 2017](https://arxiv.org/abs/1703.01959) | site rules、forced changes、预设密码、site-specific requirements | 不提供 exact finite-language cardinality 或 rank-then-encipher lifecycle | 未提供 | 支持轮换，但非 permutation-domain 零重复证明 |
| [PALPAS, 2015](https://arxiv.org/abs/1506.04549) | 高熵共享 secret + per-service salt；同步非秘密 metadata；policy-aware generation | 需要同步 salt/metadata；未把 generation 作为有限策略域 permutation 输入 | 未提供 | 未提供绝对同 epoch 零重复 |
| [Spectre/Master Password](https://spectre.app/blog/2018-01-06-algorithm/) | site counter、HMAC service isolation、模板式输出、stateless regeneration | 预定义模板而非一般企业 Policy IR；示例算法直接用 byte modulo | 未提供 | counter 改变输入，但不同 counter 仍可能输出相同字符串 |
| [MFDPG, 2023](https://arxiv.org/pdf/2306.14746) | 多因素派生、zero stored credential secrets、revocation filter、regular-policy generation、100-site compatibility | 不提供 exact count/rank/unrank；artifact 的 RandExp 分支概率不是语言均匀概率 | 未提供 | 论文称新密码“except with negligible probability”不同，不是零碰撞构造 |
| [MFKDF, USENIX Security 2023](https://www.usenix.org/system/files/usenixsecurity23-nair-mfkdf.pdf) | 多因素派生、threshold MFKDF 和因素丢失后的 client recovery | 研究的是 factor-derived key，不是随机 Root Key 上的 policy-space password sequence | 不适用 | 不适用 |
| [MFKDF cryptanalysis, USENIX Security 2024](https://eprint.iacr.org/2024/935.pdf) | 分析 state integrity、share format、implementation/spec divergence 等问题 | 支持本文选择 random Root Key + standard Shamir + explicit authenticated state；不是 password generator | 不适用 | 不适用 |

结论：MFDPG 是应用最近邻；本文不能以“支持 regex/DFA policy”作为 novelty。可比较性质是 exact cardinality、语言级均匀模型、permutation-based non-repetition、显式 policy epoch 和 lifecycle migration。

## 3. Password policy representation and uniform generation

| 工作 | 已解决 | 对本文的限制 |
|---|---|---|
| [Gautam--Lalani--Ruoti PCP language, SOUPS 2022](https://www.usenix.org/system/files/soups2022-gautam.pdf) | 从 626 网站提取 270 policies，设计可表达实际 PCP 的描述语言并提供生成库 | Policy IR/真实 policy corpus 不是本文可随意宣称的新贡献；其生成器也是“安排 required slots + shuffle + fill”的构造式算法，不等同于合法语言均匀采样 |
| [Hickey--Cohen, SIAM J. Comput. 1983](https://epubs.siam.org/doi/10.1137/0212044) | 对无歧义 CFG 的均匀随机字符串生成；指出 finite-state 特例可线性生成 | 按补全数加权的均匀生成思想早已存在 |
| [Goldberg--Sipser, 1991](https://doi.org/10.1016/0304-3975(91)90144-Q) | 将 regular-language ranking 归约到给定长度字符串计数 | regular-language ranking 不是新贡献 |
| [Mairson, 1994](https://doi.org/10.1016/0020-0190(94)90033-7) | 无歧义 CFG 上均匀生成与预计算权衡 | exact-count-guided generation 不是新贡献 |
| [Lorenz--Ponty, 2012](https://arxiv.org/abs/1211.0303) | non-redundant generation；比较 rejection 和 unranking | 不重复枚举以及用 unranking 避免重复已有形式化研究 |

本文的 Policy IR 应只声称是为 legacy enterprise password constraints 设计的可执行子集。固定有限最大长度、有限 alphabet、class counters、边缘约束、bounded repeats/sequences 和有限 forbidden substrings 都可编译为 finite-state product。用户名等 credential-specific forbidden substring 可以在编译时变成有限 automaton，但会使 policy hash/隐私边界发生变化，必须明确是否支持。

## 4. Format-preserving and arbitrary-domain permutations

| 工作 | 已解决 | 对本文的限制 |
|---|---|---|
| [Black--Rogaway, Ciphers with Arbitrary Finite Domains, 2002](https://www.cs.ucdavis.edu/~rogaway/papers/subset.pdf) | 正式研究 `[0,k-1]` 等任意有限 message domain 上的 cipher | arbitrary-domain permutation 不是本文发明 |
| [Bellare et al., Format-Preserving Encryption, SAC 2009](https://eprint.iacr.org/2009/251.pdf) | 定义 FPE security；直接研究 rank-then-encipher、Feistel 和 cycle walking | `Rank -> PRP -> Unrank` 总体形式已有直接 prior art，是最关键 novelty blocker |
| [NIST SP 800-38G](https://doi.org/10.6028/NIST.SP.800-38G) | 标准化 FF1/FF3 的 radix-string FPE | 不是任意 BigInt domain 的即插即用标准；版本与参数必须固定 |
| [NIST SP 800-38G Rev.1 2PD, 2025](https://csrc.nist.gov/pubs/sp/800/38/g/r1/2pd) | 将 FF1 minimum domain 提升至 1,000,000；移除 FF3；禁止 FF1 浮点实现 | 当前 prototype 的 domain lower bound 有来源，但草案状态与最终标准状态必须在投稿时刷新 |
| [Hoang--Tessaro--Trieu, CRYPTO 2018](https://eprint.iacr.org/2018/556.pdf) | small-domain FPE attacks，尤其 multi-target 场景 | policy space 大不等于所有攻击模型下自动安全；需要 query/domain bounds |
| [Hoang--Morris--Rogaway, Swap-or-Not](https://arxiv.org/abs/1208.1176) | 小域 cipher/PRP 的另一已分析构造 | 若替换 backend，也仍是使用既有原语，不是本文创新 |

安全定位：论文定义 `DomainPermutation` 的 correctness/bijection 与 PRP assumption；prototype backend 只验证接口与性能，不声称发明或完整证明 arbitrary-domain cipher。

## 5. Recovery and distributed storage

| 工作 | 已解决 | 与本文边界 |
|---|---|---|
| [Shamir, 1979](https://doi.org/10.1145/359168.359176) | `k-of-n` information-theoretic threshold sharing | 2-of-3 恢复完全是标准组件 |
| [MFKDF, 2023](https://www.usenix.org/conference/usenixsecurity23/presentation/nair-mfkdf) | threshold factor-derived key 和 client recovery | 本文的区别是随机 Root Key、标准 shares 和显式 state authentication，不是“首次 threshold recovery” |
| [TOPPSS, ACNS 2017](https://www.research.ed.ac.uk/en/publications/toppss-cost-minimal-password-protected-secret-sharing-based-on-th/) | threshold password-protected secret sharing；UC threshold OPRF | threshold OPRF + recovery 的组合远非新颖；当前 elementary interpolation 不足以替代 reviewed malicious-secure construction |
| [RFC 9497](https://www.rfc-editor.org/rfc/rfc9497.html) | 两方 OPRF、VOPRF、POPRF 标准化实例 | RFC 不定义 threshold VOPRF；生产设计需额外 reviewed threshold protocol |
| [SafetyPin, OSDI 2020](https://www.usenix.org/conference/osdi20/presentation/dauterman-safetypin) | HSM 集群下人类 PIN 保护的 encrypted backup，量化真实集群 | 本文无 HSM 集群或同等级实验，不能做优越性结论 |
| [Signal Secure Value Recovery](https://signal.org/blog/secure-value-recovery/) | enclave/key-splitting、retry-count consistency 与 Raft | 本文 freshness prototype 不能等同生产 rollback/guess-limit service |
| [Kintsugi, 2025](https://arxiv.org/abs/2507.21122) | 无专用硬件的 decentralized E2EE key recovery，password-authenticated threshold nodes | 网络恢复不是本文首创，且 Kintsugi 有正式 asynchronous model |
| [Tahoe-LAFS architecture](https://tahoe-lafs.org/trac/tahoe-lafs/browser/docs/architecture.rst) | capability access、encryption、erasure-coded availability | capability + encrypted erasure fragments 不是新贡献；erasure coding 本身不提供 secret sharing confidentiality |
| [PURBs, PoPETs 2019](https://arxiv.org/abs/1806.03160) | 隐藏 encrypted-format metadata 与长度泄漏的 padded blobs | fixed-size opaque object 只能声称采用同类原则；当前 access patterns/IP/timing 仍泄漏 |
| [Apollo, 2025](https://arxiv.org/abs/2507.19484) | social recovery metadata、trustees hidden among indistinguishable non-trustee data | 当前 network profile 不提供 Apollo 的 self-recovery/anonymity-set properties |
| [Horcrux, ACSAC 2017](https://arxiv.org/abs/1706.05085) | 将存储凭据 secret-share 到多服务器并隔离 trusted client component | 本文不存逐服务 password value；但 distributed password-manager storage 已有充分 prior art |

结论：Network Recovery Profile 的可保留贡献只能是 factor-collapse 检查和本项目 lifecycle binding 的具体实例化。它不应成为主论文标题或第一贡献。

## 6. 逐项 novelty 判定

| 拟主张内容 | 判定 | 可接受表述 |
|---|---|---|
| “首次用 DFA 表达密码策略” | 否 | 使用 finite-state policy compiler |
| “首次精确计算 regular policy space” | 否 | 在本系统中实现 exact BigUint cardinality，并用于 entropy/reporting |
| “首次 rank/unrank 合法密码” | 否/未发现密码领域首创证据也不能反推首创 | 实例化已有 ranking/unranking 技术 |
| “首次 rank-then-encipher” | 明确否 | 采用 Bellare et al. 的 rank-then-encipher 范式 |
| “新的 arbitrary-domain PRP” | 否 | 假设 reviewed `DomainPermutation`; prototype 使用有界 backend |
| “数学上无碰撞轮换” | 性质成立，但不是基础新定理 | 在固定 domain/tweak 下由 permutation 单射性得到 intra-epoch non-repetition |
| “输出在合法语言上精确均匀” | 仅在 ideal/random-key permutation 模型下成立 | 对固定 input、随机 permutation key 的 marginal uniformity；不能称实际单 key 序列独立 |
| “服务之间绝不相同” | 否 | HKDF/tweak domain separation 给出计算隔离；跨服务字符串仍可能碰撞 |
| “首次 threshold Root-Key recovery” | 否 | standard Shamir 2-of-3 supporting mechanism |
| “P2P factor-collapse prevention” | 可能是系统性术语/实例化贡献，仍需更广查新 | network share release 的 conjunction 与 lifecycle invariant |

## 7. 投稿可接受性判断

### 可投稿的版本

论文将贡献写成一个经过严格限定、可复现、向后兼容的应用安全协议：

1. 面向实际 enterprise constraints 的 finite-state Policy IR 和 compiler；
2. 对每个 policy 给出 exact `N_P`、rank/unrank 及可验证实现；
3. 采用既有 rank-then-encipher 范式，使 credential generation 在固定 epoch 内无重复；
4. policy epoch/domain exhaustion/history exclusion 的完整语义；
5. 与 threshold-protected random Root Key、CDR、remote evidence 和 v2 migration 的端到端实现；
6. 对 MFDPG artifact、EncoderV2、whole-string rejection 和新方案的可复现实验。

### 高风险、不可投稿的版本

- 标题或摘要称 PSPPD 为“全新密码学置换”；
- 不引用 Black--Rogaway 2002 和 Bellare et al. 2009；
- 把 `Rank -> FPE -> Unrank` 当成本研究原创；
- 把 MFDPG 描述成“必然错误 DFA random walk”而不限定论文/commit；
- 把理想 permutation 下的 marginal uniformity写成实际输出序列独立同分布；
- 忽略 FF1 minimum-domain 与 small-domain security；
- 以本地 microbenchmark 代替企业 compatibility、failure 和 migration evidence。

## 8. 后续查新保留项

正式投稿前仍需在 ACM DL、IEEE Xplore、SpringerLink、USENIX、IACR、Google Patents/Espacenet 和学位论文库以以下组合词再检索一次，并记录查询日期与命中：

```text
password policy + rank-then-encipher
password generation + format-preserving encryption
deterministic password + permutation + rotation
regular language + password + unranking
password history + pseudorandom permutation
policy-compliant credential + arbitrary-domain cipher
```

当前审计没有发现一篇公开论文同时覆盖本文计划中的 policy compiler、exact count、generation-indexed permutation、policy epoch/history、threshold random Root Key 和 evidence-bounded migration；但“未发现完整组合”不等于每个组件或其简单组合具有基础数学新颖性。

