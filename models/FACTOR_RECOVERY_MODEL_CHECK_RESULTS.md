# Factor-preserving recovery TLC result

- Last reproduced: 2026-08-11
- Tool: TLC2 2.19 (`tla2tools.jar` v1.7.4, SHA-256
  `936a262061c914694dfd669a543be24573c45d5aa0ff20a8b96b23d01e050e88`)
- Java: Oracle Java 21 on x86-64 macOS
- Specification: `FactorPreservingRecovery.tla`
- Configuration: `FactorPreservingRecovery.cfg`
- Result: no invariant violation
- Search: breadth-first, 438,050 states generated, 7,296 distinct states,
  depth 22, zero states left on the queue
- Reproduction seed: `-4542227193850536472`

The checked invariants were `TypeInvariant`,
`NoUnauthorizedNetworkRelease`, `NoSingleShareRecovery`,
`NetworkAloneCannotRecover`, `StaleEpochCannotRelease`, and
`NoDuplicateNodeContribution`.

The finite model separates the managed-device share, USB share, peer-storage
fragments, administrator approvals, threshold-node responses, object epoch,
network-share release, Root-Key recovery, and mandatory share-set refresh. It
does not model cryptographic primitives, Byzantine OPRF responses, traffic
analysis, approval-service implementation, or actual libp2p routing.
