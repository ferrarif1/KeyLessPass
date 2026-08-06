# Known Limitations

- The Flutter desktop screens and generated localization still use “mnemonic” wording in several flows. The Rust v3 artifact is an offline paper recovery share, but complete UI terminology and lifecycle conversion is unfinished.
- `KLRP v1` currently renders the paper recovery share as 108 English words. It is not intended for memorization, has no human-subject transcription study, and has no implemented QR presentation.
- The share-envelope HMAC is verified after Root Key reconstruction. With all three factors the implementation tries every pair and can identify the factor excluded from the only successful pair; with only one failing pair it cannot identify which member is bad.
- The managed factor uses the platform-provider abstraction. Linux/file fallback is weaker than TPM, Secure Enclave, Keychain, or DPAPI and is reported as degraded.
- A standard USB file is copyable. No uniqueness, unclonability, or hardware presence claim is made.
- Full Root Key material exists briefly in process memory during recovery and derivation. This is not threshold MPC.
- Re-sharing the same Root Key does not revoke two already compromised old shares. Threshold compromise requires a new Root Key.
- Automatic Root Key rotation is implemented only for an empty vault. A non-empty vault is rejected because safe completion requires staged remote password rotation for every service and handling partial completion.
- The SQLite freshness service is a persistent local CAS prototype, not a deployed or independently administered enterprise service. Local-only mode cannot detect rollback of every valid copy.
- Target-system adapters, account-lockout budgets, and automatic remote authentication checks for reconciliation are not included. The state machine and adapter-facing APIs are implemented.
- No Windows or Linux measurements were fabricated. Cross-platform builds remain CI targets until those runners execute the experiment.
- TLC exhaustively checks the bounded single-operation rotation model, but the cryptographic protocol, adapters, persistence implementation, and dependency composition have not received an independent external security audit or formal proof.
