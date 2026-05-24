# Security Policy

## Security Boundaries

KeyLessPass derives passwords locally and does not store target-system plaintext passwords, encrypted service-password vaults, or mnemonic phrases. The master key is randomly generated during enrollment and protected through local factor packages.

Display metadata such as record name, service hint, account hint, and notes is not part of the password derivation path. The stable derivation path uses `recordSeq`, `recordId`, `version`, `salt`, and `encodingDescriptor`.

New factor packages include package/schema version fields. New enrollments also include a protected mnemonic verifier so USB package rebuild from local material can reject an incorrect mnemonic without storing the mnemonic itself.

## Sensitive Data Handling

The following values must never be logged, exported in diagnostics, or displayed longer than needed:

- Mnemonic phrase
- Master key
- Local factor secret
- USB factor secret
- Recovery fragment plaintext
- Raw HKDF output
- AEAD/HMAC keys
- Derived password

## Supported Reports

Please report issues involving local storage integrity, factor package authentication, clipboard clearing, platform factor protection, or accidental exposure of sensitive data.

Support email placeholder: security@example.com

## Disclosure

This repository is currently prepared for productization and internal validation. For external distribution, configure a monitored security contact before publication.
