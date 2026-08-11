# ASTER research artifact

This directory contains the implementation and evidence for **ASTER: Authorization-Scoped Threshold Exact-Domain Credential Derivation with Failure-Safe Root-Epoch Healing**.

The versioned artifact excludes `paper/`, `output/`, and `submission/` under
this directory. Those paths contain manuscript or journal-delivery material
and are ignored by git. Protocol specifications, executable code, raw and
generated results, adapters, formal models, and reproduction scripts remain in
the repository.

The artifact separates four evidence layers:

1. `rust_core/src/research/aster*.rs` implements canonical request encoding, Ed25519 exact-scope capabilities, durable SQLite use accounting, semantic Root-Epoch replacement, and the descriptor-only migration journal.
2. `experiments/`, `adapters/`, and `semantic/` exercise exact policy compilation, capability confinement, endpoint exposure, healing, scalability, and failure injection.
3. `mpc/` executes a fixed exact-domain circuit in MP-SPDZ's malicious honest-majority Shamir/BMR backend and checks it against an independent OpenSSL-based reference.
4. `tla/` checks the lifecycle and authorization guards with one positive configuration and eight deliberately broken negative controls.

## Main measured results

- 121 exact policy translations compiled; 97 completed 9.7 million derivations without policy, duplicate, replay, or Rank/Unrank failures; 24 oversized domains failed closed.
- Exact capabilities produced zero spill in the 32-context experiment; projected and wildcard controls produced concrete spill.
- Independent Root-Epoch replacement reduced old-root exposure from 100 credentials to zero as migrations committed; share refresh preserved outputs.
- The two-adapter fault matrix produced 96 traces with zero commit or uncertainty-preservation violations; a separate pinned single-server OpenLDAP smoke test passed modify/bind interoperability checks.
- TLC checked 777 distinct positive states and all eight broken configurations produced counterexamples.
- Three- and five-party MP-SPDZ runs agreed with independent fixed vectors; their high loopback cost is reported as feasibility evidence only.

## Reproduction

```sh
./research/aster/scripts/reproduce_all.sh --quick
./research/aster/scripts/reproduce_all.sh --full
```

`--quick` runs deterministic tests and reduced checks. `--full` regenerates the complete policy corpus, adapter traces, OpenLDAP smoke result, TLA+ outputs, summaries, tables, figures, and security scan. The OpenLDAP step requires Docker and uses the pinned image recorded by the adapter script. The MP-SPDZ image can be rebuilt with `research/aster/mpc/build_mpspdz_image.sh`; RQ6 is intentionally not rerun by the routine quick target because each fixed-vector execution is expensive.

See `LIMITATIONS.md` for the exact security and evaluation boundaries and `results/generated/RESULT_PROVENANCE.json` for manuscript-to-result field mappings.
