# Commercial Licensing

KeyLessPass is source-available for evaluation and security review, but it is not open-source software.

The public repository is intended for:

- Personal learning
- Security review
- Non-commercial testing
- Internal proof-of-concept evaluation
- Technical due diligence by potential partners

The following uses require a separate written commercial license:

- Enterprise production deployment
- Commercial use
- Redistribution
- OEM or white-label integration
- Channel or reseller sales
- Managed service use
- Security service provider bundling
- Consulting service delivery using KeyLessPass
- Processing real customer or enterprise production credentials
- Integration into paid products, appliances, platforms, or service packages

## Supported Commercial Models

KeyLessPass may support the following cooperation models:

1. Enterprise license
2. Offline/internal deployment license
3. OEM or white-label license
4. Channel/reseller cooperation
5. Security service provider integration
6. Custom enterprise support
7. Joint proof-of-concept projects

## Device Batch Authorization

For enterprise and commercial operation, KeyLessPass should use signed
organization licenses and per-device grants. The authorization layer is separate
from the password derivation and 2-of-3 recovery model: it must not store,
upload, or bind to mnemonic phrases, `Kmaster`, `deviceSecret`, `usbSecret`, CDR
plaintext passwords, or derived service passwords.

See [docs/commercial/device-batch-authorization.md](docs/commercial/device-batch-authorization.md)
for the proposed offline bulk authorization, online activation, MDM deployment,
renewal, and revocation design.

## Proof-of-Concept Rules

For PoC or evaluation use:

- Do not use real production passwords or enterprise secrets.
- Use test accounts and non-production environments where possible.
- Do not deploy KeyLessPass as a production credential management system without authorization.
- Do not redistribute the software to third parties.
- Do not remove copyright, license, or attribution information.

## Contact

For commercial licensing, OEM, reseller, or partnership inquiries, please contact revanton@icloud.com.
