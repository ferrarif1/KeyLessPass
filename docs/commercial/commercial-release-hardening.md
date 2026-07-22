# Commercial Release Hardening

This document defines the commercial distribution controls for KeyLessPass.
The goal is not to pretend that a local desktop binary is uncrackable. A
determined attacker can patch client-side checks. The mature defense is layered:
signed per-device grants, commercial builds with compile-time enforcement,
signed installers, customer-identifiable license bundles, revocation, and
support/update access tied to valid authorization.

## Principles

- Never put a licensing private key or shared activation secret in the client.
- Commercial clients embed only vendor-root public verification keys and never directly trust a customer-site key.
- Commercial clients must be built with `KEYLESSPASS_REQUIRE_LICENSE=1`.
- License bundles verify both the vendor entitlement and site signature, then bind to a device identity key, `deviceKeyId`, and fingerprint.
- A copied license bundle does not authorize another machine unless it contains
  a grant for that machine.
- License metadata is separate from password security material and must never
  contain mnemonic phrases, `Kmaster`, `deviceSecret`, `usbSecret`, service
  passwords, derived passwords, CDR secrets, or wrapper keys.
- Ordinary USB media remains copyable; licensing must not describe it as
  uncopyable hardware.

## Release Pipeline

1. The vendor issues an offline customer entitlement that delegates the site key and lists approved `deviceKeyId` values.
2. Install that entitlement and deploy `admin_backend`.
3. Build commercial clients with the vendor root and compile-time enforcement:

   ```bash
   KEYLESSPASS_LICENSE_KEY_ID='keylesspass-vendor-root-2026' \
   KEYLESSPASS_LICENSE_PUBLIC_KEY_B64='<vendor root public key>' \
   CODESIGN_IDENTITY='Developer ID Application: Your Company (TEAMID)' \
   tools/commercial/build_commercial_release.sh macos
   ```

4. Sign the generated app/installer:

   - macOS: Developer ID Application signing, notarization, and stapling.
   - Windows: Authenticode signing of binaries and installer.
   - Linux: signed repository metadata or signed release checksum manifest.

Commercial macOS and Windows packaging rejects missing platform certificates by default. Linux requires `KEYLESSPASS_LINUX_GPG_KEY_ID` and emits `SHA256SUMS.asc`. `KEYLESSPASS_ALLOW_UNSIGNED=1` is for local testing only and must not be published.

5. Put installers in `admin_backend/downloads/`, set `KEYLESSPASS_RELEASE_DIRECTORY` on the offline vendor workstation, and run `cargo run -- issue-release-manifest > downloads/release-manifest.json`. The backend refuses to list files absent from this vendor-signed manifest or whose size/hash changed.
6. Distribute through customer-specific channels. Keep release artifacts,
   `licenseId`, `organizationId`, signing `keyId`, and contract records linked.
7. Require a valid device grant for commercial support and updates.

## Anti-Abuse Model

This design raises the cost and reduces the value of unauthorized redistribution:

- A shared app binary without a valid grant stays unlicensed when enforcement is
  compiled in.
- A copied license bundle is tied to the original commercial device identity and
  fingerprint.
- A modified customer backend cannot authorize a device absent from the vendor-signed allowlist.
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
KEYLESSPASS_BUILD_CHANNEL=commercial
KEYLESSPASS_APP_MAJOR_VERSION=1
KEYLESSPASS_MANAGED_LICENSE_FILE=<managed bundle path>
```

These key values identify the vendor root. During vendor-root rotation, add the overlapping public-key map:

```text
KEYLESSPASS_LICENSE_TRUSTED_KEYS_JSON={"old-key-id":"old-public-key","new-key-id":"new-public-key"}
```

Evaluation/source builds keep a non-blocking default so the project remains
reviewable. Commercial release automation must use
`tools/commercial/build_commercial_release.sh` or an equivalent CI job that sets
the same variables. The commercial entry point requires the vendor-root key ID and public key explicitly; there is no production key-ID default.

Runtime environment variables may enable stricter checks for testing, but a
commercial build must not rely on a runtime switch for enforcement.

The commercial build script assigns managed bundle defaults under
`/Library/Application Support/KeyLessPass`, `/etc/keylesspass`, or
`C:\ProgramData\KeyLessPass`. A valid managed bundle is verified and refreshed
into the private local license store whenever status is evaluated.

Customer-specific update feeds and public release transparency logs remain
optional distribution enhancements; they are not required for authorization
correctness.
