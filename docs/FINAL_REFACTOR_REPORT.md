# Research Refactor Report

Date: 2026-08-06

## Executive judgment

The original design was not threshold sharing: three pairwise keys encrypted three complete copies of the Root Key. Its rotation workflow was not two-phase commit, its CDR serialization was not normative, and the paper's novelty claim overlapped heavily with AutoPass and PALPAS.

The refactor corrects those claims and implements a substantially more rigorous v3 core. It does **not** complete every item in `改进.md`. The defensible research contribution is now a cross-layer lifecycle protocol that joins recovery generations, credential generations, remote-outcome evidence, replica ancestry, and optional freshness state. Shamir, HKDF, HMAC, JCS, and unbiased sampling are foundations, not claimed innovations.

## Recovery decision

Authenticated and version-bound Shamir 2-of-3 was selected after comparing pairwise complete-key wrappers, plain/authenticated Shamir, VSS, proactive sharing, threshold MPC, hardware-backed factors, and SLIP-0039. `vsss-rs` 5.4.0 supplies GF(256) split/combine operations; no custom field arithmetic is used. VSS does not solve this local trusted-dealer scenario's rollback or revocation problems, while proactive/MPC protocols require online parties and solve a different problem.

The v3 data model replaces complete-key wrappers with three 33-byte shares. Envelopes bind the vault, Root-Key generation, share set, index, threshold/count, factor role/ID/generation, suite, timestamp, and phrase encoding. Root-Key-derived HMAC authenticates the envelope after reconstruction and a KCV confirms the Root Key. Generation-specific files are validated before a manifest-last commit.

Legacy v2 remains a migration reader only. Migration dry-runs, validates all three legacy paths, preserves the Root Key, validates v3, commits v3, writes a phrase-redacted audit record, and can copy/verify/archive the old local and USB packages. This preserves existing service passwords.

## `改进.md` coverage

| Area | Status | Evidence | Honest boundary |
|---|---|---|---|
| Architecture diagnosis and recovery comparison | Complete | `ARCHITECTURE_DIAGNOSIS.md`, `RECOVERY_DESIGN_REVIEW.md`, ADR-001 | External audit not performed |
| Root-Key hierarchy/domain separation | Complete in core | `crypto/kdf.rs`, `KEY_HIERARCHY.md` | Some reserved subkeys have no consumer yet |
| Mature 2-of-3 recovery/all pairs | Complete in v3 core | `crypto/recovery.rs`, `recovery_store.rs` | Flutter enrollment still creates v2 |
| High-entropy recovery phrase | Complete format/core | KLRP v1, checksum, fixed vector | 108 words; no QR or human study |
| Share authenticity/version binding/KCV | Complete | Envelope MAC, validation, manifest/KCV tests | Bad pair detected; bad member not identified |
| Factor replacement/share-set refresh/stale rejection | Complete in core | `recovery_lifecycle.rs` | Full desktop UX not wired |
| Threshold compromise response | Partial | Empty-vault Root-Key rotation rejects old shares | Non-empty vault requires every remote password to rotate and is rejected |
| v2 pairwise migration | Core path complete | dry-run, all-path verification, commit, archive/audit test | Exhaustive interruption recovery is not complete |
| CDR formal schema/canonical serialization | Complete in core | CDR v3, RFC 8785 JCS, fixed vector | Dedicated external schema file/code generator absent |
| Policy-aware unbiased encoder | Complete for documented rules | HMAC stream, rejection sampling, shuffle, bounds/tests | Complex-policy entropy is an upper bound, not exact count |
| Staged rotation/unknown outcome/reconciliation | State core complete | persistent states, validated transitions, exhaustive exploration | No real target adapter or lockout-budget UI |
| Sync/conflict model | Core classification complete | parent hash, generations, operation/replica metadata | Existing UI sync is not a complete distributed synchronization engine |
| Whole-copy rollback | Interface/test complete | local-only boundary, CAS freshness interface | Production enterprise anchor not deployed |
| Threat model/security analysis | Complete at design level | `THREAT_MODEL.md`, paper claim matrix | No independent formal proof/audit |
| Fixed vectors/property-style tests | Complete for recovery/CDR/encoder/rotation | `rust_core/test-vectors`, Rust tests | No fuzzing campaign or proof assistant |
| Fault injection | Partial | manifest-last, illegal/replay, unknown-outcome, stale/mixed-share tests | Not every requested write boundary is injected |
| Reproducible experiment | Partial but real | release example and macOS quick/full JSON/CSV | CPU/peak memory, human entry, real adapters, baseline latency missing |
| 100--100,000 CDR scaling | Complete on current host | `experiments/results/full-2026-08-06` | One x86-64 macOS environment |
| Cross-platform execution | Configuration complete, execution pending | GitHub Actions macOS/Windows/Linux workflow | No Windows/Linux results are claimed |
| Paper and figures | Complete for current evidence | rewritten `.tex`, compiled 16-page PDF, render QA | Resubmission still needs stronger deployment evidence |
| README/design/security/changelog | Updated | root and `docs/` files | Some legacy product/user-guide pages still describe v2 and are historical |

