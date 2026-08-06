# PSPPD：正则密码策略空间上的无碰撞确定性派生

> 研究设计说明，版本 0.1（2026-08-06）  
> 本文档独立于当前 JISA 系统论文；现有 Encoder v2 与生产派生路径暂不修改。

## 0. 查新后的关键结论：本构造不能按“新数学算法”投稿

补充检索发现，Bellare、Ristenpart、Rogaway 和 Stegers 在 2009 年的 *Format-Preserving Encryption* 中已经正式提出 `rank-then-encipher`：先把复杂有限格式排序并编号，在整数域上执行 tweakable FPE，再 unrank 回原格式。该文第 5 节进一步直接处理**任意正则语言**，给出的 `BuildTable`、`rank` 和 `unrank` 算法与下文的 DFA 补全计数递推实质相同，并给出 (O(|Q||\Sigma|n)) 预处理和 (O(|\Sigma|n)) rank/unrank 复杂度。换言之：

- “正则密码语言 + 精确计数 + Rank/Unrank”已有明确先例；
- “Rank/Unrank + 有限整数域 PRP/FPE”已有正式安全框架；
- 策略符合、理想置换均匀、同一置换对不同输入不碰撞等结论主要是已有框架的直接推论；
- 把 generation (g) 当作明文编号再 unrank 成密码，是有用的应用映射，但目前没有证据证明它足以构成独立的密码学原语创新。

因此，PSPPD 应降级为**正确的研究基线和系统组件候选**，不能以当前形式宣称“真正可证明的新数学构造”。尚可能形成研究问题的部分是动态策略序列：在策略语言随 epoch 变化、历史窗口受限且不存储历史密码时，如何得到可证明终止、条件均匀、跨 epoch 历史不重复且元数据最小的确定性序列；以及能否证明无状态方案在相交策略语言上不可能保证跨 epoch 不重复。这些问题仍需系统查新，不能在本说明中预设其新颖性。

## 1. 研究问题与创新边界

给定服务密码策略 (P)，令其允许的全部有限长度字符串构成有限语言

\[
\mathcal L_P=\{w\in\Sigma^*:w\models P\}.
\]

目标是构造确定性派生函数

\[
\mathsf{Pwd}(K,T,g,P)\in\mathcal L_P,
\]

其中 (K) 为凭据专用密钥，(T) 为服务、账户和策略上下文，(g) 为凭据代次。构造应满足四项核心性质：策略符合、可精确计算的策略空间、在理想置换模型下对合法集合均匀，以及同一策略周期内的代次无重复。

本研究不把下列已有技术本身作为创新：正则表达式到 DFA 的转换、有限自动机上的计数与均匀生成、ranking/unranking、HKDF、AES 或 FF1。MFDPG 已使用正则策略、DFA 遍历与 HMAC-DRBG 生成确定性密码；已有研究也已讨论正则语言的均匀生成，并形式化验证了符合组合策略的均匀随机密码生成。本文原拟研究、现经查新后只能视为候选应用贡献的是：

1. 将合法密码语言的精确计数与双射编号引入**确定性、服务隔离、可轮换**的密码派生；
2. 将 `credentialGeneration` 作为有限域置换的输入，在明确的 epoch 内给出严格无重复保证；
3. 将策略的规范表示、计数表、排列上下文和生命周期元数据绑定，使“策略变化”成为可验证的域变化，而不是编码器的隐式参数变化；
4. 同时给出信息论结论（有限语言层）与计算安全结论（真实 PRP 层），避免把两者混为一谈。

与 MFDPG 的最近邻差异应表述为：MFDPG 的 Xeger 式随机 DFA 游走解决“确定地产生一个匹配正则策略的字符串”；PSPPD 先对全部可接受字符串精确计数和编号，再在编号域上执行密钥控制置换，从而研究合法集合上的分布和跨代次不重复性。不能仅以“采用 DFA”或“支持任意正则策略”主张创新。

## 2. 策略语言与规范表示

策略编译器接收有界长度策略，并生成确定有限自动机

\[
A_P=(Q,\Sigma,\delta,q_0,F,L_{\min},L_{\max}).
\]

