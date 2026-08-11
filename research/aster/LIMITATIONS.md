# ASTER evidence boundaries

## Cryptographic backend

The checked-in artifact separates a process-local semantic evaluator from a real MP-SPDZ feasibility circuit.

- The semantic backend holds Root-Epoch keys and supports protocol/state experiments only; its timings are not threshold-performance evidence.
- The MP-SPDZ backend uses `mal-shamir-bmr-party.x` with three parties at corruption threshold $t=1$ and five parties at $t=2$. Every configured party participates online, so these are not arbitrary-subset availability claims.
- The MPC circuit is an AES-based Feistel/cycle-walk composition over a fixed 20-bit superset domain. It is not FF1 and not a new threshold primitive.
- RQ6 contains three single-host loopback repetitions per configuration. It establishes fixed-vector agreement and concrete feasibility cost, not LAN/WAN or production latency.
- DKG, share refresh, history-window MPC, malicious-input attribution, and production deployment remain outside the measured boundary.

## Adapter boundary

The fault matrix uses a real loopback HTTP service and an independent TCP process with durable verifier hashes and LDAP-style modify/bind/readback semantics. A separate pinned single-server OpenLDAP container passed a modify/new-bind/old-bind-rejection smoke test. Neither experiment models a replicated directory cluster, failover, WAN conditions, or production performance.

## Policy and permutation limits

Unsupported source-policy semantics are rejected instead of approximated. All 121 exactly translated policies compiled; the configured permutation implementation completed full sequences for 97 and failed closed for 24 domains above its 512-bit ceiling. Exact generation does not make a low-entropy legacy domain resistant to exhaustive search.

## Formal-method and secret-erasure boundaries

TLC exhaustively checks a bounded abstract state machine and verifies that eight broken configurations expose counterexamples. It is not a cryptographic proof or a refinement proof of the Rust and adapter implementations. The endpoint inventory and journal-schema checks do not prove absence from process memory, swap, operating-system crash dumps, or unrelated files.
