# TLC result

- Date: 2026-08-06
- Tool: TLC2 2026.07.31.184830 (`tla2tools.jar` SHA-1
  `feffd16994db963ad945628cfd03d154c195a468`)
- Java: Oracle Java 21 on x86-64 macOS
- Specification: `PasswordRotation.tla`
- Configuration: `PasswordRotation.cfg`
- Result: no invariant violation
- Search: breadth-first, 17 states generated, 16 distinct states, depth 6,
  zero states left on the queue

The checked invariants were `TypeInvariant`, `NoUnconfirmedCommit`,
`AmbiguityNeverCommits`, and `AbortNeverCommits`. The model contains one
rotation operation and all four reconciliation observations. It does not model
cryptographic primitives, retry timing, lockout counters, SQLite, or a network
adapter.
