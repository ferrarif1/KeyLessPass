# Security Notes

- Randomness comes from OS CSPRNG through Rust `rand`/`getrandom`.
- Mnemonic phrase is never persisted.
- Target-system plaintext passwords are never persisted.
- There is no encrypted service-password vault.
- CDR and factor packages carry integrity tags.
- Passwords are shown/copied only briefly; Flutter clears clipboard after 30
  seconds by default.
- Logs must not include mnemonic, master key, factors, USB payload plaintext, or
  derived passwords.
- MVP client-only rollback detection is limited to local/USB metadata and MAC
  checks. Coordinated rollback of every local copy is outside the MVP guarantee.
