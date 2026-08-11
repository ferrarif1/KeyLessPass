# Latest manuscript gap audit

Audit date: 2026-08-09  
Target: `paper/manuscript.tex` and the 24-page PDF built on 2026-08-09  
Decision vocabulary: **KEEP**, **EXTEND**, **IMPLEMENT**, **PROVE**, **EVALUATE**

This audit is intentionally performed before changing the manuscript. It preserves
the completed EPSCD evidence and identifies only the gaps introduced by the new
exposure, compromise, recovery, and rollback research questions.

## Section-by-section decisions

| Current section | Decision | What remains valid | Required work before the next manuscript |
|---|---|---|---|
| Abstract | EXTEND last | The exact-language sequence, 120/121 corpus outcome, warm median, and bounded-backend limitation remain accurate. | Rewrite only after recovery prior art and new evidence are fixed. Add at most one sentence each for observed-credential exposure and deployment-threshold preservation; retain no more than 2--3 quantitative results. |
| 1. Introduction | EXTEND | The distinction between local modulo-bias freedom and uniformity over the final language remains central. The novelty boundary around counting, ranking, and rank-then-encipher remains correct. | Replace the single algorithm-only question with one unified question: exact recoverable credential sequencing without stored service-password values and without deployment threshold collapse. State that injectivity supplies non-repetition while PRP security supplies unpredictability. |
| Contributions | EXTEND | Exact indexing and evidence-bounded rotation remain defensible. | Use exactly two technical axes: exposure-aware policy-space sequencing and factor-preserving heterogeneous Root-Key recovery. Treat the integrated lifecycle/evaluation as evidence, not a third cryptographic novelty. |
| 2.1 Deterministic credential generation | KEEP | Published systems and the accurately labelled MFDPG preprint remain relevant. Existing MFDPG artifact evidence must not be rerun unless implementation changes invalidate it. | Add only the exposure comparison that is directly supported by published work. Do not compare experimentally with an unpublished local encoder. |
| 2.2 Policy representation and measurement | KEEP | PCP corpus provenance and bounded IR scope are current. | No structural change. Add credential-space adequacy terminology where the corpus statistics are interpreted. |
| 2.3 Uniform generation and indexing | KEEP | The manuscript already concedes the classical foundations. | Add the conditional residual-uniformity lemma as a standard random-permutation symmetry specialized to generation-indexed credentials, not as new probability theory. |
| 2.4 Finite-domain encryption | EXTEND | Rank-then-encipher, FF1, cycle walking, and small-domain cautions are accurately bounded. | Separate backend-domain eligibility from credential-security adequacy. Add known-pair exposure and concrete small-domain attack boundaries. |
| 2.5 Rotation lifecycle and Root-Key recovery | REPLACE/EXTEND | Shamir and MFKDF citations remain necessary. | Replace the statement that recovery is only supporting 2-of-3 functionality with a prior-art-bounded discussion of deployment capability closure. Include SVR3, SafetyPin, secure account recovery, SPHINX, and threshold PPSS where relevant. |
| 3. Model and security assumptions | EXTEND | Public derivation metadata and trusted-while-deriving endpoint assumptions remain clear. | Enumerate all public metadata explicitly. Add one-password, q-password, Kcred, Kroot, server-verifier, endpoint, removable-medium, network-node, approver, and rollback adversaries. Define compromise domains D/U/N/A and `Closure(X)`. |
| 4. Policy model and exact bijection | KEEP | IR, recurrence, exact count, and rank/unrank lemma remain valid. | No algorithm rewrite. Add `B_P = log2 N_P` as credential-space size, not fixed-output entropy. |
| 5. Generation-indexed derivation | EXTEND/PROVE | The current formulas and Theorems 1--4 remain usable. Code inspection confirms `policyEpoch` is in the tweak but not the HKDF key context. | Add observed-credential residual uniformity for q known pairs, a real-PRP game/reduction sketch, and an explicit injectivity/unpredictability separation. State the current Kcred leakage consequence. Introduce credential rekey semantics only after implementation is selected. |
| 6.1 Versioning | EXTEND/IMPLEMENT | Orthogonal policy, credential, root, and share-set counters are the correct starting point. | Add an independently advancing credential-key lineage/rekey state or a fresh-salt lineage rule. Define authenticated history exclusion across the new lineage. |
| 6.2 History exclusion | KEEP/EXTEND | Proposition 1 and Theorem 5 remain valid. | Apply exclusion to credential-key lineage change, not only policy change. State that stale metadata can reuse a generation despite permutation injectivity. |
| 6.3 Remote rotation | KEEP | Candidate durability, new/old probing, ambiguous state, lockout budget, and evidence-bounded commit remain valid. | Bind successful commits to freshness publication. Preserve the existing TLA+ rotation evidence rather than rebuilding it. |
| 6.4 Supporting threshold recovery | REPLACE/IMPLEMENT | A random 256-bit Root Key and Shamir 2-of-3 remain standard components. | Promote recovery into an independent section only if prior-art audit supports the scoped abstraction. Implement D/U/N top-level shares; 3-of-5 network fragments; independent A-domain signed approval; session-bound encrypted fragment release; stale-ticket/share-set rejection; ordinary re-share versus Root-Key replacement. Do not use view/data-key separation or threshold OPRF. |
| 7. Implementation | EXTEND | Compiler/PRP separation and fixed vector remain current. | Describe credential rekey, a multidimensional freshness checkpoint, and the new network recovery prototype. Clearly mark the old view/data-key + OPRF peer-recovery prototype as superseded research code and remove it from the active artifact path. |
| 8.1 Corpus | KEEP/EVALUATE | The attempted 121 translations and 120 completions are final unless compiler semantics change. | Recompute `log2(N_P)` and configurable security-floor eligibility from existing raw data; do not repeat compilation. |
| 8.2 MFDPG | KEEP | Pinned commit, source analysis, toy-distribution result, and end-to-end repeat observation remain valid and accurately labelled as preprint-artifact evidence. | Do not rerun or promote it to a peer-reviewed experimental baseline. |
| 8.3 Dichopile | KEEP | Published-baseline status and cold/warm measurements remain valid. | No rerun unless shared policy/compiler code changes its inputs. |
| 8.4 Performance | KEEP/EXTEND | Existing cold/warm data remain valid. | Measure only new recovery crypto/network/approval components, with approval latency reported separately. |
| 8.5 Cycle walking | KEEP | Corpus-wide walk results and ideal/concrete boundary remain valid. | Add credential-space adequacy interpretation; no rerun. |
| 8.6 Lifecycle model checking | EXTEND | Existing 1,006,128-distinct-state rotation model and invariants remain evidence. | Add separate recovery-access and integrated freshness models. Abstract the PRP as an atomic injective operation. |
| 9. Security discussion | EXTEND | Current unconditional/ideal/computational separation remains correct. | Add exposure hierarchy, server-verifier enumeration, Kcred/Kroot compromise remediation, factor closure, rollback, and the statement that ordinary re-sharing does not repair an exposed Root Key. |
| 10. Limitations | EXTEND | Corpus, platform, adapter, backend, and bounded-model limits remain accurate. | Add approval-domain independence assumptions, endpoint compromise timing, cloneable U media, non-Byzantine prototype limitations, and the fact that one administrative plane can collapse nominally separate domains. |
| 11. Conclusion | EXTEND last | Current EPSCD summary remains usable. | Close on the unified rotation-and-recovery question only after prior art and evidence support it. |
| Formal appendix | EXTEND/PROVE | Bijection, exclusion, and cycle-walk proofs remain valid. | Add conditional residual-uniformity proof and precise real-PRP advantage statement. Do not claim the symmetry argument as a new theorem of permutation theory. |

