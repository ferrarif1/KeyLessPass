# Research Go/No-Go v2: dual-collapse credential exposure

Decision date: 2026-08-09  
Decision scope: whether a defensible innovation method has been identified  
Decision: **CONDITIONAL GO FOR THE METHOD; MANUSCRIPT REMAINS FROZEN**

## Decision

The research now has a concrete method worth pursuing:

> jointly compute the effective deployment-domain access to the master and the
> authorization-indexed set of deterministically derivable credential
> contexts, then quantify unauthorized spill as an exact function of the number
> of legitimate approvals.

This solves a real blind spot.  A design can preserve its nominal Root-Key
threshold and never materialize the master yet still expose every service
password through an over-broad legitimate evaluation interface.  Root-only
analysis cannot decide that property. The corrected joint model also proves a
one-way dominance relation: once Root capability is reachable, every modeled
credential context is exposed, so only three security states are reachable.

## Novelty boundary

| Candidate element | Verdict |
|---|---|
| Shamir, DPRF/OPRF, constrained PRF | No-go as novelty; established primitives |
| Context-bound tickets and least privilege | No-go as novelty; established authorization work |
| Fixed-point capability closure / attack witnesses | No-go as novelty; MulVAL and attack graphs |
| Permission-expansion analysis in general | No-go as novelty; PolyScope is close prior art |
| Dual master/output semantics for deterministic credential derivation | Conditional go as the domain-specific method core |
| Exact approval-budget profile for projection-bound credential tickets | Conditional go as the quantitative result |
| Credential exposure threshold spectrum | Conditional go as the strongest quantitative result; generic minimum-cost reachability remains prior art |
| Full-context digest binding | Repair produced by the method, not a new cryptographic primitive |

No exact-phrase web result was found for the working name DCCEA.  That search is
not a plagiarism clearance or proof of priority.  The contribution must always
be described through definitions, theorems, artifacts, and explicit
differences from published work—not through the name.

## Evidence result

| Evidence | Result |
|---|---|
| Unit tests | 16/16 pass |
| Joint security states | safe, scope-only, Root collapse plus full exposure |
| One-ticket exposure curve | `32, 32, 16, 8, 4, 2, 1` contexts |
| Service-only ticket profile, budgets 1--4 | `15, 30, 30, 30` unauthorized contexts |
| Exact-context ticket profile, budgets 1--4 | `0, 0, 0, 0` |
| Exposure threshold, exact scope | `2` domains for targets 1--32 |
| Exposure threshold, unscoped DPRF | `1` domain for targets 1--31; `2` for target 32 |
| Exposure threshold, collapsed Root | `1` domain for targets 1--32 |
| TLA+ exact scope | 28 generated, 9 distinct, depth 6, no violation |
| TLA+ wildcard scope | expected violation, 25 generated, 11 distinct, depth 5 |
| Analyzer median/P95 at 100,000 contexts | 3,555.38 / 3,863.90 ms |

The timing is a local exhaustive-analysis baseline, not an online cryptographic
performance claim.

## Why this is not yet a manuscript Go

PolyScope already combines permission expansion with reachable attack
operations, and MulVAL already computes rule closure.  A reviewer may still
classify DCCEA as an application-specific specialization unless the published
system corpus shows failures that conventional Root-only analyses actually
miss.  The current corpus verifies the literature boundary but does not yet
contain 12 independently reviewed message-level encodings.

Therefore the existing paper is not changed.  The next stage is empirical
validation of the method against published protocols; only after that gate is
passed should the contribution replace any architecture-only novelty claim.

## Exact proposed contribution sentence

If the gate passes, the defensible sentence is:

> We introduce a credential-specific dual-collapse analysis that separates
> master-capability reachability from authorization-indexed output exposure and
> computes an exact approval-budget exposure profile for projection-bound
> derivation interfaces; the analysis detects deployments that satisfy a
> nominal threshold or never reconstruct the master but still amplify one
> authorized evaluation into multiple credential outputs.

Do not use “first,” “novel cryptographic primitive,” “new access structure,” or
“provably secure system” in the claim.
