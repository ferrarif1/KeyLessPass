# Security Policy and v3 Claim Boundary

Report vulnerabilities privately to `revanton@icloud.com`. Include the affected commit, reproduction steps, impact, and mitigation; never include production credentials or customer secrets.

## What v3 protects

- The service-password inventory contains canonical metadata, not plaintext or encrypted service-password entries.
- A random 256-bit Root Key is split with mature Shamir 2-of-3 secret sharing. One share is insufficient at the recovery API boundary.
- Share envelopes bind vault, Root-Key/share-set/factor generations, factor role, suite, and encoding version. Root-Key-derived HMAC and a KCV reject modified metadata or the wrong reconstruction.
- Purpose-specific Root-Key subkeys are domain separated.
- CDR MACs and parent hashes detect tampering, replay, and partial-replica inconsistency.
- The staged rotation protocol persists uncertain remote outcomes and requires reconciliation before local activation.

## What v3 does not protect

- An ordinary USB share file is copyable. It proves possession of bytes, not possession of a unique physical device.
- A fully compromised endpoint can read the reconstructed Root Key or generated password in process memory and can capture a recovery phrase while it is entered.
- When a USB is connected to a compromised managed computer, those two factors are no longer operationally independent.
- Standard Shamir reconstruction creates the complete Root Key in memory; this is not threshold MPC.
- HMACs, hashes, and secret sharing do not detect coordinated rollback of every valid local copy. That claim requires an external monotonic freshness anchor.
- Envelope authentication detects a bad pair after reconstruction but does not identify which member is malicious.
- Share-set refresh invalidates mixing old and new single shares, but it cannot revoke an attacker who already holds any two old shares. Threshold compromise requires Root-Key rotation and rotation of every affected remote service password.

## Current implementation boundary

The Rust core implements v3 creation, all three recovery pairs, share-set refresh, recovery/USB/computer replacement, empty-vault Root-Key rotation, v2 migration, canonical CDRs, policy encoding, staged rotation, conflict classification, and a freshness-service interface. The Flutter enrollment UI still creates legacy v2 packages; v3 currently requires the migration API. Root-Key rotation for a non-empty vault, target-specific rotation adapters, production freshness transport, and a complete v3 desktop recovery UX are not shipped and are not security claims.

Use only test accounts during evaluation. Do not treat the prototype as production-approved without independent cryptographic review, platform hardening, adapter-specific lockout analysis, and organization approval.
