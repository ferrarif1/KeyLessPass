# Privacy Policy Draft

KeyLessPass is designed as a local-only desktop application.

## Data That Stays On Your Device

KeyLessPass stores protected local state, CDR metadata, USB factor packages, optional USB CDR metadata backups, and recovery metadata. These files are used only to support local password derivation and recovery.

KeyLessPass does not store target-system plaintext passwords and does not maintain an encrypted service-password vault.

USB CDR backups contain credential metadata such as record sequence, record ID, version, salt, display labels, account hints, and integrity tags. They do not contain derived service passwords or mnemonic phrases.

## Data Not Collected

KeyLessPass does not collect, upload, sell, or share:

- Derived service passwords
- Mnemonic phrases
- Master keys
- Local factor secrets
- USB factor secrets
- Raw HKDF output or cryptographic keys
- CDR records
- Service names, URLs, account hints, or notes
- Device analytics

## Network Access

The desktop client is local-only by default. It does not require a cloud account, does not sync to a server, and does not include browser autofill or a web backend.

## Diagnostics

Diagnostics export is intended to exclude sensitive data. Do not attach real USB factor packages, local factor packages, CDR databases, mnemonic phrases, or derived passwords to support requests.

## Contact

Support email placeholder: support@example.com