字符表 Σ 的顺序是协议的一部分；DFA 必须完全确定，缺失转移视为拒绝。策略可由多个约束自动机取积得到，例如：

- 长度上下界；
- 字符类别的最小/最大出现次数；
- 固定位置和首尾字符限制；
- 最大连续相同字符数；
- 禁止相邻递增或递减字符；
- 禁用子串、用户名或其他有限禁词；
- 有界长度下的“不重复使用字符”。

禁用子串可用前缀自动机表达，类别计数可用有限计数器表达；在最大长度固定时，它们仍构成有限自动机。“不得重复任意已用字符”需要在状态中保存已用字符集合，最坏产生 (2^{|\Sigma|}) 因子，因此虽然可表达，却未必可计算。PSPPD 的支持边界应定义为：**能够在资源预算内编译成确定有限状态机的有界策略**，而不是不加限制地声称“任意密码策略”。字典检查、泄露口令黑名单的远端查询、上下文语义规则等不天然属于该正则边界。

策略规范对象 `CanonicalPolicy` 至少绑定：字符表及顺序、长度区间、DFA 的规范状态编号与转移表、接受状态、字符串规范化规则和策略格式版本。定义

\[
policyHash=H(\mathsf{CanonicalEncode}(CanonicalPolicy)).
\]

任何会改变语言、排序或规范化的变化都必须产生新的 `policyHash` 和 `policyEpoch`。

## 3. 精确计数

对目标长度 ℓ，定义

\[
C_{\ell}(i,q)=|\{x\in\Sigma^{\ell-i}:\delta^*(q,x)\in F\}|.
\]

边界与递推为

\[
C_{\ell}(\ell,q)=\mathbf 1[q\in F],
\qquad
C_{\ell}(i,q)=\sum_{c\in\Sigma:\delta(q,c)\neq\bot}
C_{\ell}(i+1,\delta(q,c)).
\]

长度 ℓ 的合法字符串数为 (N_{P,\ell}=C_{\ell}(0,q_0))，总空间为

\[
N_P=\sum_{\ell=L_{\min}}^{L_{\max}}N_{P,\ell}.
\]

计数必须使用任意精度非负整数；浮点数既不能表示大策略空间的准确值，也可能在区间选择时引入错误。预计算的时间复杂度为

\[
O((L_{\max}-L_{\min}+1)L_{\max}|Q||\Sigma|),
\]

朴素存储复杂度为 (O((L_{\max}-L_{\min}+1)L_{\max}|Q|))。对固定长度或复用后缀表时可进一步压缩，但不是第一版原型的研究重点。

精确空间大小本身并不自动等于“固定密钥下单个输出的随机熵”。正确表述是：若编号在 《0,(N_P)》 上均匀，则输出的 Shannon 熵和 min-entropy 均为 《log2 (N_P)》；对固定 (K,T,g)，输出是确定值。论文中的安全陈述必须明确随机性来自随机密钥实验或理想随机置换实验。

## 4. Rank 与 Unrank 双射

定义规范顺序：先按长度从小到大，再按 Σ 中字符的既定顺序作字典序。令

\[
\mathcal L_P=(w_0,w_1,\ldots,w_{N_P-1}).
\]

`Unrank_P(r)` 先用各长度的计数确定目标长度，再逐位扫描候选字符。若某候选字符后的合法补全数为 (n_c)，则它对应一个连续编号区间；编号落入该区间时选择该字符，否则减去 (n_c) 后继续。

`Rank_P(w)` 执行逆过程：先累加所有较短长度的空间，再对每一位置累加所有规范序更小字符的补全数。若字符串不能到达接受状态则拒绝。

**命题 1（双射）**：在计数递推正确且规范顺序固定时，

\[
\mathsf{Unrank}_P:[0,N_P)\leftrightarrow\mathcal L_P
\]

为双射，且 `Rank_P` 是其逆函数。

**证明要点**：每一状态的候选边按补全数划分为互不相交、首尾相接的区间，其并集大小由递推式恰为当前状态的补全总数。对剩余长度归纳即可得到存在性和唯一性；多长度情形外层区间同理。

## 5. 策略空间置换

先由 Root Key 派生凭据专用密钥：

