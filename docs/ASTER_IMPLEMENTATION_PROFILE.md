# ASTER implementation profile

This document records which ASTER mechanisms are used by the KeyLessPass desktop product and which mechanisms remain research evidence. It is the implementation boundary for code review, release notes, and manuscript claims.

## Desktop product path

New credentials use exact-policy-space scheme v3:

1. The Rust core compiles the supported finite password policy and computes its exact cardinality.
2. The credential context binds the vault, service, account, lineage, credential salt, Root generation, policy identity, policy version, policy hash, and policy epoch.
3. HKDF-SHA256 derives a credential-specific permutation key.
4. FF1-AES-256 cycle walking maps the credential generation into the exact policy domain.
5. `Unrank` returns the unique policy-conforming password at that rank.

The implementation rejects unsupported policy semantics, domains smaller than the configured FF1 minimum, domains above the 512-bit ceiling, exhausted generations, and cycle walks that exceed the safety bound. It does not silently approximate a policy or fall back to the legacy encoder.

Password rotation is evidence-bound. The previous record remains active while an update outcome is unknown. New-password and old-password authentication probes are recorded separately; commit is permitted only when the configured remote contract has conclusive evidence. Candidate selection excludes the configured window of locally derivable historical generations.

The local compatibility profile still reconstructs the Root Key transiently from the selected Shamir factors. Legacy v2 records remain readable only so existing users can reproduce and migrate their credentials. New records use v3.

## ASTER research profile

Building `rust_core` with `--features research` additionally enables:

- canonical ASTER request encoding;
- Ed25519 exact-scope authorization capabilities;
- durable use-budget and revocation accounting;
- semantic Root-Epoch replacement and descriptor-only migration;
- endpoint secret-inventory instrumentation;
- research experiments and negative controls.

The process-local semantic evaluator holds Root-Epoch keys. It verifies authorization and lifecycle invariants but is not an MPC or threshold deployment.

`research/aster/mpc/` contains a separate MP-SPDZ fixed-domain circuit and independent reference used for feasibility measurements. It does not provide the desktop application's runtime backend, arbitrary-subset availability, DKG, share refresh, or production LAN/WAN performance.

## Claim boundary

It is accurate to state that the repository implements and tests exact-domain credential derivation, exact-scope capability validation, durable authorization accounting, evidence-bound rotation, Root-Epoch lifecycle semantics, formal models, and a separate threshold-computation feasibility circuit.

It is not accurate to state that the released desktop client performs ordinary credential derivation through a production threshold ASTER service or that the complete Root key can never appear in client memory. That stronger deployment requires an independently operated threshold backend and endpoint integration that are outside the current artifact.

## Verification

```bash
cd rust_core
cargo test
cargo check --all-targets --all-features

cd ..
./research/aster/scripts/reproduce_all.sh --quick
```

See `research/aster/README.md`, `research/aster/LIMITATIONS.md`, and `research/aster/results/generated/RESULT_PROVENANCE.json` for evidence scope and provenance.
