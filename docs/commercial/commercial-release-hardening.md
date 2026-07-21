# Commercial Release Hardening

This document defines the commercial distribution controls for KeyLessPass.
The goal is not to pretend that a local desktop binary is uncrackable. A
determined attacker can patch client-side checks. The mature defense is layered:
signed per-device grants, commercial builds with compile-time enforcement,
signed installers, customer-identifiable license bundles, revocation, and
support/update access tied to valid authorization.

## Principles

- Never put a licensing private key or shared activation secret in the client.
- Commercial clients embed only public verification keys.
- Commercial clients must be built with `KEYLESSPASS_REQUIRE_LICENSE=1`.
- License bundles are signed by the issuer and bound to
  `commercialDeviceId + deviceFingerprint`.
- A copied license bundle does not authorize another machine unless it contains
  a grant for that machine.
- License metadata is separate from password security material and must never
  contain mnemonic phrases, `Kmaster`, `deviceSecret`, `usbSecret`, service
  passwords, derived passwords, CDR secrets, or wrapper keys.
- Ordinary USB media remains copyable; licensing must not describe it as
  uncopyable hardware.

## Release Pipeline

1. Deploy `admin_backend` on an internal host.
2. Copy the admin page `publicKeyB64` for the signing key.
3. Build commercial clients with compile-time enforcement:

   ```bash
   KEYLESSPASS_LICENSE_PUBLIC_KEY_B64='<public key from admin_backend>' \
   tools/commercial/build_commercial_release.sh macos
   ```

4. Sign the generated app/installer:

   - macOS: Developer ID Application signing, notarization, and stapling.
   - Windows: Authenticode signing of binaries and installer.
   - Linux: signed repository metadata or signed release checksum manifest.

5. Distribute through customer-specific channels. Keep release artifacts,
   `licenseId`, `organizationId`, signing `keyId`, and contract records linked.
6. Require a valid device grant for commercial support and updates.

## Anti-Abuse Model

This design raises the cost and reduces the value of unauthorized redistribution:

- A shared app binary without a valid grant stays unlicensed when enforcement is
  compiled in.
- A copied license bundle is tied to the original commercial device identity and
  fingerprint.
- Customer bundles contain organization/license/grant identifiers, giving every
  distributed activation package an audit trail.
- Revoked grants are carried in later offline bundles.
- Updates and support can require proof of a valid license status.
- Signed installers make tampered builds distinguishable from official builds.

This does not stop someone from patching a binary and distributing a modified
client. Countermeasures for that class are operational and legal as much as
technical: watermark commercial releases, publish official checksums, require
signed update channels, keep customer-specific license records, and enforce the
commercial license agreement.

## Build-Time Controls

The Rust core now reads the trusted license public key from compile-time
environment variables:

```text
KEYLESSPASS_LICENSE_KEY_ID
KEYLESSPASS_LICENSE_PUBLIC_KEY_B64
KEYLESSPASS_REQUIRE_LICENSE=1
```

Evaluation/source builds keep a non-blocking default so the project remains
reviewable. Commercial release automation must use
`tools/commercial/build_commercial_release.sh` or an equivalent CI job that sets
the same variables.

Runtime environment variables may enable stricter checks for testing, but a
commercial build must not rely on a runtime switch for enforcement.

## Recommended Future Additions

- Customer-specific signed update feed.
- Transparency log of official release checksums.
- MDM managed license bundle path.
- CSV/MDM bulk import in `admin_backend`.
- Append-only admin audit log export.
- Key rotation policy with overlapping public-key trust windows.