\[
K_{cred}=\mathsf{HKDF}(K_{root},
\texttt{"credential-permutation"}\parallel
vaultID\parallel serviceID\parallel accountID\parallel credentialSalt).
\]

置换 tweak 绑定：

\[
T=policyHash\parallel policyEpoch\parallel rootGeneration
\parallel derivationVersion\parallel permutationVersion.
\]

需要一个作用于任意整数域 《0,(N_P)》 的可逆、密钥控制置换 Π。研究原型采用 FF1-AES-256 在二进制超集

\[
D=[0,2^b),\quad b=\lceil\log_2N_P\rceil
\]

上的排列，并用 cycle walking 限制回 《0,(N_P)》：从输入 (x<N_P) 开始反复计算 FF1，直到输出再次小于 (N_P)。由于 (N_P>2^{b-1})，在理想置换启发下平均迭代次数小于 2；但该值不是单次执行的确定上界，因此实现必须设置资源上限并把超限作为错误，而不能降级为取模。

截至本设计日期，NIST SP 800-38G Rev.1 仍为第二轮公开草案；它移除了 FF3，并把 FF1 的最小域提高为 1,000,000。原型因此只允许 (N_P\ge 10^6)，使用 2 的幂 radix 以避免非 2 幂 radix 路径中的浮点运算。该选择是可替换的实例化，不应把 FF1 本身写成论文创新或未经条件限定的“最终标准方案”。

完整派生为

\[
r_g=\Pi_{K_{cred},T}(g),\qquad
password_g=\mathsf{Unrank}_P(r_g),
\]

定义域要求 (0\le g<N_P)。generation 达到 (N_P) 时必须报告策略空间耗尽，禁止回绕。

## 6. 安全性质与准确表述

**推论 1（策略符合性）**：对所有有效输入，`password_g`∈ℒ_P。该结论由 `Unrank` 的值域直接得到，不依赖密码学假设。

**推论 2（epoch 内无重复）**：固定 (K_{cred},T,P)，若 (g_1\ne g_2) 且二者均在 《0,(N_P)》 内，则 `password_g1`≠`password_g2`。因为 Π 与 `Unrank` 均为单射。该结论不跨 `policyEpoch`、`rootGeneration`、`credentialSalt` 或身份上下文成立。

**推论 3（理想模型精确均匀）**：若 Π 从 《0,(N_P)》 上的全部置换中均匀抽取，则对固定 (g) 和任意 (w\in\mathcal L_P)，

\[
\Pr[password_g=w]=1/N_P.
\]

**推论 4（真实 PRP 的计算均匀性）**：若所用 tweakable PRP 在规定查询界限内安全，则 PSPPD 输出与用理想随机置换生成的输出计算上不可区分。真实 AES/FF1 实例不能未经额外证明写成“对密钥精确均匀”。

**推论 5（服务域分离）**：只要 HKDF 的输入编码无歧义且服务/账户/盐唯一，不同凭据获得域分离密钥；结合 tweakable PRP 安全性，可以把不同上下文的排列建模为计算上独立，而不是信息论独立。

该构造提供的是“不重复”，不是“历史密码不可关联”的无条件保证。置换输出对同一密钥的多个 generation 是不放回样本，彼此并非统计独立；这是无重复的必然代价，应在安全模型中明确。

## 7. 策略变化与历史排除

当策略由 (P_1) 变为 (P_2) 时，两种语言可能相交，独立 epoch 的置换不能保证跨 epoch 不重复。若目标要求不得复用最近 (h) 个密码，客户端保留这些历史项的 `(generation, policyEpoch, policyHash, rootGeneration, credentialSalt)`，重新派生历史密码形成排除集合 (E)。新 epoch 使用候选序号 (j=0,1,\ldots)：

\[
p_j=\mathsf{Unrank}_{P_2}(\Pi_{K,T_2}(\mathsf{Encode}(g,j))).
\]

选择第一个 (p_j\notin E) 的值。为了保持严格终止性，`Encode(g,j)` 必须在域内枚举互不重复的输入；当 (N_{P_2}\le|E\cap\mathcal L_{P_2}|) 时直接报告不可满足。不得用无上限的“重新哈希直到不同”替代这一规则。

