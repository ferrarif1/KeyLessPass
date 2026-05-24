# Contributing

Thank you for improving KeyLessPass. Keep changes aligned with the local-only password derivation model.

## Rules

- Do not add network sync, cloud accounts, browser autofill, or a web backend.
- Do not store target-system plaintext passwords.
- Do not store mnemonic phrases.
- Do not make the mnemonic phrase the service-password root seed.
- Do not log sensitive material.
- Keep derivation based on stable CDR fields, not mutable display metadata.

## Checks Before Submitting

```bash
cd rust_core
cargo test

cd ../flutter_app
flutter analyze
flutter test
```

Run a sensitive-term scan before release and review any findings manually.
