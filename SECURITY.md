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
