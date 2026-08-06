# Changelog

## Unreleased

- Added selectable v3 authenticated Shamir 2-of-3 Root-Key recovery with `vsss-rs` 5.4.0, version-bound share envelopes, KCV validation, a checksum-protected recovery phrase, manifest-last commits, share-set refresh, factor replacement, and empty-vault Root-Key rotation.
- Added verified v2 pairwise-wrapper to v3 migration with dry-run, optional recoverable archive, all-path validation, and a redacted audit record.
- Added CDR schema v3 with RFC 8785 canonical JSON, explicit vault/service/account and generation identifiers, parent hashes, operation IDs, rotation state, and replica metadata.
- Replaced the password encoder with deterministic rejection sampling, unbiased shuffling, bounded retries, explicit policy contradictions, class min/max counts, edge restrictions, and repeat/sequence controls.
- Added persistent pending-confirm-reconcile rotation states, replica conflict classification, and a compare-and-set freshness-service interface.
- Added fixed recovery/CDR vectors, property-style state/recovery/encoder tests, a macOS full-scale experiment through 100,000 CDRs, and cross-platform CI configuration.
- Repositioned the research artifact as a legacy service-password lifecycle protocol; the v3 Flutter enrollment UX, non-empty-vault Root-Key rotation, production freshness transport, and target adapters remain explicit limitations.

## 0.1.0

- Productized Flutter Desktop shell with dashboard, records, add, derive, rotation, recovery, USB, security, settings, and about sections.
- Added English and Simplified Chinese ARB resources.
- Added i18n resource completeness tests.
- Improved macOS USB candidate detection for writable removable volumes without an existing factor package.
- Added notes metadata for credential records without changing derivation.
- Added cancel rotation flow.
- Added root privacy, security, release, contributing, and development documents.
