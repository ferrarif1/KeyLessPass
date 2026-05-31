# Security Policy

KeyLessPass is designed as a local password derivation tool. It does not aim to store service passwords, but users are still responsible for protecting their mnemonic phrase, local device, USB authentication factor, recovery materials, and operating environment.

## Reporting a Vulnerability

If you discover a security vulnerability, please do not disclose it publicly before it has been reviewed and addressed.

Please report security issues to revanton@icloud.com with:

- A clear description of the issue
- Affected version or commit
- Steps to reproduce
- Potential impact
- Suggested mitigation, if available

Do not include real production passwords, enterprise secrets, customer credentials, private keys, or sensitive business data in vulnerability reports.

## Evaluation and PoC Safety

During evaluation or proof-of-concept testing:

- Use test accounts and test data only.
- Do not use real enterprise production credentials unless expressly authorized.
- Do not deploy the software as a production credential management system without a commercial license and formal approval.
- Validate the security model before using it in sensitive environments.

## Security Boundary

KeyLessPass may reduce risks associated with stored password vaults by avoiding storage of service passwords. However, security still depends on:

- Strength and secrecy of the mnemonic phrase
- Protection of the USB authentication factor
- Security of the local device
- Integrity of the application binary
- Secure backup and recovery procedures
- User operational discipline
- Enterprise endpoint and access control policies

## 2-of-3 Local Recovery Model

The Rust core follows the paper-aligned local recovery model:

- `F_M = KDF(Normalize(mnemonic), saltM)`
- `F_C = KDF(deviceSecret || deviceID || userID, saltC)`
- `F_U = KDF(usbSecret || usbID || userID, saltU)`
- `W_MC = AES-256-GCM(HKDF(F_M || F_C, "KeyLessPass/wrap/MC"), Kmaster)`
- `W_MU = AES-256-GCM(HKDF(F_M || F_U, "KeyLessPass/wrap/MU"), Kmaster)`
- `W_CU = AES-256-GCM(HKDF(F_C || F_U, "KeyLessPass/wrap/CU"), Kmaster)`

`Kmaster` is randomly generated during enrollment. It is not derived from the mnemonic and is not persisted as plaintext in local or USB payloads. At rest, it exists only inside the `W_MC`, `W_MU`, and `W_CU` wrapper ciphertexts.

The recovery paths are:

- Mnemonic + this computer: recover through `W_MC` and rebuild a paired USB package.
- Mnemonic + USB package: recover through `W_MU` and rebuild this computer's local factor.
- This computer + USB package: recover through `W_CU` and reset the mnemonic without the old mnemonic.

Normal password derivation uses mnemonic + this computer through `W_MC`; the USB package is not required for daily derivation and can remain offline until setup or recovery.

A single factor alone is not sufficient to recover `Kmaster`. A USB package is ordinary copyable storage and should be treated as a copyable factor container, not as an uncopyable hardware key.

## Package Storage Boundary

- Local factor package: stores local metadata such as `deviceId`, `saltC`, `mnemonicSalt`, mnemonic verifier, `W_MC`, optional `W_CU`, recovery generation, and schema/version metadata. It does not store plaintext `Kmaster` or `usbSecret`.
- USB factor package: stores USB metadata and factor material such as `usbId`, `saltU`, `usbSecret`, `W_MU`, `W_CU`, recovery generation, and schema/version metadata. It does not store plaintext `Kmaster` or `deviceSecret`.
- CDR backup: stores credential description metadata and MACs only. It does not store service-password plaintext.
- macOS `com.keylesspass.local-factor` remains the platform protected source for `deviceSecret`; it is not `Kmaster` and is not the mnemonic.

In the V2 package schema, `encryptedPayload` is a historical field name. It now carries a base64 encoded factor payload; that payload is not a mnemonic-encrypted vault and does not contain plaintext `Kmaster`.
