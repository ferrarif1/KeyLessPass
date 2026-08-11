# Legacy v2/v3 removal audit

The initial pre-rewrite pass recorded 181 matches (DELETE 53; REWRITE 27;
KEEP_INTERNAL_ONLY 101). This file was refreshed after the rewrite so that its
inventory also records the final location and disposition of product-only
compatibility material and revision reports.

- Audit date: 2026-08-09
- Repository commit before the rewrite: `8ed2fb8d75c412987eb9c47219393e02e96aa6c4`
- Scope: manuscript, bibliography, Rust source, experiments, formal models, figures/tables, vectors, READMEs, CI, and supplementary documentation
- Initial pre-rewrite matches: 181 (DELETE 53; REWRITE 27; KEEP_INTERNAL_ONLY 101)
- Refreshed matches: 165 (DELETE 53; REWRITE 11; KEEP_INTERNAL_ONLY 101)

## Disposition rules

- **DELETE**: remove from the standalone paper and submitted research artifact; do not rename it as a baseline.
- **REWRITE**: retain the exact-policy-space function but rename it as public `schemeVersion = 1` / `EXACT_POLICY_V1`, with no migration narrative.
- **KEEP_INTERNAL_ONLY**: required by the existing product or historical tests, but excluded from the paper build, paper experiments, formal model, fixed vectors, and review artifact manifest.

The `rust-cache@v2` CI matches are third-party GitHub Action release tags, not protocol versions. Evidence-harness uses of “v1/v2” that denote database record revisions are also internal product history, not research baselines.

## Complete occurrence inventory

