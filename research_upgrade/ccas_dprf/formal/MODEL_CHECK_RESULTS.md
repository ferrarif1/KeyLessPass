# CCAS routine-derivation model-check result

Model: `CCASRoutine.tla`
Configuration: `CCASRoutine.cfg`
TLC: 2.19 (tla2tools 1.7.4)
Run date: 2026-08-09

## Final result

TLC completed exhaustive breadth-first exploration of the configured finite
model without finding an invariant violation:

```text
21,085 states generated
3,528 distinct states
0 states left on queue
complete-state-graph depth: 23
```

Checked invariants:

- `TypeOK`
- `NoRoutineRootMaterialization`
- `NoCrossCredentialDerivationFromSingleAuthorizedEvaluation`
- `NoSingleDomainMasterCapability`
- `EffectiveThresholdNotBelowConfiguredThreshold`
- `UnauthorizedContextCannotCompleteEvaluation`
- `RawShareNeverLeavesTokenDomain`
- `CurrentCredentialExposureDoesNotAuthorizeOtherContexts`
- `RecoveryIsExplicitlySeparate`

## Scope

The model checks an abstract two-context protocol and the independent-approval
deployment profile. Partial evaluations are symbolic events. The result does
not verify a DPRF construction, implementation memory erasure, hardware-token
security, network authentication, liveness, or usability.

## Context-authorization amplification model

`ContextAmplification.tla` checks the stronger property
`derived subseteq approved` while keeping both the raw token share and Root Key
abstractly unavailable to the endpoint.

- With `TokenScope = "Exact"`, TLC exhaustively checked 28 generated states,
  9 distinct states, and a complete-state-graph depth of 6 without an invariant
  violation.
- With `TokenScope = "Wildcard"`, TLC generated the expected counterexample
  after 25 generated and 11 distinct states: approval covers `credential-A`,
  the endpoint requests `credential-B`, the token evaluates `credential-B`, and
  the combine step adds it to `derived`, violating
  `NoAuthorizationAmplification` at depth 5.

This establishes a protocol-model distinction: Root-Key non-materialization is
compatible with whole-context authorization amplification. It does not prove
the security of a DPRF implementation.

## Unified Root-and-context model

`DualCollapseUnified.tla` records UDC reachability and exposed credential
contexts in the same state. Exhaustive finite checks produced these results:

- `Mode = "Exact"`: 28 generated states, 9 distinct states, graph depth 6;
  `RootDominance`, exact-scope non-amplification, and the type invariant hold.
- `Mode = "Wildcard"`: the expected non-amplification counterexample occurs
  after 25 generated and 11 distinct states at depth 5, with the Root absent.
- `Mode = "Root"`: 60 generated states, 15 distinct states, graph depth 6;
  `RootDominance` and the type invariant hold throughout the complete graph.
- Rechecking `Mode = "Root"` with `NoAuthorizationAmplification` produces the
  expected depth-3 counterexample immediately after UDC acquisition exposes
  both contexts.

Together these runs cover the three reachable abstract states: exact scope
below the UDC threshold, scope amplification without UDC access, and UDC
access with full-context exposure. The model is deliberately finite and does
not establish cryptographic security, network liveness, or implementation
correctness.

## Temporal ticket-lifecycle model

`TemporalTickets.tla` models two contexts, time values `0..2`, two freshness
generations, ticket issue and expiry, revocation, single-use replay state,
requests, evaluations, combination, and unrestricted derivation capability
(UDC) acquisition. `AcquireUDC` abstracts deployment-specific witnesses: in
the POPRF reference deployment it denotes either endpoint-plus-approval
compromise or endpoint-plus-both-evaluators compromise. It does not treat the
two evaluator domains alone as a UDC witness.

With exact scope and every lifecycle check enabled, TLC completed the finite
state graph without an invariant violation:

```text
3,997 states generated
888 distinct states
0 states left on queue
complete-state-graph depth: 10
```

The checked properties were type correctness, UDC dominance, below-UDC scope
confinement, and rejection of expired, revoked, stale-generation, and replayed
tickets.

Five single-fault configurations each produced the intended counterexample:

| Configuration | Violated property |
| --- | --- |
| Projected scope | Below-UDC scope confinement |
| Expiry check disabled | No expired acceptance |
| Revocation check disabled | No revoked acceptance |
| Generation check disabled | No stale-generation acceptance |
| Single-use check disabled | No replay acceptance |

The negative controls are counterexample checks, not scalability benchmarks.
With parallel TLC, the generated-state count at discovery can vary with search
ordering, so it is intentionally not treated as a manuscript result.

The model checks the final abstract transition relation. It does not verify the
POPRF implementation, durable replay storage, an unbounded clock, or network
liveness.
