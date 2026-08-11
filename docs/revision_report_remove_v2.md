# EPSCD Standalone-Paper Revision Report

Date: 2026-08-09

## 1. Removed internal encoder material

The manuscript, bibliography, figures, tables, fixed EPSCD vector, main
experiments, and lifecycle model contain no internal encoder name, unpublished
encoder formula, or unpublished encoder result. The full repository inventory
and disposition are recorded in `remove_legacy_v2_audit.md`. Product-only
compatibility paths are marked `KEEP_INTERNAL_ONLY` and excluded by
`submission_artifact_manifest.md`.

## 2. Removed scheme-upgrade narrative

The paper now begins with the legacy password-only service problem and defines
EPSCD as public `schemeVersion = 1`. It contains no algorithm-upgrade story,
pair-version dispatch, backward-compatibility claim, or transition chapter.
`policyVersion`, `policyEpoch`, `rootGeneration`, `credentialGeneration`, and
`shareSetGeneration` now have independent protocol meanings.

## 3. Removed experiments

All unpublished internal-encoder distribution, collision, latency, migration,
and byte-compatibility results were removed from the paper and main evaluation.
Mechanistic controls remain only under `supplementary/mathematical_controls/`
and are labeled as controls, not competing systems.

## 4. Published baselines added

The executable main baseline is Oudinet, Denise, and Gaudel's published TCS
2013 Dichopile algorithm, reproduced from the full paper with exact BigUint
arithmetic. Grilo et al.'s iFM 2022 verified password generator is used only for
property comparison. PwdHash, Password Multiplier, and PALPAS are published
deterministic-credential comparison rows. The SOUPS 2022 PCP work supplies the
public policy corpus. No unpublished work appears as a manuscript comparison
object.

## 5. MFDPG reproduction status

MFDPG could not be verified as an NDSS publication: the public manuscript
retains a placeholder DOI and the official NDSS 2024 accepted-paper list does
not contain it. Its official repository was pinned at commit
`6c26096dd22ff2b18aa5d8e4c3d5b0caf7b45bb7`; the twelve upstream unit tests
passed under Node 22.23.1/npm 10.9.8, and a dependency-accurate output-selection
probe was run. This is not an end-to-end reproduction because Argon2 preimages
were replaced by labeled test seeds. It is therefore excluded from the paper,
the published-baseline table, and the submission artifact boundary.

## 6. Real-policy corpus

The corpus is the public 270-record workbook released with Gautam, Lalani, and
Ruoti, SOUPS 2022. The downloaded workbook digest is
`2d424c30fa1e4d2e5f0b82a5c67b4214fc2a96574d8b70052994d6f92712a77a`.
Of 270 rows, 121 translate exactly; 55 satisfy the evaluation resource gate and
all compile; 66 are resource-skipped; 149 are rejected for unsupported
semantics. Every row retains its source and disposition.

## 7. New tables and figures

- Table 1: property comparison containing only published generation work and
  EPSCD;
- Table 2: full real-corpus accounting and state/time measurements;
- Table 3: representative synthetic stress policies;
- Table 4: Oudinet exact reproduction and EPSCD ideal-permutation distribution;
- Table 5: rotation/non-repetition scope;
- Table 6: rank, unrank, full derivation, and Dichopile timings;
- Figure 1: standalone EPSCD policy-to-password pipeline;
- Figure 2: evidence-bounded remote rotation and reconciliation outcomes.

## 8. Related-work expansion

The manuscript now cites 21 formally published papers or official standards:
PwdHash, Password Multiplier, PALPAS, password persistence, the iFM 2022
verified generator, PRML, SOUPS 2022 PCP, CCS 2023 policy measurement,
Hickey--Cohen, Goldwurm, Denise--Zimmermann, Bernardi--Giménez,
Goldberg--Sipser ranking, Oudinet et al., Black--Rogaway, Bellare et al., NIST
FF1, Durak--Vaudenay, Shamir, MFKDF, and the published MFKDF cryptanalysis.
The novelty audit covers additional literature and explicitly denies novelty
for DFA construction, exact counting recurrences, ranking, uniform
regular-language generation, FPE, cycle walking, and Shamir sharing.

## 9. TLA+ changes

The standalone model contains `Active`, `Selecting`, `Pending`, `Submitted`,
`UnknownOutcome`, `NewOnly`, `OldOnly`, `Both`, `Neither`, and `Recovering`.
It removed all compatibility/version-pair state. Nine invariants cover evidence
gating, generation agreement, stable salt, epoch semantics, history exclusion,
unknown outcomes, and persistence before submission. TLC 2.19 generated
3,129,888 states, found 1,006,128 distinct states, reached depth 56, and reported
no invariant violation under `MaxEpoch=2` and `MaxGeneration=3`.

## 10. Dependency on an older encoder

None exists in the research call path. `rust_core::epscd::derive_password`
directly composes the standard HKDF/JCS dependencies, `DomainPermutation`, and
policy unranking; it no longer imports the product crypto module. The public
fixed vector and main experiment call that API. Historical
product modules remain in the development tree only for application
compatibility and are not included in the submission manifest.

## 11. Unresolved issues

The concrete backend is bounded FF1 cycle walking rather than a proof-matched
total finite-domain implementation; the compiler still has product-state
explosion; the public corpus is historical Web data; only macOS x86-64 has been
measured; production target adapters, real-service fault injection, recovery
usability, and independent cryptographic review remain absent. The TLA+ result
is bounded and abstracts cryptography, storage implementation, timing, and
lockout behavior.

## 12. Claims deliberately downgraded

The paper does not claim a first uniform password generator, a new automaton or
ranking algorithm, a new FPE, unconditional uniformity for a real PRP, a total
prototype backend, universal policy support, a new recovery construction, or
production readiness. It claims a scoped integration: exact accepted-language
indexing plus a keyed generation-indexed permutation, explicit credential and
policy epochs, authenticated history exclusion, and evidence-bounded rotation.
The non-repetition theorem is structural but elementary; acceptance still
depends on the lifecycle integration and empirical evidence being judged
substantive.
