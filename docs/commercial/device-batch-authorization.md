# KeyLessPass Device Batch Authorization Plan

This document defines a commercial device authorization scheme for KeyLessPass.
It is a product and licensing layer. It must not weaken the local password
derivation or 2-of-3 recovery design.

## Goals

- Support enterprise and channel sales with account-level licenses, seat counts,
  bulk device onboarding, renewals, and revocation.
- Support offline/internal deployments where production machines cannot reach a
  public licensing service.
- Keep KeyLessPass privacy-preserving: no mnemonic, `Kmaster`, `deviceSecret`,
  `usbSecret`, CDR plaintext password, or derived service password is sent to a
  licensing server.
- Keep authorization independent from password recovery. License failure must
  not modify factor packages, CDR data, USB packages, or recovery wrappers.

## Non-Goals

- Do not treat licensing as cryptographic protection for user passwords.
- Do not promise unbreakable DRM. A local-only desktop client can be patched by
  a determined attacker; commercial protection comes from signed licensing,
  update/support access, contracts, auditability, and enterprise workflows.
- Do not bind licenses to `Kmaster`, mnemonic phrases, `deviceSecret`, or USB
  factor material.

## Authorization Objects

### Organization License

An organization license describes the commercial entitlement:

- `schemaVersion`
- `licenseId`
- `organizationId`
- `organizationName`
- `plan`: `evaluation`, `enterprise`, `offline-enterprise`, `oem`,
  `reseller`, or `managed-service`
- `maxSeats`
- `validFrom`
- `validUntil`
- `features`: for example `desktop-client`, `offline-activation`,
  `batch-device-import`, `enterprise-support`, `white-label`
- `offlineGraceDays`
- `allowedMajorVersions`
- `issuedAt`
- `issuer`

The license is distributed as a signed envelope. The client embeds only the
public verification key. The private signing key must stay on the license
issuer side.

### Device Request

Each client can export a device request file for batch authorization:

- `schemaVersion`
- `requestId`
- `organizationId` or activation code
- `commercialDeviceId`: random per-installation UUID generated for licensing
- `deviceFingerprint`: privacy-preserving hash of stable non-secret device
  attributes and the commercial device id
- `platform`
- `appVersion`
- `buildChannel`
- optional admin-visible labels such as hostname, asset tag, department, and
  operator
- `createdAt`

`commercialDeviceId` is not the KeyLessPass `deviceId` used for the computer
factor, and it is not `deviceSecret`. It exists only for licensing and seat
management.

### Device Grant

A device grant authorizes one installation/seat:

- `schemaVersion`
- `grantId`
- `licenseId`
- `organizationId`
- `commercialDeviceId`
- `deviceFingerprint`
- `seatLabel`
- `validFrom`
- `validUntil`
- `features`
- `offlineGraceDays`
- `issuedAt`
- `issuer`

The device grant is signed by the licensing issuer and verified locally by the
client.

### License Bundle

A batch license bundle contains one organization license plus multiple device
grants:

- `schemaVersion`
- `bundleId`
- `organizationLicense`
- `deviceGrants`
- optional signed revocation list
- `issuedAt`
- `signature`

The bundle format should be a canonical JSON payload inside a signed envelope,
for example:

```json
{
  "schemaVersion": 1,
  "type": "keylesspass-license-bundle",
  "payload": "base64url(canonical-json)",
  "signatureAlgorithm": "Ed25519",
  "keyId": "keylesspass-license-2026-q3",
  "signature": "base64url(signature)"
}
```

## Cryptography

- Use Ed25519 or another modern asymmetric signature algorithm for license and
  grant signatures.
- Do not use symmetric license secrets in the client.
- The client contains public verification keys and a `keyId` allowlist.
- Rotate signing keys by adding a new public key before issuing new licenses.
- Canonicalize JSON before signing to avoid parser-dependent signature bugs.
- Treat package signatures as commercial integrity checks only; they must not
  replace AEAD tags or 2-of-3 recovery wrappers.

## Batch Authorization Flows

### 1. Offline Bulk Authorization

This is the preferred flow for regulated enterprises and internal networks.

1. Admin deploys KeyLessPass to target machines.
2. Each machine exports a `.klp-device-request.json` file or MDM collects those
   request files.
3. Admin imports all requests into the commercial portal or an internal
   `keylesspass-admin` tool.
4. The issuer verifies the purchased seat count and signs a
   `.klp-license-bundle`.
5. Admin distributes the bundle through MDM, file share, USB, or manual import.
6. Each client verifies the bundle signature locally and activates only the
   grant matching its `commercialDeviceId` and `deviceFingerprint`.

### 2. Online Activation

This is the simpler SaaS-style flow for small teams.

1. Admin purchases seats and receives an organization activation code.
2. User enters the activation code in KeyLessPass.
3. Client sends a device request to the licensing API over TLS.
4. Licensing service returns a signed device grant.
5. Client stores the signed grant locally and verifies it on every launch.

