# TLC result

- Last reproduced: 2026-08-11
- Tool: TLC2 2.19 (`tla2tools.jar` v1.7.4, SHA-256
  `936a262061c914694dfd669a543be24573c45d5aa0ff20a8b96b23d01e050e88`)
- Java: Oracle Java 21 on x86-64 macOS
- Specification: `PasswordRotation.tla`
- Configuration: `PasswordRotation.cfg`
- Result: no invariant violation
- Search: breadth-first, 172 states generated, 79 distinct states, depth 14,
  zero states left on the queue
- Reproduction seed: `-4601088482909617910`

The checked invariants were `TypeInvariant`, `NoUnconfirmedCommit`,
`OpaqueTargetNeverCommits`, `OverlapIsContractBound`, and
`AtomicBothNeverCommits`.

The model nondeterministically selects atomic replacement, overlap-then-revoke,
or opaque replacement; explores both authentication-probe orders, lost update
and revocation responses, evidence refinement, overlap establishment, old
credential revocation, and local commit. It does not model cryptographic
primitives, retry timing, lockout counters, SQLite, adapter implementation, or
whether an adapter's claimed endpoint coverage is true.
