# EPSCD Research-Mainline Repository Audit

Date: 2026-08-10

## Decision boundary

The repository is being reconstructed around one research question:

> How can an enterprise rotate credentials for legacy password-only systems while generating every credential exactly inside the accepted policy language and never treating an uncertain remote update as a committed local generation?

This audit is a boundary document, not a novelty claim. It identifies reusable implementation, missing evidence, and content that must remain outside the EPSCD proof boundary.

## Protected independent work

The following trees contain the independent CETS research line and are outside the EPSCD reconstruction:

- `paper_dccea/`
- `research_upgrade/ccas_dprf/`
- `research_upgrade/cets_reference_protocol/`

They are neither EPSCD baselines nor EPSCD formal evidence. The EPSCD paper may cite a published version in the future, but the present unpublished manuscript, models, and measurements are not comparison objects and are not imported into the new paper.

## Reusable EPSCD implementation

| Capability | Existing location | Reuse decision |
|---|---|---|
| Bounded policy IR and compiler | `rust_core/src/policy/mod.rs` | Reuse; document the accepted constraint fragment and state cap. |
| Big-integer exact suffix counts | `rust_core/src/policy/mod.rs` | Reuse; expose compile and online cost metrics. |
| Exact `Rank`/`Unrank` | `rust_core/src/policy/mod.rs` | Reuse; extend exhaustive and randomized inverse tests. |
| Exact-domain keyed sequence | `rust_core/src/epscd/mod.rs` | Reuse after making lineage semantics explicit and removing history-filter assumptions. |
| FF1 bounded cycle walking | `rust_core/src/permutation/mod.rs` | Retain as the audited backend; scope timing and fail-closed claims. |
| Evidence refinement for old/new credentials | `rust_core/src/domain/rotation.rs` | Reuse the four-state remote evidence lattice. |
| Durable local commit guard | `rust_core/src/service/rotation.rs` | Reuse after adding an explicit adapter capability/evidence contract. |
| 270-policy source corpus | `experiments/policies/soups2022_pcp_corpus.json` | Reuse with provenance and translation-status reporting. |
| Existing policy measurements | `experiments/real_policy_corpus/` | Treat as provisional; regenerate through the new one-command artifact. |

## Gaps that block the new paper

1. The scheme context derives a lineage identifier indirectly but does not persist an explicit `lineage_id` in the public metadata contract.
2. Rekey documentation still requires authenticated history exclusion, which is not part of the new baseline.
3. The rotation state machine does not publish a compact adapter capability model stating which remote facts can actually be established.
4. Existing TLA+ models are not a dedicated EPSCD rotation model and contain old history/recovery concerns.
5. No single artifact command regenerates the policy, sequence, permutation-density, and rotation-fault results consumed by the paper.
6. Existing `paper/manuscript.tex` mixes more than two contributions and contains material from superseded research directions.
7. The prior-art and permutation-backend audits are fragmented across older notes rather than tied to the claims of the reconstructed paper.

## Reconstruction rule

The new artifact is additive and reuses tested library code. It does not delete old manuscripts or mutate protected CETS directories. The authoritative new manuscript is `paper/epscd.tex`; the authoritative new formal model is `tla/epscd_rotation.tla`.