The API must not receive mnemonic phrases, factor packages, CDR secrets,
`Kmaster`, `deviceSecret`, `usbSecret`, or derived passwords.

### 3. MDM / Managed Deployment

For enterprise desktop management:

- Admin pushes the app package.
- Admin pushes a license bundle to a managed path.
- Client reads the managed bundle at startup and stores a verified copy in the
  local license store.
- Optional managed policy controls update channel, telemetry disabled state,
  license refresh URL, and allowed feature set.

## Local Storage

Add a dedicated license store separate from factor storage:

- macOS: Application Support plus optional Keychain item for
  `commercialDeviceId`
- Windows: `%APPDATA%` or ProgramData plus DPAPI-protected local id
- Linux: XDG config/data directory plus Secret Service when available

The license store may contain:

- signed organization license
- signed device grant
- signed revocation list
- commercial device id
- last successful validation timestamp
- non-secret admin labels

It must not contain:

- plaintext `Kmaster`
- mnemonic phrases
- `deviceSecret`
- `usbSecret`
- service passwords
- CDR-derived passwords

## Enforcement Policy

Recommended commercial policy:

- Evaluation builds allow non-commercial testing and show evaluation status.
- Commercial builds require a valid signed device grant for production use.
- Expired licenses should enter a grace period rather than immediately blocking
  emergency access to existing derived passwords.
- After grace expiration, block new enrollment, adding records, rotation,
  recovery setup, and USB rebuild. Existing emergency derivation behavior should
  be a business decision documented in the commercial agreement.
- Never delete or mutate local factor packages, USB packages, CDRs, or recovery
  wrappers because of license status.

This avoids turning licensing failure into a credential availability incident.

## Revocation and Renewal

- Online deployments fetch a signed revocation list during license refresh.
- Offline deployments receive revocation lists inside renewed license bundles.
- Device grants should be short enough for commercial control, but long enough
  for operational continuity, for example 90-365 days depending on contract.
- Renewal creates a new signed organization license and fresh device grants.
- Device replacement consumes a new seat unless an admin revokes or releases the
  old grant.

## Suggested Rust Core Modules

Add licensing as an outer service layer:

- `rust_core/src/domain/license.rs`
  - organization license, device request, device grant, license bundle, license
    status models
- `rust_core/src/crypto/signing.rs`
  - Ed25519 signature verification and canonical payload helpers
- `rust_core/src/storage/license_store.rs`
  - local signed license and device grant storage
- `rust_core/src/service/license.rs`
  - export device request, import license bundle, online activation, status
    validation
- `rust_core/src/ffi.rs`
  - FFI operations exposed to Flutter

Recommended FFI operations:

- `getLicenseStatus`
- `exportDeviceAuthorizationRequest`
- `importLicenseBundle`
- `activateLicenseOnline`
- `clearLicense`
- `listLicensedFeatures`

The existing password derivation and recovery services should call a small
`require_feature(feature)` guard at entry points where commercial enforcement is
needed. The guard must never read or write factor secrets.

## Suggested Flutter UI

Add an `Authorization` or `License` page under Settings:

- current license status
- organization name
- plan
- seat/device label
- expiry date and grace status
- export device request
- import license bundle
- online activation code
- copy diagnostics for support

For batch operation, the UI should clearly support:

- "Export this device request"
- "Import enterprise license bundle"
- "This device is authorized"
- "License expires on ..."
- "Offline grace period active"

## Commercial Operations

Minimum backend/admin capabilities:

- customer account and contract record
- seat count and plan management
- license issuance
- device request import by JSON or CSV
- batch grant generation
- grant revocation/release
- renewal bundle generation
- support diagnostics lookup by `licenseId`, `organizationId`, or `grantId`

For the first commercial version, an internal CLI can replace a full web portal:

```text
keylesspass-admin issue-org-license --org acme --seats 200 --until 2027-12-31
keylesspass-admin import-device-requests ./requests/
keylesspass-admin issue-bundle --license LIC-... --out acme.klp-license-bundle
keylesspass-admin revoke-device --grant GRANT-...
```

## Rollout Plan

1. Documentation and commercial terms.
2. Local signed license verification in Rust core.
3. Flutter license page for status, request export, and bundle import.
4. Offline admin CLI for batch device grants.
5. Online activation API.
6. MDM managed path support.
7. Renewal and revocation workflow.

## Acceptance Criteria

- A single signed license bundle can authorize hundreds or thousands of devices.
- A copied license bundle does not authorize a different device unless the
  bundle includes a grant for that device.
- A copied app binary without a valid grant reports unlicensed status.
- License validation never requires or exposes mnemonic phrases, `Kmaster`,
  `deviceSecret`, `usbSecret`, or service passwords.
- License failure never deletes or rewrites factor packages, USB packages, CDRs,
  or recovery wrappers.
- Offline deployments can renew by importing a new signed bundle.
- Online deployments can activate with an activation code and receive a signed
  device grant.

