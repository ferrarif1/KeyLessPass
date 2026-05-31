# Security Notes

- Randomness comes from OS CSPRNG through Rust `rand`/`getrandom`.
- Mnemonic phrase is never persisted.
- Target-system plaintext passwords are never persisted.
- There is no encrypted service-password vault.
- CDR and factor packages carry integrity tags.
- Factor packages and recovery metadata carry schema/version fields.
- New enrollments include a protected mnemonic verifier for recovery checks; the mnemonic itself is not stored.
- Passwords are shown/copied only briefly; Flutter clears clipboard after 30
  seconds by default.
- Logs must not include mnemonic, master key, factors, USB payload plaintext, or
  derived passwords.
- Client-only rollback detection is limited to local/USB metadata and MAC checks.
  Coordinated rollback of every local copy requires an external trusted state or
  append-only audit integration.

## 2-of-3 Recovery Boundary

- `Kmaster` is a random 256-bit root secret generated during enrollment.
- `Kmaster` is not persisted as plaintext in the local factor package or the USB
  factor package.
- `Kmaster` is protected at rest only by `W_MC`, `W_MU`, and `W_CU` pairwise
  wrapper ciphertexts.
- Mnemonic + this computer recovers through `W_MC`.
- Mnemonic + USB package recovers through `W_MU`.
- This computer + USB package recovers through `W_CU` and can reset the mnemonic
  without the old mnemonic.
- Normal password derivation uses mnemonic + this computer through `W_MC`; the
  USB package can stay offline during daily use.
- A single factor alone cannot recover `Kmaster`.
- A USB package is ordinary copyable storage. Copying it copies the USB factor,
  but the copied USB factor still needs either the mnemonic factor or the
  matching computer factor.

## V2 Package Notes

- Local package stores local metadata, mnemonic verifier, `W_MC`, and optional
  `W_CU`; it does not store `usbSecret`.
- USB package stores USB factor material, `W_MU`, and `W_CU`; it does not store
  `deviceSecret`.
- The V2 `encryptedPayload` field name is historical. In V2 it is a base64
  encoded factor payload and does not contain plaintext `Kmaster`.
