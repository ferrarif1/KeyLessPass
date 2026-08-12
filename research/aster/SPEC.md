# ASTER executable specification

## Canonical request

`AsterRequest` uses a tagged, length-delimited binary encoding beginning with `ASTER-REQUEST\0`. The stable test vector binds protocol version, operation, vault, service, account, lineage, credential salt, policy identifier and hash, policy epoch, Root-Epoch, generation, freshness generation, expiry, nonce, and use budget.

## Authorization

The Approval Authority signs the complete canonical request with Ed25519. Each evaluator validates the signature and the same field set before consuming the capability in a SQLite `BEGIN IMMEDIATE` transaction. Expiry, revocation, freshness, Root-Epoch, generation, lineage, policy hash/epoch, and nonce budget are independent checks. Projected and wildcard modes exist only as research negative controls.

## Exact-domain evaluation

The semantic evaluator implements ASTER's arbitrary-precision suffix counts, Rank/Unrank bijection, and FF1-AES-256 cycle-walking backend. The evaluator API releases only the derived credential. Its process-local Root-Epoch storage is not an MPC claim.

## Root-Epoch lifecycle

- Share refresh increments public sharing metadata but preserves the Root-Epoch secret and output family.
- Root-Epoch replacement samples an independent 256-bit secret.
- Cross-epoch history selection derives historical and candidate credentials inside the evaluator boundary and releases only the first accepted candidate.
- The SQLite migration journal persists public `(rootEpoch,generation)` descriptors before submission and stores no password column.
- Ambiguous evidence enters `UNKNOWN_OUTCOME` with both old and candidate descriptors.
- A referenced epoch cannot retire while committed, pending, unknown, or required-history state depends on it.
