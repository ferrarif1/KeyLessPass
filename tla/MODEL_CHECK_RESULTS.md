# EPSCD Rotation Model-Checking Results

Date: 2026-08-10

Model: `epscd_rotation.tla`

Tool: TLC 2.19 (tla2tools 1.7.4)

## Main configuration

`epscd_rotation.cfg` sets `MaxGeneration = 3` and disables both injected faults. TLC performed a complete breadth-first exploration:

- generated states: 2,877;
- distinct states: 1,069;
- states remaining: 0;
- maximum depth: 16;
- result: no invariant violation.

Checked invariants:

- `CommitRequiresEvidence`;
- `CommitMatchesRemoteAcceptance`;
- `UncertaintyKeepsBoth`;
- `PreparedBeforeSubmission`;
- `UnknownDoesNotAdvance`;
- `SequentialUniqueGenerations`;
- the type invariant.

## Negative controls

| Configuration | Injected defect | Expected result | Observed result |
|---|---|---|---|
| `epscd_rotation_negative_http.cfg` | Permit a generic successful response to commit without adapter evidence. | `CommitRequiresEvidence` violation. | Violation found after 49 distinct states. |
| `epscd_rotation_negative_drop.cfg` | Delete candidate reconstructability on entry to `UnknownOutcome`. | `UncertaintyKeepsBoth` violation. | Violation found after 15 distinct states. |

These are sensitivity controls for the abstract transition relation. They do not establish implementation correctness, cryptographic security, liveness, or behavior outside the modeled bounds.