## Implemented lifecycle rules

- One lost or suspected single share: reconstruct with two trusted shares, generate a new share set, increment factor generation, validate all pairs, and commit the new manifest. Old/new shares cannot mix.
- Two potentially leaked old shares: re-sharing is insufficient. Generate a new Root Key, increment `rootGeneration`, and rotate every remote password. The prototype performs this automatically only for an empty vault.
- Lost USB: recovery phrase plus computer creates a new USB factor and share set.
- Replaced computer: recovery phrase plus USB registers the new provider/device share and share set.
- Replaced recovery phrase: computer plus USB refreshes all shares and returns a new phrase.
- Ordinary USB: possession of a copyable file, never an unclonable-device claim.

## Test and experiment results

The final verification commands are:

```bash
cd rust_core
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-targets
cargo run --release --example research_evaluation -- ../experiments/results/quick-2026-08-06
KEYLESSPASS_FULL=1 cargo run --release --example research_evaluation -- ../experiments/results/full-2026-08-06
```

Final local verification passed rustfmt, Clippy with warnings denied, and all 72 Rust tests. The full macOS run measured 10,000 Shamir splits at 152.4 microseconds mean and the three recovery pairs at 248.5--258.0 microseconds mean. The 100,000-record workload wrote in 5.25 seconds, loaded in 1.98 seconds, queried in 2.08 milliseconds, and used 203.54 MiB. Password derivation plus policy encoding averaged 64.85 ms for the tested configuration. All values come from the checked-in raw JSON/CSV and are not cross-platform claims.

## Innovation assessment

AutoPass already includes deterministic site-specific generation, forced changes, and site rules. PALPAS already uses a random high-entropy seed, per-service salts, policy-aware generation, and metadata synchronization without centrally stored passwords. Consequently, these features cannot independently support novelty.

The strongest remaining contribution is the **generation- and failure-aware lifecycle protocol**:

1. recovery artifacts are selected by a committed `(vault, rootGeneration, shareSet, factorGeneration)` tuple;
2. credential activation is selected by remote evidence and a persistent operation state, including epistemic uncertainty;
3. replica ancestry is not reduced to `recordSeq`, and concurrent remote-password histories are not silently merged;
4. freshness is explicitly externalized instead of being falsely attributed to HMAC, hash chains, or secret sharing.

This is a credible systems-research direction, but the current artifact is not yet enough for a strong claim of production or deployment novelty. The highest-value next evidence would be real legacy-system adapters with injected crash/timeout outcomes, Windows/Linux runs, a human recovery study, and a deployed minimal freshness service. Adding more cryptographic primitives would increase complexity without strengthening the contribution.

## Paper change summary

The title no longer says `Storage-Free`. The paper explicitly defines service-password-storage-free semantics, acknowledges AutoPass/PALPAS/policy-language prior art, presents an evidence-based feature comparison, systematizes the threat model, replaces pairwise wrappers with authenticated Shamir, replaces two-phase-commit terminology, defines CDR and encoder algorithms, separates integrity from freshness, reports only current-run quantitative data, and carries all unimplemented functions into the limitations section.
