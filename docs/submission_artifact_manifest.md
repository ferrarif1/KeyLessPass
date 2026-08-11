# Submission Artifact Manifest

Date: 2026-08-09

This manifest defines the artifact for *Exact Policy-Space Credential
Derivation with Factor-Preserving Root-Key Recovery*.

## Included

- `rust_core/src/epscd/`: credential context, key derivation, rekeying,
  generation-to-password composition, and tests;
- `rust_core/src/policy/`: bounded policy IR, compiler, exact counting,
  rank/unrank, and property tests;
- `rust_core/src/permutation/`: finite-domain permutation interface and bounded
  FF1 cycle-walking research backend;
- `rust_core/src/research/peer_recovery.rs`: active factor-preserving recovery
  profile with Shamir 3-of-5 network fragments, signed authorization,
  session-bound release, and adversarial tests;
- `rust_core/src/service/freshness.rs`: multidimensional CAS freshness state;
- `rust_core/src/published_baselines/dichopile.rs`: exact-arithmetic
  reproduction of Oudinet, Denise, and Gaudel, TCS 2013;
- `rust_core/test-vectors/epscd-scheme-v1.json`;
- main experiment drivers under `rust_core/examples/`;
- published-corpus inputs/provenance and outputs under
  `experiments/real_policy_corpus/`;
- `experiments/known_credential_exposure/known_credential_exposure.json`;
- `experiments/password_space_statistics/space_adequacy.json`;
- `experiments/results/factor-recovery-quick-2026-08-09/`;
- `supplementary/formal_proofs.tex`;
- `formal/lifecycle.*`, `formal/recovery_access.*`,
  `formal/integrated_model.*`, and their result records;
- manuscript source, figures, tables, bibliography, and final PDF.

## Explicitly excluded

- prior manuscripts and their revision-history prose;
- unpublished internal encoder measurements or collision results;
- MFDPG and other preprint artifact probes;
- superseded View-Key/Data-Key, OPRF, opaque-object, or erasure-coded recovery
  experiments;
- product migration/UI compatibility code not called by the research artifact;
- any result that would imply unmeasured network, approval, usability,
  cross-platform, or target-adapter performance.

No excluded component is cited by a table or needed to reproduce a claim in
the submission manuscript.

## Primary checks

See `docs/REPRODUCIBILITY.md` for exact commands. The minimum release gate is:

```text
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-targets --all-features
latexmk -pdf -interaction=nonstopmode -halt-on-error manuscript.tex
```
