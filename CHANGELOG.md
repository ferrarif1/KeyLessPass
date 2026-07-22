# Changelog

## Unreleased

- Completed commercial device authorization with signed offline bundles and HTTPS online activation.
- Added cross-bundle seat enforcement, release-channel and major-version checks, revocation, expiry, and grace handling.
- Added managed license auto-import and overlapping trusted public keys for signing-key rotation.
- Added administrator, operator, and auditor roles, append-only audit export, and device CSV workflows.
- Added commercial build validation and desktop online-activation UI in English and Simplified Chinese.

## 0.1.0

- Productized Flutter Desktop shell with dashboard, records, add, derive, rotation, recovery, USB, security, settings, and about sections.
- Added English and Simplified Chinese ARB resources.
- Added i18n resource completeness tests.
- Improved macOS USB candidate detection for writable removable volumes without an existing factor package.
- Added notes metadata for credential records without changing derivation.
- Added cancel rotation flow.
- Added root privacy, security, release, contributing, and development documents.
