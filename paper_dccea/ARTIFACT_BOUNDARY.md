# Artifact boundary

## Paper claim supported

The artifact reproduces the finite credential-exposure analysis reported in
the manuscript:

- deployment-domain capability closure;
- unrestricted derivation capability (UDC) reachability and minimal witnesses;
- exact, wildcard, and projection-bound context exposure;
- approval-budget spill profiles;
- cardinality and independently minimized monotone set-cost CETS;
- a complete scoped credential-key derivation path using two RFC 9497 POPRF
  evaluations with signed tickets; and
- temporal TLA+ positive and single-fault negative controls.

## Source locations

- Analyzer and tests: `../research_upgrade/ccas_dprf/`
- Joint result: `../research_upgrade/ccas_dprf/results/dual_collapse_cases.json`
- Projection result and benchmark: `../research_upgrade/ccas_dprf/results/`
- TLA+ models and results: `../research_upgrade/ccas_dprf/formal/`
- POPRF reference protocol and results:
  `../research_upgrade/cets_reference_protocol/`
- Novelty/correctness audit: `../docs/dccea_novelty_correctness_audit.md`

## Reproduction commands

From `research_upgrade/ccas_dprf/`:

```bash
export PYTHONDONTWRITEBYTECODE=1
python3 -m unittest -v \
  test_ccas_analyzer.py \
  test_context_exposure.py \
  test_dual_collapse_analyzer.py
python3 dual_collapse_analyzer.py dual_cases.json \
  --output results/dual_collapse_cases.json
```

From `research_upgrade/ccas_dprf/formal/`, using TLA+ tools 1.7.4:

```bash
java -cp ../../../tmp/tla2tools-v1.7.4.jar tlc2.TLC \
  -config TemporalTicketsExact.cfg TemporalTickets.tla
```

The projected, expired, revoked, stale-generation, and replay configurations
are intentional negative controls and must produce the named counterexample.

From `research_upgrade/cets_reference_protocol/`:

```bash
cargo test --locked
cargo run --release --locked -- \
  --output results/reference_protocol.json
```

## Not supported

The artifact invokes a published POPRF library but does not verify that library
or implement a threshold-shared single PRF key, hardware token, production
recovery transport, or real service adapter. It does not classify any cited
published system as vulnerable. The finite models do not establish
probabilistic, side-channel, unbounded-time, liveness, or
deployment-independence claims. In the reference protocol, the stable client
input is endpoint-held and therefore is not modeled as an independent
authorization factor after endpoint compromise. Its minimum UDC witness is the
endpoint plus approval authority; the endpoint plus both evaluator domains is
a separate three-domain witness. The evaluator pair alone is not a UDC
witness. The temporal model represents UDC acquisition abstractly rather than
assigning it to evaluator-key compromise.
