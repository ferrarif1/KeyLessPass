# EPSCD Research GO / NO-GO Decision

Date: 2026-08-10

## Verdict: PARTIAL GO

Proceed with the reconstructed EPSCD paper and artifact, subject to the gates below.

## Why the work proceeds

1. There is a real deployment problem: legacy enterprise targets accept only passwords and expose heterogeneous, sometimes weakly observable password-change operations.
2. Existing deterministic generators, verified random generators, policy languages, finite-language algorithms, and FPE establish the pieces but do not by themselves specify the combined credential-lineage and remote-commit contract.
3. The repository already contains working exact policy counting, bijective rank/unrank, an audited standard-primitive permutation backend, and a durable evidence-aware rotation core. The remaining work is integration and evidence, not invention of cryptography.

## Why the decision is not unconditional GO

Contribution 1 is a novel system-specific construction only; its mathematical ingredients are known. Contribution 2 is the stronger research contribution, but it is credible only after adapter capability tests, boundary fault injection, and a dedicated EPSCD TLA+ model demonstrate the stated safety properties.

## Mandatory gates

- [x] Canonical metadata includes an explicit credential lineage identifier.
- [x] No history-filter assumption remains in the baseline derivation contract.
- [x] Exact policy cardinality and `Rank`/`Unrank` inverses are tested exhaustively on small spaces and randomly on large spaces.
- [x] Permutation inversion, injection, walk density, and fail-closed limits are tested.
- [x] Rotation adapters declare what evidence they can establish; HTTP success alone never commits a generation.
- [x] Old and new credentials remain reconstructible in all uncertain states.
- [x] At least two distinct adapter semantics are exercised.
- [x] Fault injection covers every persistence/network boundary and the requested ambiguity cases.
- [x] `tla/epscd_rotation.tla` checks commit safety, uncertainty preservation, monotonicity, and generation uniqueness, with negative controls.
- [x] A clean command regenerates all manuscript numbers, tables, and model results.
- [x] The paper contains exactly two numbered technical contributions and uses only published works as experimental comparison objects.

If any safety gate fails, the paper is **NO-GO** until it is fixed. If only scale or optional baseline experiments remain incomplete, the paper stays **PARTIAL GO** and must narrow its claims.
