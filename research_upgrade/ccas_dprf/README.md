# CCAS and non-reconstructing derivation research branch

This directory is an isolated research artifact for capability closure,
context exposure, and dual-collapse analysis. Manuscripts and submission files
are outside the versioned repository boundary.

## Capability-closure analyzer

Run the six required deployment cases:

```bash
python3 ccas_analyzer.py cases.json --output results/effective_access_cases.json
python3 -m unittest -v test_ccas_analyzer.py
```

The analyzer computes the monotone least fixed point of automatically
exercisable protocol rules for every deployment-domain subset.  It then maps
the shares in each closure to the nominal cryptographic access structure and
reports the effective structure, cardinality threshold, compromise-set cost,
minimal domain sets, and one derivation trace per minimal set.

The implementation intentionally uses exhaustive subset enumeration.  That is
appropriate for the small trust-domain models evaluated here and avoids an
unsupported complexity or synthesis claim.

## Context-exposure analyzer

The companion analyzer measures authorization amplification: how many distinct
credential contexts become derivable when a user approves one context and the
token checks only a projection of the canonical context.

```bash
python3 context_exposure.py context_cases.json \
  --output results/context_exposure_cases.json
python3 -m unittest -v test_context_exposure.py
```

It reports the exposed equivalence class for each binding policy and enumerates
minimal collision-free field sets in the finite test corpus.  This is an
authorization/interface analysis, not a cryptographic DPRF experiment.

The standalone scalability run reports median and nearest-rank P95 latency:

```bash
python3 benchmark_context_exposure.py --repetitions 5 \
  --output results/context_exposure_performance.json
```

## Dual-collapse report

The joint report computes Root reachability and credential spill from the same
compromised-domain set and approved-context set.  In a Root-derived credential
profile, three states are reachable: neither failure, authorization
amplification without Root access, and Root access with full-context exposure.
The apparent "factor collapse only" state is rejected because a reachable Root
and public derivation algorithm expose every modeled context.

```bash
python3 dual_collapse_analyzer.py dual_cases.json \
  --output results/dual_collapse_cases.json
python3 -m unittest -v test_dual_collapse_analyzer.py
```

The implementation is deliberately small and does not present fixed-point
closure or projection-key discovery as new algorithms.
Its Python API accepts an arbitrary finite monotone set-cost oracle; additive
per-domain weights remain the command-line reference model.

## Abstract model

The TLA+ model separates routine derivation from recovery provisioning and
checks capability-threshold and context-confinement invariants:

```bash
java -cp ../../tmp/tla2tools-v1.7.4.jar tlc2.TLC \
  -config formal/CCASRoutine.cfg formal/CCASRoutine.tla

java -cp ../../tmp/tla2tools-v1.7.4.jar tlc2.TLC \
  -config formal/DualCollapseUnifiedExact.cfg formal/DualCollapseUnified.tla

java -cp ../../tmp/tla2tools-v1.7.4.jar tlc2.TLC \
  -config formal/TemporalTicketsExact.cfg formal/TemporalTickets.tla
```

The model abstracts partial evaluations as protocol events.  It neither models
nor claims to verify the algebra of any DPRF construction.
The wildcard and Root-amplification configurations are negative controls and
are expected to produce `NoAuthorizationAmplification` counterexamples.
`TemporalTickets.tla` additionally models issue, expiry, revocation, replay,
freshness generation, and UDC dominance.  Its projected, expired, revoked,
stale, and replay configurations are single-fault negative controls.
UDC acquisition deliberately abstracts deployment-specific witnesses.  For the POPRF
reference deployment, the minimum witness is endpoint plus approval authority;
endpoint plus both evaluators is an alternate three-domain witness, while the
evaluator pair alone lacks the endpoint-held client input.

## Cryptographic reference case

The sibling directory `../cets_reference_protocol/` contains a complete
two-server derivation case. It composes two independent RFC 9497 POPRF
evaluations, verifies both proofs, checks an Ed25519 approval ticket at each
server, and combines both outputs. It is a reference protocol rather than a
new threshold-PRF construction.

## Cryptographic boundary

No new PRF is implemented in this directory.  A deployable non-reconstructing
profile requires a reviewed threshold/distributed PRF and an isolated token
whose raw share never enters the endpoint.  The prior-art and Go/No-Go reports
record why a toy group construction would not be valid evidence.
