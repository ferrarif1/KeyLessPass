# Synchronization, Conflict, and Rollback

## Replica model

Each CDR carries a `replicaID`, Lamport clock, epoch, operation ID, and authenticated parent hash. `compare_replicas` returns one of: identical, left/right descendant, left/right stale, concurrent modification, forked history, duplicate-operation conflict, or cannot merge. Cross-vault and cross-record objects never merge. `recordSeq` is ordering metadata, not a synchronization protocol.

## Integrity, authenticity, freshness, rollback

- HMAC detects modification without the Root Key.
- The Root-Key-derived HMAC authenticates a record to a vault generation.
- Parent hashes expose missing-parent and fork relationships when at least one newer reference is present.
- None of these proves that a complete, internally valid copy is current.

## Local-only mode

Local and USB copies can reveal partial rollback, divergent histories, and stale copies when another valid copy is newer. If every valid local copy is restored together to the same old state, the system cannot distinguish rollback from legitimate old state. The implementation reports this mode as `local_only_unanchored` and makes no stronger claim.

## Enterprise-anchored mode

`FreshnessService` stores only `vaultID`, latest `rootGeneration`, CDR epoch, operation-log digest, and update time. It stores no service password or Root Key. `compare_and_set` rejects non-monotonic updates and concurrent writers. Evaluation yields current, needs publish, rollback detected, fork detected, or offline read-only.

Offline enterprise mode is read-only for mutations. Password retrieval may be permitted by deployment policy with an explicit degraded-status warning, but root rotation, share-set commit, and CDR rotation must wait for anchor availability if the deployment promises anchored freshness.

The repository includes `InMemoryFreshnessService` as an executable local/test implementation. A production remote transport, authentication policy, retention policy, and privacy review are outside the current prototype boundary.
