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