该排除过程只解决客户端知道的历史值，不能替代远端系统自身的密码历史判定。若远端还实施未知规则，轮换协议仍可能收到拒绝并进入现有的确认/协调状态机。

## 8. 原型、验证与对照实验

第一版原型以规范 DFA 为输入，刻意不实现新的正则表达式解析器：正则到 DFA 已是成熟工具，不是研究贡献。原型提供精确计数、`Rank`、`Unrank`、FF1 cycle-walking 置换及组合派生；所有代码位于 `rust_core::research`，不接入 Encoder v2。

必须保留以下可运行检查：

1. 穷举小 DFA，验证 `Rank(Unrank(r))=r`、计数等于穷举数量、所有输出被 DFA 接受；
2. 在同一 epoch 枚举全部合法 generation，验证 PSPPD 无碰撞；
3. 对 ℒ={a,b0,…,b9} 比较朴素等概率分支游走与按补全数选择，报告总变差距离；
4. 比较当前 Encoder v2、Xeger 式局部随机游走、整体拒绝采样和 PSPPD；
5. 报告 DFA 状态数、计数表大小、预计算时间、派生中位数/P95、cycle-walking 次数和拒绝采样接受率；
6. 对状态数和计数器维度逐步增加的策略测量状态爆炸，设置可复现的编译资源上限；
7. 收集真实企业策略时，只报告已实际编译和测试的策略，不从语法样本外推“全面兼容”。

四种方法的比较必须区分目标：整体拒绝采样在正确选择母空间且随机源均匀时也能产生严格均匀分布，但可能效率很差；MFDPG/Xeger 式游走是否有偏取决于具体转移选择和回溯算法，不能把所有 DFA 游走一概判为有偏；当前 Encoder v2 只证明局部索引采样无取模偏差，尚无合法集合整体均匀证明；PSPPD 的新增价值主要是把双射、置换和 generation 结合成确定性无重复序列。

## 9. 阶段性结论

PSPPD 值得保留为可验证基线，但以下表述只能作为系统应用构造，不能称为新 FPE 或新正则语言算法：

> 对可编译为资源可承受 DFA 的有界密码策略，构造精确计数和规范双射，并用标准有限域 PRP 将凭据代次映射为同一 epoch 内不重复的合法密码；在理想置换模型下获得精确均匀性，在真实 PRP 假设下获得计算不可区分性。

若要形成独立算法论文，下一步应把问题改写为“动态、相交策略语言上的历史受限伪随机序列”，首先证明无状态跨 epoch 不重复的不可能性或状态下界，再提出满足可验证终止和条件分布性质的构造。当前最需要验证的是：（1）是否已有工作把密码轮换计数器、策略演化和历史窗口形式化为同一安全游戏；（2）cycle walking 和可观察派生时间是否泄露与 (N_P) 相关的信息；（3）复杂企业策略下自动机规模是否仍可用；（4）FF1 草案变化及小域安全限制如何影响最终实例化。

## 参考资料（本设计使用）

1. Nair, V.; Song, D. *MFDPG: Multi-Factor Authenticated Password Management With Zero Stored Secrets*, NDSS 2024. <https://arxiv.org/abs/2306.14746>
2. Oudinet, J.; Denise, A.; Genitrini, A. *A new dichotomic algorithm for the uniform random generation of words in regular languages*, Theoretical Computer Science 502 (2013): 165–176. <https://doi.org/10.1016/j.tcs.2012.07.025>
3. NIST. *SP 800-38G Rev.1, Second Public Draft: Methods for Format-Preserving Encryption*, 2025. <https://csrc.nist.gov/pubs/sp/800/38/g/r1/2pd>
4. Grilo, M. et al. *Verified Password Generation from Password Composition Policies*, iFM 2022. <https://joaoff.com/publication/2022/iFM/iFM22-verifiedPwdGen.pdf>
5. RustCrypto. `fpe` 0.6.1 FF1 implementation documentation. <https://docs.rs/fpe/0.6.1/fpe/>
6. Bellare, M.; Ristenpart, T.; Rogaway, P.; Stegers, T. *Format-Preserving Encryption*, Selected Areas in Cryptography 2009. <https://eprint.iacr.org/2009/251>
