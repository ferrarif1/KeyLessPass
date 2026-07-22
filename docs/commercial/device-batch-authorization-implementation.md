# Device Batch Authorization Implementation Notes

This branch implements the first commercial authorization layer for
KeyLessPass. The current stable product state is preserved in the main worktree
and tagged as `baseline-before-device-auth-20260721`.

## Why This Design

KeyLessPass is a local desktop password derivation product. A licensing system
for it must solve a commercial problem without becoming part of the password
security boundary.

The correct first implementation is therefore:

- signed organization licenses and signed/imported device grants;
- local verification with a public key embedded in the client;
- per-installation commercial device identity;
- per-device fingerprint binding;
- status display and import/export workflows in the desktop UI;
- a narrow enforcement guard that can be enabled per commercial build.

This prevents casual unlicensed use such as copying a DMG or sharing one
enterprise bundle with unrelated machines, while keeping emergency credential
availability and the 2-of-3 recovery model intact.

## Threat Model

Covered by this implementation:

- A user copies the app binary to another machine without a signed grant.
- A user copies a license bundle that lacks a grant for this device.
- A customer exceeds purchased seats in offline deployment.
- Support needs to identify the organization/license/grant associated with an
  installation.
- A license expires and the app must clearly report renewal/grace/expired
  status.

Not fully covered:

- A determined attacker patches a local desktop binary.
- A user clones an entire machine image including all local application state.
- A source-available fork removes local authorization checks.

Those risks are handled commercially through contracts, watermarking,
support/update access, issued grant records, and auditability, not by pretending
local DRM is absolute.

## Security Boundary

The authorization layer must never store, send, derive from, or encrypt with:

- mnemonic phrases;
- `Kmaster`;
- `deviceSecret`;
- `usbSecret`;
- service passwords;
- derived service passwords;
- pairwise wrapper keys.

License failure must never delete or rewrite:

- local factor packages;
- USB factor packages;
- CDRs;
- recovery wrappers;
- CDR backups.

The licensing `commercialDeviceId` is separate from the KeyLessPass computer
factor. The device fingerprint may include non-secret local platform identity
material to bind a grant to one installation, but it must not include
`deviceSecret`.

## Delivered Scope

Rust core:

- `domain::license`
- `crypto::signing`
- `storage::license_store`
- `service::license`
- FFI operations:
  - `getLicenseStatus`
  - `exportDeviceAuthorizationRequest`
  - `importLicenseBundle`
  - `clearLicense`

Flutter:

- license status model;
- CoreApi methods;
- Settings license panel;
- copy device authorization request;
- paste/import enterprise license bundle;
- clear local license grant.

Admin backend:

- standalone `admin_backend` Rust service;
- token-protected intranet browser UI;
- SQLite organization, device, bundle, and grant metadata storage;
- import of desktop device authorization requests;
- Ed25519 signing of client-compatible license bundle envelopes;
- grant revocation records included in newly issued bundles;
- Docker Compose deployment with `scripts/intranet_deploy.sh`.
- activation-code online activation;
- cross-bundle seat enforcement;
- admin, operator, and auditor roles;
- append-only audit log and CSV export;
- CSV device-request import and device export.

Commercial release hardening:

- commercial builds can inject the trusted license public key with
  `KEYLESSPASS_LICENSE_PUBLIC_KEY_B64`;
- commercial builds must set `KEYLESSPASS_REQUIRE_LICENSE=1`;
- `tools/commercial/build_commercial_release.sh` checks those inputs before
  calling the platform packaging script.
- commercial release channel entitlement and application major-version checks;
- overlapping trusted public keys for signing-key rotation;
- managed license bundle auto-import paths for macOS, Windows, and Linux.

Commercial enforcement:

- Default source-available/evaluation builds do not block local password
  workflows.
- Commercial builds can set `KEYLESSPASS_REQUIRE_LICENSE=1` at Rust compile
  time.
- When enabled, FFI entry points for enrollment, add/update/derive/rotation,
  recovery, mnemonic reset, and CDR USB sync require a valid `desktop-client`
  device grant.
- License status, device request export, bundle import, and license clearing
  remain available so an unlicensed device can be authorized.

Tests:

- unlicensed state is explicit;
- exported device request contains no password factor secrets;
- valid signed bundle authorizes the intended device;
- copied bundle does not authorize another device;
- tampered payload/signature fails;
- expired license reports expired or grace status without mutating factor data.

## Operational Requirements

The authorization implementation is complete. Production rollout still requires
organization-owned infrastructure and credentials: TLS termination and rate
limiting for online activation, protected signing-key storage, platform code
signing/notarization, backups of the authorization database, and a documented
key-rotation ceremony. These are deployment controls rather than missing product
code.
