# KeyLessPass v3 Design

## Research position

KeyLessPass is a lifecycle architecture for enterprise systems that still accept only textual passwords. It does not propose a new KDF, AEAD, secret-sharing primitive, or password-policy language. Its research object is the cross-layer protocol that keeps recovery generations, credential generations, remote rotation outcomes, replicas, and optional freshness state consistent under loss, crashes, and concurrency.

AutoPass already supports deterministic generation, forced changes, and site rules; PALPAS already combines a high-entropy seed, per-service salts, synchronization, and policy-aware generation. MFDPG adds multi-factor deterministic generation and password rotation without stored credential secrets. MFKDF supplies threshold multi-factor key derivation and client-side recovery. KeyLessPass therefore claims neither deterministic generation, threshold recovery, nor component integration alone as novelty. The narrower contribution is an implemented state model joining random-Root-Key share lifecycle, canonical Credential Description Records (CDRs), pending-confirm-reconcile rotation, replica conflict classification, and freshness anchoring for password-only legacy targets.

## Security core

The v3 Root Key is 256 random bits. Purpose-specific keys are derived with HKDF-SHA256 using fixed labels and context binding to `vaultID`, `rootGeneration`, and `cryptoSuiteVersion`. The Root Key is split by `vsss-rs` into three GF(256) Shamir shares at threshold two:

- paper recovery share: a printable offline package currently encoded as 108 checksum-protected words and not intended for memorization;
- managed-computer share: protected by the platform provider;
- USB share: stored on ordinary copyable removable media.

Each share envelope binds the vault, Root-Key generation, share-set ID, share index, threshold/count, factor type/ID/generation, suite, timestamp, and encoding version. A Root-Key-derived HMAC authenticates the envelope after reconstruction; a KCV confirms the reconstructed Root Key. A committed manifest is written last and selects the only active set. Shamir itself does not provide authenticity, malicious-member identification, revocation, or freshness.

Legacy v2 pairwise complete-key wrappers remain readable only for migration. Migration validates all three legacy paths, preserves the Root Key, writes and validates v3 artifacts, commits the v3 manifest, and can archive the old packages. The current Flutter enrollment workflow still creates v2 packages; v3 selection currently uses the Rust migration operation.

## CDR and derivation boundary

CDR v3 uses RFC 8785 JSON canonicalization. `recordID` is a stable logical credential identity; `recordSeq` is a stable vault-local ordinal retained for deterministic compatibility and human/audit ordering. `credentialGeneration` advances one service password, while `rootGeneration` advances the vault Root Key.

Password-changing inputs are the Root-Key generation, stable service/account identifiers, credential generation, 128-bit salt, derivation/encoder versions, and the hash of policy identity, version, and encoding descriptor. `recordID`, `recordSeq`, storage `version`, display fields, notes, replica clocks, and rotation evidence do not alter derivation-v2 output. Any encoding-policy change requires a new credential generation.

Encoder v2 uses a domain-separated HMAC stream, rejection sampling for modulo-bias-free bounded indices, Fisher--Yates shuffling with the same index sampler, randomized mandatory-character placement, explicit min/max class counts and edge/repetition/sequence rules, contradiction detection, and a bounded attempt count. It never silently relaxes a policy. This does not imply uniform sampling over the complete set of policy-valid strings; reported entropy is an upper bound when policy constraints overlap.

## Lifecycle protocol

Rotation is not two-phase commit. A legacy target is not a transactional participant. The persistent states include `STABLE`, `PREPARED`, `UPDATE_SENT`, `REMOTE_CONFIRMED`, `LOCAL_COMMITTED`, `UNKNOWN_OUTCOME`, `RECONCILIATION_REQUIRED`, `AMBIGUOUS_REMOTE_STATE`, `ROLLBACK_REQUIRED`, `ABORTED`, and `SUPERSEDED`.

After a timeout or crash with an unknown remote result, reconciliation tests the candidate password and old password within a lockout budget. New-only success permits local commit; old-only success aborts; both succeeding enters `AMBIGUOUS_REMOTE_STATE`; neither succeeding requires manual recovery. The external adapter and lockout-budget UI are interfaces, not production implementations in this repository.

Replica comparison distinguishes descendant, stale, concurrent, forked, replayed, and cross-vault state using parent hashes, credential generations, operation IDs, and replica metadata. Local-only mode detects tampering and partial-copy inconsistency but cannot detect rollback of every valid local copy. Enterprise-anchored mode exposes a compare-and-set freshness interface over Root-Key generation, CDR epoch, and digest; the included SQLite implementation proves persistence and atomic CAS semantics locally, but is not a deployed network service.

## Implementation map

- Recovery: `rust_core/src/crypto/recovery.rs`, `storage/recovery_store.rs`
- Factor lifecycle and migration: `service/recovery_lifecycle.rs`, `service/migration.rs`
- CDR and encoder: `domain/cdr.rs`, `crypto/encoder.rs`
- Rotation: `domain/rotation.rs`, `service/rotation.rs`
- Sync and freshness: `domain/sync.rs`, `service/freshness.rs`
- Fixed vectors: `rust_core/test-vectors/`
- Reproducible experiment: `rust_core/examples/research_evaluation.rs`

The complete claim boundary is maintained in `LIMITATIONS.md` and the reproducibility commands in `REPRODUCIBILITY.md`.
