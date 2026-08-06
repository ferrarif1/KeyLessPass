# Recovery Design Review

Decision date: 2026-08-06  
Scope: one user, one local vault, three independently handled recovery factors, any two required

## Candidate comparison

Legend: Yes = inherent in the primitive; Protocol = supplied by the surrounding KeyLessPass protocol; No = not supplied.

| Candidate | True t-of-n | Less-than-threshold secrecy | Share authenticity / malicious-share handling | Root reconstructed | Rollback protection | Replace / revoke / refresh | Scale and libraries | Complexity, auditability, usability | Research effect |
|---|---:|---|---|---:|---|---|---|---|---|
| Three pairwise complete-key wrappers | No | Depends on pair-key security | AEAD authenticates each wrapper | Yes | No | Rewrap; three wrappers must remain consistent | Easy, mature AEAD | Low code complexity, misleading threshold semantics | Weakens credibility; looks like an engineering composition |
| Plain Shamir 2-of-3 | Yes | Information-theoretic for fewer than two valid shares | No / KCV only detects wrong reconstruction | Yes | No | Re-share same root; old threshold shares remain valid | Mature implementations; general t,n | Small API, but metadata and lifecycle remain undefined | Correct terminology, but adopting Shamir is not novelty |
| Authenticated and version-bound Shamir | Yes | Same as Shamir | Protocol MAC after reconstruction, protected local package, checksum, and KCV | Yes | Protocol | New share set for one-factor loss; new root after threshold compromise | `vsss-rs` plus standard HKDF/HMAC | Moderate and auditable; three recovery paths stay simple | Best match: credible foundation for lifecycle contribution |
| Feldman/Pedersen VSS | Yes | Feldman exposes a public commitment; Pedersen hides it | Detects invalid shares against verifier data | Yes | No | Possible, but verifier lifecycle is additional state | Available in `vsss-rs` | More group arithmetic and verifier material than this local dealer setting needs | Adds complexity without addressing the reviewers' main lifecycle failures |
| Proactive sharing / periodic share refresh | Yes | Yes between refresh epochs under its mobile-adversary assumptions | Depends on protocol | Usually during refresh protocols | No | Strong refresh semantics if parties erase old shares | Distributed protocols and assumptions | Poor fit: the three factors are passive storage, not continuously communicating parties | Hard to validate and easy to overclaim |
| Threshold cryptography / MPC | Yes | Yes | Protocol-specific | No for supported operations | No | Protocol-specific | Mature for some signing/decryption operations, not for this password derivation stack | Disproportionate; output still appears on a compromised endpoint | Solves a different problem and would obscure the systems contribution |
| TPM / Secure Enclave / security-token assisted recovery | Not by itself | Depends on split/wrapping protocol | Hardware can authenticate or protect a factor | Usually | Monotonic hardware may help locally | Device replacement and attestation workflows required | Platform APIs vary | Valuable defense-in-depth for the computer factor; not portable enough as the sole recovery design | Improves deployment credibility when described as a factor protector |
| SLIP-0039 recovery mnemonics | Yes | Yes | RS1024 checksum detects transcription errors, not adversarial freshness | Yes | No | Identifier/group metadata support; lifecycle still external | Mature specification; Rust choices reviewed were old or restrictive | Excellent human format, but wallet-oriented two-level semantics and library fit require more work | Candidate for a future phrase-format revision, not the current production primitive |

## Decision

Use **authenticated and version-bound Shamir 2-of-3 secret sharing** for the 256-bit Root Key:

- `vsss-rs 5.4.0`, `Gf256::split_array` and `Gf256::combine_array`, performs the finite-field operations. KeyLessPass does not implement interpolation or field arithmetic.
- Every share is carried in a `ShareEnvelope` binding schema, crypto suite, vault, root generation, share set, share index, threshold, factor type, factor identity, factor generation, creation time, and encoding version.
- An HMAC under a dedicated Root-Key-derived subkey authenticates envelope metadata after recovery. The local factor package receives platform protection. A four-byte SHA-256 checksum detects recovery-phrase transcription errors.
- A Root Key Confirmation Value binds `vaultID` and `rootGeneration` and rejects wrong reconstruction.
- A committed manifest selects the current share set. Factor files are generation-specific and the manifest is written last.

This choice gives the exact property requested by the scenario without inventing cryptography. VSS was rejected because a trusted local dealer creates all three shares at enrollment; pre-reconstruction public share verification does not solve rollback, revocation, or endpoint compromise. Proactive sharing and MPC were rejected because passive paper/computer/USB factors do not form an online distributed protocol.

## Important non-properties

Shamir does not provide authenticity, malicious-share identification, revocation, freshness, rollback resistance, or secure deletion. The current protocol detects a bad pair through KCV and envelope MAC but does not identify which member of the pair is malicious. A standard USB proves possession of a copyable file only. Recovery reconstructs the complete Root Key in process memory.

Re-sharing the same Root Key protects after loss or suspected exposure of one share only if the attacker did not already obtain another old share. If two old shares may be compromised, a new Root Key and coordinated service-password migration are required.

## Library risk review

`sharks 0.5.0` was not selected because RustSec advisory RUSTSEC-2024-0398 documents biased polynomial coefficients. `vsss-rs` exposes the required arbitrary-byte GF(256) API and includes zeroization support. It is an external implementation dependency, not a formal verification result or an independent audit claim. The exact version is pinned in `Cargo.lock` and exercised by fixed API, cross-set, tamper, and recovery-path tests.

## Recovery phrase format

`KLRP v1` serializes a recovery envelope into a compact binary structure, appends a four-byte checksum, and maps 11-bit groups to the public 2048-word English BIP-39 dictionary. It is **not BIP-39** and **not SLIP-0039**. The current 108-word output is robust but long; manual-entry usability is a measured limitation. A future format change requires a new `encodingVersion` and retained decoder test vectors.