| Disposition | Location | Matched text |
|---|---|---|
| KEEP_INTERNAL_ONLY | `README.md:45` | It is not a web app, browser extension, cloud password manager, or password vault. KeyLessPass does not store target-system plaintext passwords or maintain an encrypted service-password database. Its v3 paper recovery share is an offline encoding of a random high-entropy Shamir share; it is not a phrase users are expected to memorize and is not persisted by the application. |
| KEEP_INTERNAL_ONLY | `README.md:77` | - Verified migration from legacy v2 pairwise complete-key wrappers to v3 shares without changing the Root Key or derived service passwords. |
| KEEP_INTERNAL_ONLY | `README.md:105` | \| Canonical versioned CDR metadata, salts, state, replica metadata, and MAC tags \| Plaintext Root Key in any persisted v3 object \| |
| KEEP_INTERNAL_ONLY | `README.md:106` | \| Legacy v2 pairwise wrappers only until an explicit verified migration archives them \| Recovery-share phrase (the application displays but does not persist it) \| |
| KEEP_INTERNAL_ONLY | `README.md:110` | In the selectable v3 recovery schema, enrollment or migration starts with a random 256-bit Root Key and uses the external \`vsss-rs\` finite-field implementation to split it into three shares at threshold two. The dependency currently describes itself as under audit; KeyLessPass does not claim an independent audit result: |
| KEEP_INTERNAL_ONLY | `README.md:120` | Legacy v2 profiles retain a read-only pairwise-wrapper path solely for compatibility and verified migration. New v3 recovery takes precedence once its manifest is committed. The current Flutter enrollment screens still create v2 data, so selecting v3 currently requires the Rust migration API; this is an explicit prototype limitation. |
| KEEP_INTERNAL_ONLY | `README.md:145` | - The v3 recovery-share phrase is not stored by the application. |
| KEEP_INTERNAL_ONLY | `README.md:146` | - The Root Key is not persisted in any v3 local or USB payload. |
| KEEP_INTERNAL_ONLY | `README.zh-CN.md:3` | > **v3 研究实现说明：** Rust Core 已提供经过认证和代际绑定的 Shamir 2-of-3 Root Key 恢复、CDR v3、无取模偏差的策略编码、证据有界轮换及新鲜度接口。本文后续部分若出现 \`W_MC\`、\`W_MU\`、\`W_CU\`，描述的是仍由当前 Flutter 初始化界面生成的 legacy v2 格式；它们不是 secret shares，仅用于兼容和迁移。v3 当前通过 Rust 迁移接口启用，完整桌面 UX 尚未交付。 |
| KEEP_INTERNAL_ONLY | `.github/workflows/research-core.yml:31` | - uses: Swatinem/rust-cache@v2 |
| KEEP_INTERNAL_ONLY | `.github/workflows/research-core.yml:72` | - uses: Swatinem/rust-cache@v2 |
| KEEP_INTERNAL_ONLY | `docs/SECURITY.md:3` | The normative security boundary is [../SECURITY.md](../SECURITY.md). The v3 design uses authenticated and generation-bound Shamir 2-of-3 shares; legacy \`W_MC\`, \`W_MU\`, and \`W_CU\` packages are migration-only v2 data and are not secret shares. |
| REWRITE | `rust_core/src/derivation/mod.rs:95` | b"credential-permutation/v3", |
| REWRITE | `rust_core/src/derivation/mod.rs:99` | return Err(validation("v3 credential salt must contain 128 bits")); |
| REWRITE | `rust_core/src/derivation/mod.rs:102` | domain: "KeyLessPass/credential-key/v3", |
| REWRITE | `rust_core/src/derivation/mod.rs:120` | domain: "KeyLessPass/policy-space-permutation/v3", |
| REWRITE | `rust_core/src/derivation/mod.rs:140` | return Err(validation("record is not a derivation/encoder v3 record")); |
| REWRITE | `rust_core/src/derivation/mod.rs:147` | return Err(validation("v3 derivation identifiers must be non-nil")); |
| REWRITE | `rust_core/src/derivation/mod.rs:150` | return Err(validation("v3 derivation requires policyEpoch")); |
| REWRITE | `rust_core/src/derivation/mod.rs:241` | "../../test-vectors/password-derivation-v3.json" |
| KEEP_INTERNAL_ONLY | `rust_core/src/crypto/kdf.rs:103` | let factor = hkdf_32(&stretched, salt, b"KeyLessPass mnemonic factor v2"); |
| KEEP_INTERNAL_ONLY | `rust_core/src/crypto/kdf.rs:128` | hkdf_32(&ikm, salt, b"KeyLessPass computer factor v2") |
| KEEP_INTERNAL_ONLY | `rust_core/src/crypto/kdf.rs:141` | hkdf_32(&ikm, salt, b"KeyLessPass USB factor v2") |
| KEEP_INTERNAL_ONLY | `rust_core/src/crypto/kdf.rs:150` | b"KeyLessPass pairwise wrapper salt v2", |
| KEEP_INTERNAL_ONLY | `rust_core/src/crypto/kdf.rs:164` | hkdf_32(master_key, b"", b"KeyLessPass v2 derivation key") |
| KEEP_INTERNAL_ONLY | `rust_core/src/crypto/kdf.rs:188` | /// Returns the RFC 8785 input authenticated by the v3 password-derivation HMAC. |
| KEEP_INTERNAL_ONLY | `rust_core/src/crypto/kdf.rs:201` | "v3 derivation requires non-nil service, account, and policy identifiers".to_string(), |
| KEEP_INTERNAL_ONLY | `rust_core/src/crypto/kdf.rs:207` | "v3 credential salt must contain 128 bits".to_string(), |
| KEEP_INTERNAL_ONLY | `rust_core/src/crypto/kdf.rs:229` | /// Derives the 256-bit deterministic seed consumed by encoder v2 for CDR v3. |
| KEEP_INTERNAL_ONLY | `rust_core/src/crypto/kdf.rs:339` | hasher.update(b"KeyLessPass mnemonic Argon2id salt v2"); |
| KEEP_INTERNAL_ONLY | `rust_core/src/crypto/kdf.rs:378` | "../../test-vectors/password-derivation-v2.json" |
| KEEP_INTERNAL_ONLY | `rust_core/src/domain/cdr.rs:342` | /// Creates an explicit encoder/derivation-v3 candidate without changing v2 semantics. |
| KEEP_INTERNAL_ONLY | `rust_core/src/domain/cdr.rs:343` | /// The credential salt is stable inside the v3 permutation domain. |
| KEEP_INTERNAL_ONLY | `rust_core/src/domain/cdr.rs:612` | let expected = include_str!("../../test-vectors/cdr-v3-rfc8785.json").trim(); |
| KEEP_INTERNAL_ONLY | `docs/novelty_literature_audit.md:132` | 5. 与 threshold-protected random Root Key、CDR、remote evidence 和 v2 migration 的端到端实现； |
| KEEP_INTERNAL_ONLY | `rust_core/examples/evidence_harness.rs:231` | "Create a pending rotation version and derive v1 and v2.", |
| KEEP_INTERNAL_ONLY | `rust_core/examples/evidence_harness.rs:232` | "v2 password differs from v1.", |
| KEEP_INTERNAL_ONLY | `rust_core/examples/evidence_harness.rs:248` | ok("Pending v2 derivation differed from active v1.") |
| KEEP_INTERNAL_ONLY | `rust_core/examples/evidence_harness.rs:310` | ok("In-place descriptor mutation was rejected and rotation created v2.") |
| KEEP_INTERNAL_ONLY | `rust_core/examples/evidence_harness.rs:501` | "Create a pending rotation and inspect v1/v2 states.", |
| KEEP_INTERNAL_ONLY | `rust_core/examples/evidence_harness.rs:502` | "v1 remains active and v2 is pending_rotation.", |
| KEEP_INTERNAL_ONLY | `rust_core/examples/evidence_harness.rs:522` | ok("v1 remained active while v2 was pending_rotation.") |
| KEEP_INTERNAL_ONLY | `rust_core/examples/evidence_harness.rs:533` | "v2 active; v1 retired.", |
| KEEP_INTERNAL_ONLY | `rust_core/examples/evidence_harness.rs:571` | ok("v2 became active and v1 was retired after commit.") |
| KEEP_INTERNAL_ONLY | `docs/LIMITATIONS.md:3` | - The Flutter desktop screens and generated localization still use “mnemonic” wording in several flows. The Rust v3 artifact is an offline paper recovery share, but complete UI terminology and lifecycle conversion is unfinished. |
| KEEP_INTERNAL_ONLY | `docs/SECURITY.zh-CN.md:28` | ## V2 包说明 |
| KEEP_INTERNAL_ONLY | `docs/SECURITY.zh-CN.md:32` | - V2 \`encryptedPayload\` 是历史字段名，现在是 base64 编码因子 payload，不包含明文 \`Kmaster\`。 |
| KEEP_INTERNAL_ONLY | `rust_core/src/crypto/encoder.rs:12` | let mut stream = DeterministicStream::new(secret, b"KeyLessPass/password-encoder/v2"); |
| KEEP_INTERNAL_ONLY | `docs/DESIGN.md:1` | # KeyLessPass v3 Design |
| KEEP_INTERNAL_ONLY | `docs/DESIGN.md:11` | The v3 Root Key is 256 random bits. Purpose-specific keys are derived with HKDF-SHA256 using fixed labels and context binding to \`vaultID\`, \`rootGeneration\`, and \`cryptoSuiteVersion\`. The Root Key is split by \`vsss-rs\` into three GF(256) Shamir shares at threshold two: |
| KEEP_INTERNAL_ONLY | `docs/DESIGN.md:19` | Legacy v2 pairwise complete-key wrappers remain readable only for migration. Migration validates all three legacy paths, preserves the Root Key, writes and validates v3 artifacts, commits the v3 manifest, and can archive the old packages. The current Flutter enrollment workflow still creates v2 packages; v3 selection currently uses the Rust migration operation. |
| KEEP_INTERNAL_ONLY | `docs/DESIGN.md:23` | CDR v3 uses RFC 8785 JSON canonicalization. \`recordID\` is a stable logical credential identity; \`recordSeq\` is a stable vault-local ordinal retained for deterministic compatibility and human/audit ordering. \`credentialGeneration\` advances one service password, while \`rootGeneration\` advances the vault Root Key. |
| KEEP_INTERNAL_ONLY | `docs/DESIGN.md:25` | Password-changing inputs are the Root-Key generation, stable service/account identifiers, credential generation, 128-bit salt, derivation/encoder versions, and the hash of policy identity, version, and encoding descriptor. \`recordID\`, \`recordSeq\`, storage \`version\`, display fields, notes, replica clocks, and rotation evidence do not alter derivation-v2 output. Any encoding-policy change requires a new credential generation. |
| KEEP_INTERNAL_ONLY | `docs/DESIGN.md:27` | Encoder v2 uses a domain-separated HMAC stream, rejection sampling for modulo-bias-free bounded indices, Fisher--Yates shuffling with the same index sampler, randomized mandatory-character placement, explicit min/max class counts and edge/repetition/sequence rules, contradiction detection, and a bounded attempt count. It never silently relaxes a policy. This does not imply uniform sampling over the complete set of policy-valid strings; reported entropy is an upper bound when policy constraints overlap. |
| KEEP_INTERNAL_ONLY | `docs/PRODUCTIZATION_REPORT.zh-CN.md:46` | - V2 \`encryptedPayload\` 是历史字段名，承载 base64 因子 payload，不是助记短语加密 vault。 |
| DELETE | `docs/final_research_restructure_report.md:22` | 6. 已接入 CDR v3、显式 v2-to-v3 pending migration、policy epoch、历史排除 |
| DELETE | `docs/final_research_restructure_report.md:23` | 和现有证据状态机；v2 \`(2,2)\` 路径保持不变。 |
| DELETE | `docs/final_research_restructure_report.md:32` | - v3 的 \`credentialGeneration\` 是置换输入，不进入 tweak。 |
| DELETE | `docs/final_research_restructure_report.md:37` | - migration 只创建 pending v3 记录，不自动提交或覆盖 v2 active 记录。 |
| DELETE | `docs/final_research_restructure_report.md:48` | - 分布：254 输出、每种 10 万样本；Encoder v2 的经验 TVD 为 0.06668， |
| DELETE | `docs/final_research_restructure_report.md:50` | - 碰撞：玩具域全 254 generations 中 Encoder v2 观察到 93 个碰撞；双射 |
| DELETE | `docs/final_research_restructure_report.md:51` | oracle 为 0；真实 v3 后端 2,000 generations 观察到 0。 |
| DELETE | `docs/final_research_restructure_report.md:52` | - 性能：18 位策略完整 v3 派生中位数约 76 微秒、P95 约 171 微秒。 |
| KEEP_INTERNAL_ONLY | `docs/security/2-of-3-recovery-implementation-notes.md:33` | ## V2 \`encryptedPayload\` Field |
| KEEP_INTERNAL_ONLY | `docs/security/2-of-3-recovery-implementation-notes.md:36` | - In V2, this is a historical field name. It stores a base64 encoded factor |
| KEEP_INTERNAL_ONLY | `docs/security/2-of-3-recovery-implementation-notes.md:41` | - \`Kmaster\` may only appear in V2 packages inside the \`W_MC\`, \`W_MU\`, and |
| KEEP_INTERNAL_ONLY | `docs/CDR_SPEC.md:1` | # Credential Description Record Specification v3 |
| KEEP_INTERNAL_ONLY | `docs/CDR_SPEC.md:5` | CDR v3 uses RFC 8785 JSON Canonicalization Scheme through \`serde_json_canonicalizer 0.3.2\`. The HMAC input is the canonical object with \`macTag\` set to the empty string. \`K_cdr_authentication\` is derived from the Root Key with \`vaultID\`, \`rootGeneration\`, and \`cryptoSuiteVersion\`. Ordinary JSON member order is never security-significant. |
| KEEP_INTERNAL_ONLY | `docs/CDR_SPEC.md:19` | \| \`recordID\` \| Stable logical credential identity across generations \| No in derivation v2; authenticated administrative identity \| |
| KEEP_INTERNAL_ONLY | `docs/CDR_SPEC.md:20` | \| \`recordSeq\` \| Human/audit ordering within a vault; not a freshness proof \| No in derivation v2; yes only for backward-compatible derivation v1 \| |
| KEEP_INTERNAL_ONLY | `docs/CDR_SPEC.md:31` | \| \`version\` \| Storage row version retained for migration and lookup \| No in derivation v2 \| |
| KEEP_INTERNAL_ONLY | `docs/CDR_SPEC.md:70` | are deliberately excluded. Encoder v2 expands \`seed\` as |
| KEEP_INTERNAL_ONLY | `docs/CDR_SPEC.md:71` | \`HMAC-SHA-256(seed, "KeyLessPass/password-encoder/v2" \|\| u64be(blockCounter))\`; |
| KEEP_INTERNAL_ONLY | `docs/CDR_SPEC.md:73` | is \`rust_core/test-vectors/password-derivation-v2.json\`. |
| KEEP_INTERNAL_ONLY | `docs/CDR_SPEC.md:81` | The decoder supplies explicit defaults only for legacy records. Legacy MAC verification reconstructs the exact v2 field layout and JSON order. Migration verifies the legacy MAC before assigning v3 identities and re-authenticating the canonical object. Unknown schema, suite, encoder, or derivation versions must fail closed rather than downgrade. |
| KEEP_INTERNAL_ONLY | `docs/adr/ADR-001-AUTHENTICATED-SHAMIR-RECOVERY.md:8` | The v2 system encrypted a complete Root Key once for each authorized factor pair. Reviewers correctly noted that this was not threshold secret sharing. The replacement must fit three passive local factors, be explainable and testable, and avoid new cryptographic mathematics. |
| KEEP_INTERNAL_ONLY | `docs/adr/ADR-001-AUTHENTICATED-SHAMIR-RECOVERY.md:28` | - Migration reads and verifies all three legacy paths before committing v3. The service password does not change during wrapper-to-share migration. |
| KEEP_INTERNAL_ONLY | `rust_core/src/service/derivation_migration.rs:52` | "v3 migration requires one active committed credential", |
| KEEP_INTERNAL_ONLY | `rust_core/src/service/derivation_migration.rs:105` | return Err(validation("candidate is not a complete v3 record")); |
| DELETE | `docs/evaluation_results.md:36` | \| Encoder v2 \| 0.06668 \| 3011.61 \| 2.91 \| |
| DELETE | `docs/evaluation_results.md:41` | 后两项的差异处于本次有限样本噪声量级；数据支持“Encoder v2 和该等权 |
| DELETE | `docs/evaluation_results.md:48` | 在 254 个输出的玩具域上测试完整 254 个 generation：Encoder v2 观察到 |
| DELETE | `docs/evaluation_results.md:50` | 真实 v3 原型后端又在默认大策略域上检查 2,000 个连续 generation，观察 |
| DELETE | `docs/evaluation_results.md:60` | 与 Unrank 的中位数约 1--2 µs，完整 v3 派生中位数约 76 µs、P95 约 |
| KEEP_INTERNAL_ONLY | `docs/security/2-of-3-recovery-implementation-notes.zh-CN.md:31` | ## V2 \`encryptedPayload\` |
| KEEP_INTERNAL_ONLY | `docs/security/2-of-3-recovery-implementation-notes.zh-CN.md:34` | - 在 V2 中，它是历史字段名，保存 base64 编码因子 payload，不是助记短语加密 vault，也不是平台加密 vault。 |
| KEEP_INTERNAL_ONLY | `docs/security/2-of-3-recovery-implementation-notes.zh-CN.md:37` | - \`Kmaster\` 在 V2 包中只能出现在 \`W_MC\`、\`W_MU\`、\`W_CU\` wrapper 密文里。 |
| KEEP_INTERNAL_ONLY | `docs/MIGRATION.md:1` | # Pairwise-Wrapper to Shamir v3 Migration |
| KEEP_INTERNAL_ONLY | `docs/MIGRATION.md:5` | Migration does not change the Root Key or any service password. It verifies that recovery+computer, recovery+USB, and computer+USB wrappers all produce the same 256-bit key before writing v3. |
| KEEP_INTERNAL_ONLY | `docs/MIGRATION.md:15` | 7. Write the local v3 manifest last; this is the commit point. |
| KEEP_INTERNAL_ONLY | `docs/MIGRATION.md:35` | Repeat with \`dryRun: false\`. The successful response contains the new recovery share phrase exactly once; it is deliberately excluded from the audit JSON. Save it offline before archiving v2. \`archiveLegacyWrappers\` is recoverable archival, not guaranteed secure deletion on flash or copy-on-write filesystems. |
| KEEP_INTERNAL_ONLY | `docs/MIGRATION.md:37` | After v3 manifest commit, password derivation prefers v3 and interprets the input phrase as a recovery-share phrase. Mixing v2 and v3 factors is rejected. |
| KEEP_INTERNAL_ONLY | `rust_core/src/research/psppd.rs:3` | //! The production Encoder v2 does not call this module. The prototype starts from a |
| KEEP_INTERNAL_ONLY | `docs/ARCHITECTURE_DIAGNOSIS.md:1` | # Architecture Diagnosis Before the v3 Refactor |
| KEEP_INTERNAL_ONLY | `docs/ARCHITECTURE_DIAGNOSIS.md:7` | The v2 implementation generated one random 256-bit \`K_master\` and three factor values: \`F_M\` from a user-entered mnemonic processed by Argon2id and HKDF, \`F_C\` from the platform/device secret, and \`F_U\` from USB package material. It then derived three pair keys and encrypted the complete \`K_master\` three times: |
| KEEP_INTERNAL_ONLY | `docs/ARCHITECTURE_DIAGNOSIS.md:38` | The v3 core makes authenticated, version-bound Shamir 2-of-3 shares the selectable recovery schema and keeps the old wrapper reader only for migration. A v3 committed manifest takes precedence during password derivation. Archived v2 files are explicitly labelled deprecated. Full desktop UX conversion from “mnemonic” to “recovery share phrase” remains a deployment task and is not claimed as completed; see \`LIMITATIONS.md\`. |
| KEEP_INTERNAL_ONLY | `docs/KEY_HIERARCHY.md:3` | Crypto suite 1 uses a uniformly random 256-bit Root Key and HKDF-SHA-256 for high-entropy key separation. HKDF is not used as password hardening. User-entered legacy mnemonics are handled only by the v2 migration reader with Argon2id. |
| KEEP_INTERNAL_ONLY | `docs/KEY_HIERARCHY.md:16` | \| \`K_root\` \| 256 bit \| OS CSPRNG \| Root for all vault subkeys \| Never stored whole in v3; reconstructed from two shares \| Shamir at rest; KCV after recovery \| \`rootGeneration\`; rotate after threshold compromise \| All current vault-derived secrets exposed \| Vault cannot derive passwords without two valid shares \| |
| KEEP_INTERNAL_ONLY | `docs/DESIGN.zh-CN.md:1` | # KeyLessPass v3 设计说明 |
| KEEP_INTERNAL_ONLY | `docs/DESIGN.zh-CN.md:5` | v3 使用 256 位随机 Root Key，并由 \`vsss-rs\` 实现的 GF(256) Shamir 2-of-3 方案拆分为恢复份额、本机份额和 U 盘份额。恢复份额编码为带校验和的 108 个英文单词；本机份额由平台能力保护；普通 U 盘只作为可复制的份额文件载体。份额封装绑定 Vault、Root Key 代际、share set、因素类型/标识/代际及密码套件。重建后使用 Root-Key 派生的 HMAC 校验封装，并用 KCV 验证 Root Key。 |
| KEEP_INTERNAL_ONLY | `docs/DESIGN.zh-CN.md:7` | 旧 \`W_MC\`、\`W_MU\`、\`W_CU\` 是 v2 成对完整密钥 wrapper，不是 secret share，仅保留迁移读取。迁移验证三条旧路径得到相同 Root Key，再写入并验证 v3 份额，最后提交 manifest，因此不改变已有服务密码。 |
| KEEP_INTERNAL_ONLY | `docs/DESIGN.zh-CN.md:9` | CDR v3 使用 RFC 8785 JSON 规范化，显式记录 Vault、服务、账号、凭据代际、Root Key 代际、策略/编码器版本、轮换契约、证据集、操作 ID、父哈希及副本元数据。编码器使用 HMAC 确定性流、拒绝采样和无取模偏差的 Fisher--Yates 索引选择；这不等于已证明在全部合法密码上整体均匀。 |
| KEEP_INTERNAL_ONLY | `docs/DESIGN.zh-CN.md:15` | 当前限制：Flutter 初始化仍生成 v2 数据，v3 需通过 Rust 迁移接口启用；非空 Vault 的 Root Key 全量轮换、生产 freshness 服务、目标系统适配器和完整 v3 桌面恢复界面尚未交付。详见 [LIMITATIONS.md](LIMITATIONS.md)。 |
| KEEP_INTERNAL_ONLY | `rust_core/test-vectors/password-derivation-v2.json:11` | "derivationVersion": 2, |
| KEEP_INTERNAL_ONLY | `rust_core/test-vectors/password-derivation-v2.json:13` | "encoderVersion": 2, |
| DELETE | `docs/research_restructure_audit.md:8` | 当前项目不是需要推倒重来，而是需要更换研究主轴并补齐一个严格版本化的算法层：保留随机 256-bit Root Key、Shamir 2-of-3、认证恢复对象、CDR、远端证据约束轮换和复制/新鲜度语义；将 EncoderV2 固定为兼容算法，新建 \`derivationVersion = 3\`、\`encoderVersion = 3\` 的 exact policy-space 路径。 |
| DELETE | `docs/research_restructure_audit.md:10` | 仓库中的 \`research::psppd\` 已验证“有限 DFA + BigUint 动态规划 + rank/unrank”的最小可行性，但仍是研究原型，不是可以直接写进论文的完成方案。尤其是：它只接收手工构造 DFA；策略覆盖范围很窄；没有 \`DomainPermutation\` 抽象；FF1-on-binary-superset 的 cycle-walking 仅是带边界的原型；没有 CDR v3 接入、\`policyEpoch\`、跨 epoch 历史排除、迁移或固定向量。 |
| DELETE | `docs/research_restructure_audit.md:35` | 需修正的只是命名：代码函数名 \`password_derivation_input_v3\` / \`derive_service_secret_v3\` 中的 “v3” 指 CDR schema v3，而字段 \`derivationVersion\` 实际仍为 2。新算法接入前必须消除这层歧义。 |
| DELETE | `docs/research_restructure_audit.md:151` | - v3 canonical policy 表示或稳定 policy hash； |
| DELETE | `docs/research_restructure_audit.md:164` | (derivationVersion=1, legacy encoder) |
| DELETE | `docs/research_restructure_audit.md:165` | (derivationVersion=2, encoderVersion=2) |
| DELETE | `docs/research_restructure_audit.md:166` | (derivationVersion=3, encoderVersion=3) |
| DELETE | `docs/research_restructure_audit.md:176` | - \`cdr-v3-rfc8785.json\`； |
| DELETE | `docs/research_restructure_audit.md:177` | - \`password-derivation-v2.json\`； |
| DELETE | `docs/research_restructure_audit.md:184` | 1. 完全冻结 EncoderV2 代码路径和 v2 固定向量； |
| DELETE | `docs/research_restructure_audit.md:185` | 2. 新增 v3 policy descriptor/IR、canonical vector、derivation context vector、permutation vector 和 end-to-end password vector； |
| DELETE | `docs/research_restructure_audit.md:186` | 3. 旧 CDR 反序列化时不得通过默认字段偷偷进入 v3； |
| DELETE | `docs/research_restructure_audit.md:187` | 4. v3 仅由显式远端轮换创建，候选密码经远端证据确认后才 commit。 |
| DELETE | `docs/research_restructure_audit.md:195` | ### 8.2 显式 v2 -> v3 轮换 |
| DELETE | `docs/research_restructure_audit.md:199` | 1. 读取并验证 active v2 CDR； |
| DELETE | `docs/research_restructure_audit.md:200` | 2. 编译目标 v3 policy，计算 \`N_P\`，验证 \`credentialGeneration < N_P\`； |
| DELETE | `docs/research_restructure_audit.md:201` | 3. 创建 pending v3 CDR，设置 \`derivationVersion = 3\`、\`encoderVersion = 3\`、\`policyEpoch = 1\`； |
| DELETE | `docs/research_restructure_audit.md:202` | 4. 用 v3 算法派生 candidate password； |
| DELETE | `docs/research_restructure_audit.md:204` | 6. 原子提交 v3 CDR，保留 v2 predecessor 以支持 reconciliation/audit； |
| DELETE | `docs/research_restructure_audit.md:205` | 7. 失败或不确定时仍以 v2 committed record 为当前状态。 |
| DELETE | `docs/research_restructure_audit.md:207` | 迁移不能复用当前 \`service::migration\` 的名称和语义：现有模块处理 pairwise recovery wrapper -> Shamir share set，而不是 password encoder v2 -> v3。两者应拆分命名。 |
| DELETE | `docs/research_restructure_audit.md:223` | - v2 fixed vectors 原样通过，v3 新向量独立加入； |
| KEEP_INTERNAL_ONLY | `docs/REVIEW_REVISION_CHECKLIST.md:42` | exploratory code remains out of scope and is not part of derivation v3. |
| KEEP_INTERNAL_ONLY | `docs/PRODUCTIZATION_REPORT.md:61` | - V2 \`encryptedPayload\` is retained as a historical schema field name for base64 encoded factor payloads, not for a mnemonic-encrypted USB vault. |
| KEEP_INTERNAL_ONLY | `docs/FINAL_REFACTOR_REPORT.md:9` | The refactor corrects those claims and implements a substantially more rigorous v3 core. It does **not** complete every item in \`改进.md\`. The defensible research contribution is now a cross-layer lifecycle protocol that joins recovery generations, credential generations, remote-outcome evidence, replica ancestry, and optional freshness state. Shamir, HKDF, HMAC, JCS, and unbiased sampling are foundations, not claimed innovations. |
| KEEP_INTERNAL_ONLY | `docs/FINAL_REFACTOR_REPORT.md:15` | The v3 data model replaces complete-key wrappers with three 33-byte shares. Envelopes bind the vault, Root-Key generation, share set, index, threshold/count, factor role/ID/generation, suite, timestamp, and phrase encoding. Root-Key-derived HMAC authenticates the envelope after reconstruction and a KCV confirms the Root Key. Generation-specific files are validated before a manifest-last commit. |
| KEEP_INTERNAL_ONLY | `docs/FINAL_REFACTOR_REPORT.md:17` | Legacy v2 remains a migration reader only. Migration dry-runs, validates all three legacy paths, preserves the Root Key, validates v3, commits v3, writes a phrase-redacted audit record, and can copy/verify/archive the old local and USB packages. This preserves existing service passwords. |
| KEEP_INTERNAL_ONLY | `docs/FINAL_REFACTOR_REPORT.md:25` | \| Mature 2-of-3 recovery/all pairs \| Complete in v3 core \| \`crypto/recovery.rs\`, \`recovery_store.rs\` \| Flutter enrollment still creates v2 \| |
| KEEP_INTERNAL_ONLY | `docs/FINAL_REFACTOR_REPORT.md:30` | \| v2 pairwise migration \| Core path complete \| dry-run, all-path verification, commit, archive/audit test \| Exhaustive interruption recovery is not complete \| |
| KEEP_INTERNAL_ONLY | `docs/FINAL_REFACTOR_REPORT.md:31` | \| CDR formal schema/canonical serialization \| Complete in core \| CDR v3, RFC 8785 JCS, fixed vector \| Dedicated external schema file/code generator absent \| |
| KEEP_INTERNAL_ONLY | `docs/FINAL_REFACTOR_REPORT.md:43` | \| README/design/security/changelog \| Updated \| root and \`docs/\` files \| Some legacy product/user-guide pages still describe v2 and are historical \| |
| KEEP_INTERNAL_ONLY | `rust_core/test-vectors/cdr-v3-rfc8785.json:1` | {"accountHint":"operator","accountId":"bbbbbbbb-1111-2222-3333-444444444444","createdAt":"2026-08-06T00:00:00Z","credentialGeneration":3,"cryptoSuiteVersion":1,"derivationVersion":1,"displayName":"Example","encoderVersion":2,"encodingDescriptor":{"allowedAlphabet":"ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz23456789!@#$%*-_=+","alphabetProfile":"enterprise-balanced","fixedPositions":[],"forbidRepeatedCharacters":false,"forbidSequentialCharacters":false,"forbiddenChars":"\"'\`\\/:;?&<>{}[]()\|, ","forbiddenFirstChars":"","forbiddenLastChars":"","length":18,"maxAttempts":1024,"normalization":"none","requiredClasses":[{"alphabet":"ABCDEFGHJKLMNPQRSTUVWXYZ","maxCount":null,"minCount":1,"name":"upper","position":null},{"alphabet":"abcdefghijkmnopqrstuvwxyz","maxCount":null,"minCount":1,"name":"lower","position":null},{"alphabet":"23456789","maxCount":null,"minCount":1,"name":"digit","position":null},{"alphabet":"!@#$%*-_=+","maxCount":null,"minCount":1,"name":"symbol","position":null}],"ruleVersion":2},"macTag":"Jaabyj4sPokAwILbxaLHCNv405amyV/5cUCWJu9Vg1g=","notes":"vector","operationId":null,"parentRecordHash":"parent","policyId":"cccccccc-1111-2222-3333-444444444444","policyVersion":2,"recordId":"11111111-2222-3333-4444-555555555555","recordSeq":42,"replica":{"epoch":5,"lamportClock":8,"replicaId":"dddddddd-1111-2222-3333-444444444444"},"retiredAt":null,"rootGeneration":2,"rotationState":"STABLE","salt":"EREREREREREREREREREREQ==","schemaVersion":3,"serviceHint":"legacy.example","serviceId":"aaaaaaaa-1111-2222-3333-444444444444","state":"active","updatedAt":"2026-08-06T00:00:00Z","vaultId":"00112233-4455-6677-8899-aabbccddeeff","version":3} |
| REWRITE | `rust_core/test-vectors/password-derivation-v3.json:54` | "derivationVersion": 3, |
| REWRITE | `rust_core/test-vectors/password-derivation-v3.json:55` | "domain": "KeyLessPass/policy-space-permutation/v3", |
| REWRITE | `rust_core/test-vectors/password-derivation-v3.json:56` | "encoderVersion": 3, |
| DELETE | `docs/migration_v2_v3.md:1` | # EncoderV2 to Exact Policy-Space V3 Migration |
| DELETE | `docs/migration_v2_v3.md:8` | derivationVersion=2, encoderVersion=2 -> frozen EncoderV2 |
| DELETE | `docs/migration_v2_v3.md:9` | derivationVersion=3, encoderVersion=3 -> exact policy-space derivation |
| DELETE | `docs/migration_v2_v3.md:13` | Legacy pre-CDR-v3 records retain their historical derivation path. The existing v2 fixed vector remains authoritative. |
| DELETE | `docs/migration_v2_v3.md:17` | 1. Authenticate and load the current committed v2 CDR. |
| DELETE | `docs/migration_v2_v3.md:18` | 2. Compile the chosen v3 policy and reject empty/unsupported/state-exploding policies. |
| DELETE | `docs/migration_v2_v3.md:19` | 3. Create a pending successor with \`policyEpoch=1\`, epoch-local \`credentialGeneration=0\`, and versions \`(3,3)\`. |
| DELETE | `docs/migration_v2_v3.md:21` | 5. Re-derive the configured password-history window. If the v3 candidate equals a predecessor, consume the next generation index until a non-equal candidate is found or the domain is exhausted. |
| DELETE | `docs/migration_v2_v3.md:23` | 7. Commit only after the contract establishes \`new succeeds && old conclusively fails\`; otherwise the v2 record remains committed. |
| DELETE | `docs/migration_v2_v3.md:27` | ## Subsequent v3 rotation |
| DELETE | `docs/migration_v2_v3.md:36` | V2 vectors are immutable. V3 adds independent vectors for canonical policy bytes/hash, credential key, tweak (which excludes \`credentialGeneration\`), finite-domain permutation rank and final password. A new compiler/canonicalization rule requires a new encoder version, not replacement of a vector. |
| KEEP_INTERNAL_ONLY | `rust_core/src/service/migration.rs:84` | source_schema: "pairwise-complete-key-wrappers-v2".to_string(), |
| KEEP_INTERNAL_ONLY | `rust_core/src/service/migration.rs:85` | target_schema: "authenticated-shamir-2-of-3-v3".to_string(), |
| KEEP_INTERNAL_ONLY | `rust_core/src/service/migration.rs:118` | &paths.app_dir.join("recovery-migration-v3-audit.json"), |
| KEEP_INTERNAL_ONLY | `rust_core/src/storage/recovery_store.rs:8` | pub const RECOVERY_V3_MANIFEST_FILE: &str = "recovery-manifest-v3.json"; |
| KEEP_INTERNAL_ONLY | `rust_core/src/storage/recovery_store.rs:9` | pub const USB_RECOVERY_V3_DIR: &str = "keylesspass-recovery-v3"; |
| KEEP_INTERNAL_ONLY | `rust_core/src/storage/recovery_store.rs:10` | pub const USB_RECOVERY_V3_MANIFEST_FILE: &str = "recovery-manifest-v3.json"; |
| KEEP_INTERNAL_ONLY | `rust_core/src/storage/recovery_store.rs:19` | .join("recovery-v3") |
| DELETE | `docs/research/FACTOR_PRESERVING_PEER_RECOVERY.zh-CN.md:5` | 本方案替换的是 v3 中不便使用的纸质恢复份额，不替换 Root Key 的 |
| DELETE | `docs/research/PSPPD_BASELINE_RESULTS.zh-CN.md:40` | \| 当前 Encoder v2 \| 284.507 \| 355.241 \| 0 \| 不适用 \| |
| DELETE | `docs/research/PSPPD_BASELINE_RESULTS.zh-CN.md:49` | - 四个实现承担的工作不同，该表只能说明原型量级，不能直接作优劣结论。Encoder v2 还执行构造、约束检查和多次尝试；PSPPD 的 DFA 已预编译。 |
| DELETE | `docs/research/PSPPD_MATHEMATICAL_DESIGN.zh-CN.md:4` | > 本文档独立于当前 JISA 系统论文；现有 Encoder v2 与生产派生路径暂不修改。 |
| DELETE | `docs/research/PSPPD_MATHEMATICAL_DESIGN.zh-CN.md:193` | 第一版原型以规范 DFA 为输入，刻意不实现新的正则表达式解析器：正则到 DFA 已是成熟工具，不是研究贡献。原型提供精确计数、\`Rank\`、\`Unrank\`、FF1 cycle-walking 置换及组合派生；所有代码位于 \`rust_core::research\`，不接入 Encoder v2。 |
| DELETE | `docs/research/PSPPD_MATHEMATICAL_DESIGN.zh-CN.md:200` | 4. 比较当前 Encoder v2、Xeger 式局部随机游走、整体拒绝采样和 PSPPD； |
| DELETE | `docs/research/PSPPD_MATHEMATICAL_DESIGN.zh-CN.md:205` | 四种方法的比较必须区分目标：整体拒绝采样在正确选择母空间且随机源均匀时也能产生严格均匀分布，但可能效率很差；MFDPG/Xeger 式游走是否有偏取决于具体转移选择和回溯算法，不能把所有 DFA 游走一概判为有偏；当前 Encoder v2 只证明局部索引采样无取模偏差，尚无合法集合整体均匀证明；PSPPD 的新增价值主要是把双射、置换和 generation 结合成确定性无重复序列。 |

## Phase-2 acceptance checks

1. No match may remain under `paper/` except prose in this audit/report explaining that the old material was removed.
2. The submitted experiment manifest must not execute the old encoder or its fixed vectors.
3. EPSCD source, contexts, errors, and fixed vector must use `schemeVersion = 1` or `EXACT_POLICY_V1`, not “v3”.
4. Formal lifecycle state must contain no v2/v3 compatibility variables or invariants.
5. Product compatibility code may remain only outside the standalone artifact manifest and must be labelled internal.
