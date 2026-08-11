# Candidate innovation specification: dual-collapse credential exposure

Date: 2026-08-09
Working name: **Dual-Collapse Credential Exposure Analysis (DCCEA)**
Claim status: candidate method; no priority or new-primitive claim

## Problem solved

A threshold analysis answers whether a compromise set reaches a root secret.
That answer is incomplete for a distributed derivation interface: the root may
never appear while a compromised endpoint uses one broad approval to request
every credential output.  Conversely, exact per-context authorization does not
repair a deployment that already collapses the nominal factor threshold.

DCCEA reports both failures instead of treating master secrecy as a proxy for
credential compartmentalization.

## Method

For compromised domains `X` and legitimately approved contexts `T`, compute the
least protocol closure and return:

```text
B_R(X,T) = (M_R(X,T), E_R(X,T))
```

where `M_R` records master capability and `E_R` is the set of credential
contexts whose outputs become derivable.  The method then checks:

```text
factor collapse:              nominal deployment threshold is reduced
authorization amplification: E_R(X,T) is not a subset of T
```

For a ticket budget `q`, it reports the exact unauthorized-exposure profile:

```text
A_R(X,q) = max_{|T| <= q} |E_R(X,T) \ T|.
```

It also reports the minimum compromise threshold for each exposure target:

```text
tau_E(k,q) = min |X| such that some |T| <= q gives
             |E_R(X,T) \ T| >= k.
```

This **credential exposure threshold spectrum** is the central quantitative
output.  A Root threshold is one point; the spectrum reveals lower-cost
callable-interface paths that expose fewer than the complete vault.

When tickets bind only a field projection whose equivalence-class sizes are
`s_1 >= ... >= s_p`, the profile has closed form:

```text
A_R(X,q) = sum_{i=1}^{min(q,p)} (s_i - 1).
```

The repair is to bind authorization to the collision-resistant digest of the
complete canonical derivation context plus operation identifier, expiry, and
freshness generation.  This repair uses established token and DPRF mechanisms;
the claimed contribution is the diagnostic and exact exposure calculation.

## Formal results available

1. **Root dominance:** in a Root-derived credential profile, Root capability
   exposes every modeled context, so factor collapse implies maximal output
   exposure.
2. **Root-only incompleteness:** below the Root threshold, identical
   master-access predicates can have different context-exposure maps.
3. **Projection identity:** a projection-bound ticket exposes exactly the
   target's projection equivalence class under the stated evaluator model.
4. **Injectivity criterion:** one-context non-amplification holds exactly when
   the checked projection is injective over the modeled context set.
5. **Exact approval-budget profile:** the sorted class-size formula above is
   optimal.
6. **Exposure-spectrum monotonicity and Root bound:** `tau_E` is monotone in
   the exposure target and never exceeds the Root threshold under Root
   dominance.
7. **Conditional composition:** fresh-context security reduces to the reviewed
   DPRF security, ticket unforgeability, context-digest collision resistance,
   and the explicit non-amplification premise.

## Claims explicitly rejected

- a new Shamir scheme, DPRF, OPRF, constrained PRF, MAC, or capability system;
- the first context-bound or least-privilege token;
- the first attack graph, fixed-point closure, permission-expansion analysis,
  effective access structure, or candidate-key algorithm;
- a proof that the working name is globally unique; or
- production security from the abstract model.

## Evidence currently implemented

- six deployment-collapse fixtures;
- seven context-binding fixtures with a 32-context exact exposure curve;
- all three reachable states under Root dominance: safe, scope amplification
  without Root access, and Root collapse with full-context exposure;
- exact projection-class and ticket-budget profiles;
- cardinality- and cost-based credential exposure threshold spectra;
- 18 passing unit tests;
- a TLA+ positive model with 9 distinct states and an expected wildcard-scope
  counterexample with 11 distinct states; and
- local analyzer scaling through 100,000 contexts.

## Publication gate

The method is strong enough to continue as a research contribution, but not yet
strong enough to rewrite the frozen submission.  Before manuscript inclusion:

1. encode at least 12 published, sufficiently specified protocol/deployment
   profiles at message/rule level;
2. obtain an independent encoding review;
3. demonstrate at least three failures that pass a Root-only check but fail the
   context-exposure check;
4. compare only against published work and generic baselines, never the
   project's unpublished earlier encoders or papers; and
5. retain the current paper unchanged if this empirical gate fails.
