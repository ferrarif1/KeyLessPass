# Exposure and factor-preserving recovery revision report

Date: 2026-08-09

## Outcome

The manuscript and active artifact now center on two connected technical axes:

1. an exact generation-indexed credential sequence with explicit
   known-password, Kcred, Kroot, verifier, and rollback semantics;
2. a closure-based factor-preserving Root-Key recovery profile that prevents an
   endpoint holding one share from automatically obtaining the network share.

The active design contains no Data Key/View Key split, threshold OPRF,
opaque-object scan, erasure-coded ciphertext, or service-password recovery
object. The new paper contains no prior manuscript comparison, unpublished
local encoder, or unverified preprint comparison.

## Implemented changes

- Added credential-salt rekeying and lineage identifiers for Kcred compromise.
- Added `shareSetGeneration` to manifests and lifecycle operations.
- Distinguished ordinary re-sharing from Root-Key replacement.
- Replaced the old peer-recovery prototype with Shamir 3-of-5 network
  fragments, two independent Ed25519 approvals, generation/session-bound
  tickets, X25519/HKDF/AES-GCM fragment release, and a replay ledger.
- Expanded the freshness checkpoint across Root-Key, share-set, CDR, policy,
  credential generation, and lineage dimensions.
- Added known-credential, password-space adequacy, recovery, rollback, and
  adversarial experiments.
- Added recovery-access and integrated freshness/compromise TLA+ models while
  retaining the original lifecycle model.

## Verified evidence

- 85 Rust tests passed; strict all-target/all-feature Clippy passed.
- Public corpus: 120 of 121 exact translations compile under the fixed budget;
  96 meet backend bounds and 116 exceed the illustrative 80-bit floor.
- Ideal known-pair check: support is exactly `11-q`, with no observed-output
  recurrence for q=0..5.
- Recovery: all ten 3-of-5 combinations work; local three-node release and
  reconstruction average 6.036 ms, excluding network and human delay.
- TLC distinct states: lifecycle 1,006,128; recovery 852,704; integrated 40,292.
- Final manuscript: 31 pages, SHA-256
  `e6d509fba4f79432ac513390f9c7643295231920d6f0e6431b649160b1980407`.

## Acceptance assessment

Innovation is now more assessable and defensible than a component-integration
paper. The highest remaining rejection risks are still insufficient novelty
relative to established finite-domain and recovery systems, absence of a
proof-matched total finite-domain backend, and lack of a real multi-host
recovery/target-adapter evaluation. The manuscript states these as limitations
instead of implying production readiness.
