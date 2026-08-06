# Threat Model

## Protected assets

The 256-bit Root Key, current recovery shares, derived service passwords, CDR derivation fields, lifecycle state, operation evidence, and current freshness state are protected. Display metadata confidentiality is not a primary goal, although its integrity is authenticated.

## Entities and trust assumptions

Entities are the user, KeyLessPass process, managed computer, recovery card, ordinary USB, optional enterprise freshness service, and password-only target systems. OS randomness and standard primitives/libraries are trusted. A platform protector is trusted only to the level reported by `PlatformSecurityStatus`. The user keeps factors separately and verifies the intended target before using a password. The enterprise anchor, when configured, is authenticated and monotonic.

## Attacker capabilities

An attacker may copy any one stored factor, alter or replay files, substitute shares, restore snapshots, observe network ambiguity, learn one service password, or cause crashes at persistence boundaries. Stronger cases include two-factor compromise, infected USB host, process-memory read, binary replacement, and complete endpoint control.

## Security goals

- Fewer than two valid current shares do not reveal the Root Key.
- Cross-vault, cross-share-set, cross-generation, factor-type, index, and suite substitution is rejected.
- CDR and share metadata modification is detected.
- Service passwords are deterministically policy-compliant without modulo bias or silent policy relaxation.
- A remote outcome is never inferred from a missing response.
- Partial rollback, stale copies, forks, duplicate operations, and anchored full rollback are detected within stated assumptions.

## Non-goals

The system is not phishing resistant, not threshold MPC, not a hardware-token protocol, and not protection against a fully controlled endpoint during legitimate use. It does not make ordinary USB files unique or uncopyable. Local-only mode does not detect coordinated rollback of every valid copy.

## Capability matrix

| Attacker capability | Security goal | Claimed resistance | Required assumptions | Residual risk | Implementation evidence |
|---|---|---|---|---|---|
| Obtain one share / photograph recovery phrase | Root secrecy | Shamir threshold prevents reconstruction | Correct library and independent storage | Share can later combine with another compromised old share | Three-combination and cross-set tests |
| Obtain two current shares | Limit damage | No resistance: Root Key is recoverable | None | All current root-derived state exposed | Explicit root-rotation rule and limitation |
| Read managed-computer disk | Protect computer share at rest | Platform-protected generation file | OS/provider not compromised; fallback status heeded | File fallback is weaker; runtime use exposes plaintext | `protect_local_package`, platform status |
| Lose/copy ordinary USB | Treat copy as factor compromise | New share set prevents old USB mixing with new shares | Attacker has not also copied another old share | Old USB plus any second old share still recovers old root | `replace_usb_factor`, generation tests |
| Insert USB into infected computer | Preserve factor separation | No strong resistance after both factors are available to malware | Endpoint remains trustworthy | Malware can copy USB and managed share or read Root Key | Stated non-goal |
| Full endpoint control / process-memory read | Prevent root/password capture | Not claimed | Trusted endpoint during operation | Root Key and generated password can be captured | Explicit limitation; zeroization reduces duration only |
| Replace application binary/UI | Preserve execution integrity | Outside cryptographic protocol | Signed distribution and enterprise controls | Malicious binary can exfiltrate secrets | Deployment limitation |
| Modify CDR | Detect tampering | JCS HMAC rejects modification | Root Key remains secret | Compromised Root Key enables forgery | `corrupt_cdr_mac_fails` |
| Substitute or edit a share | Reject wrong reconstruction | Envelope binding, KCV, and metadata HMAC | Current manifest available | Pair is rejected but malicious member is not identified | Recovery tamper tests |
| Cross-vault replay | Isolation | Vault IDs and vault-derived keys reject | Correct manifest | Full compromise of destination root remains fatal | Cross-vault test |
| Mix share sets / root generations | Reject stale composition | Pre-combine metadata checks and current manifest | Manifest not fully rolled back | Coordinated rollback needs anchor | Cross-set/generation tests |
| Replay revoked old single share | Revocation after one-factor loss | New manifest rejects mixing | Other old shares not compromised | Old threshold still valid for old root | Lifecycle tests and limitation |
| Roll back one replica | Detect staleness | Parent hash, generation, replica comparison | A newer copy/anchor remains | None if comparison source trustworthy | `compare_replicas` tests |
| Roll back all local replicas | Detect global rollback | Only enterprise anchor detects | Anchor reachable and current | Local-only mode cannot detect | Freshness tests |
| Crash during factor write | Preserve committed set | Generation files plus manifest-last commit | Filesystem atomic rename/fsync | Orphan staged files may remain | Recovery-store round trip |
| Timeout after remote password update | Avoid wrong local commit | `UNKNOWN_OUTCOME` and reconciliation | Adapter records truthful evidence | Lockout risk; manual handling may be required | Rotation transition tests |
| Remote accepts password but returns error | Discover actual state | Test new then old within attempt budget | Target supports safe authentication check | Neither may work or account may lock | Reconciliation API/state |
| Concurrent replica rotations | Detect conflict | Operation ID, parent hash, replica relation | Both histories available | No automatic semantic merge | Sync tests |
| Policy changes / hidden rejection | Never silently weaken rules | Compiler validation, bounded attempts, explicit failure | Policy description is accurate | Undocumented server rules still require staged retry | Encoder tests |
| Encoder or crypto-suite downgrade | Fail closed | Version fields authenticated and exact-version checks | Current manifest/record not globally rolled back | Anchoring required for full freshness | Version validation tests |

## Residual-risk statement

The design primarily protects stored state against single-factor compromise, loss, partial replica attacks, policy-encoding errors, and lifecycle failures. Once a hostile endpoint simultaneously observes two factors or reads the reconstructed Root Key, the threshold boundary has been crossed.