## Blocking facts established by repository inspection

1. `derive_credential_key` binds scheme, vault, service, account, and
   `rootGeneration`; `credentialSalt` is the HKDF salt. `policyEpoch` is only
   in `permutation_tweak`. Therefore a leaked `Kcred` remains usable after a
   policy-epoch increment.
2. The existing peer-recovery prototype is a 1,210-line view/data-key,
   fixed-object, erasure-code, threshold-OPRF construction. It conflicts with
   the new design decision and cannot serve as the implementation of the new
   recovery claim.
3. `FreshnessAnchor` currently tracks only `rootGeneration`, one `cdrEpoch`,
   and a digest. It does not independently reject rollback of
   `credentialGeneration`, `policyEpoch`, and `shareSetGeneration`.
4. The current theorem prevents collisions only for distinct generation inputs
   in one fixed context. Restoring a valid old generation reuses a prior input;
   this is rollback-induced reuse, not a permutation collision.
5. The current `RecoveryManifest` uses `share_set_id` but has no explicit
   monotonic `shareSetGeneration`; the lifecycle prose is ahead of that data
   structure.

## Work that must not be repeated

- MFDPG venue/status correction, artifact pinning, unit-test run, and toy probes.
- The 121-policy compilation attempt and its fixed resource budget.
- Dichopile exact-arithmetic transcription and cold/warm measurement.
- Corpus-wide FF1 cycle-walk measurements.
- The current rotation lifecycle model check.
- The restrained abstract rewrite that removed checklist-style reporting.

