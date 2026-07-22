# Device Authorization Implementation and Usage Guide

Chinese: [device-batch-authorization-implementation.zh-CN.md](device-batch-authorization-implementation.zh-CN.md)

This is the primary commercial operations guide. It separates vendor, customer-administrator, and endpoint responsibilities.

See the [commercial authorization security audit](authorization-security-audit.md) for findings and residual boundaries.

## Components

```text
admin_backend/       public downloads and authenticated intranet licensing admin
flutter_app/         macOS, Windows, and Linux desktop client
rust_core/           device identity, license verification, enforcement, password core
tools/commercial/    enforced commercial build entry point
packaging/           platform packages and signing scripts
```

Application downloads at `/download` require no login. Organization, device, issuance, and revocation operations at `/` require an Admin token.

## Trust and seat protection

The client first verifies `customer-entitlement.json` with its embedded vendor root, obtains the delegated customer-site public key, and then verifies the site-signed device bundle. The vendor entitlement constrains customer, dates, features, versions, limits, and an approved `deviceKeyId` allowlist.

This allowlist is essential: a `maxSeats` field alone cannot stop a customer-controlled server from issuing many forked one-device bundles. A site server cannot authorize a new key absent from the vendor signature.

Each endpoint also creates a device identity private key. Grants bind its public key and key ID plus a protected device fingerprint. Windows protects software state with DPAPI and macOS uses Keychain protection.

## Vendor setup

Generate and retain the root on a vendor-controlled offline workstation:

```bash
cd admin_backend
cargo run -- generate-key
```

Never give the root seed to a customer or include it in a client. Use the root key ID and public key for all commercial client builds.

## Customer-site deployment

```bash
cd admin_backend
./scripts/intranet_deploy.sh
```

The first run creates an Admin token and customer-site key, prints the site key ID/public key, and pauses. The vendor issues a bootstrap entitlement delegated to those values. Install the returned vendor root public key in `.env` and the signed file at `license/customer-entitlement.json`, then run the script again.

```text
/download   public artifact list
/           token-authenticated admin UI
/healthz    public health check
```

Place non-loopback activation behind HTTPS and restrict administrative ingress.

## Commercial client build

Embed the vendor root, not the customer-site key:

```bash
KEYLESSPASS_LICENSE_KEY_ID='keylesspass-vendor-root-2026' \
KEYLESSPASS_LICENSE_PUBLIC_KEY_B64='<vendor root public key>' \
CODESIGN_IDENTITY='Developer ID Application: Your Company (TEAMID)' \
tools/commercial/build_commercial_release.sh macos
```

Use `linux` with `KEYLESSPASS_LINUX_GPG_KEY_ID` on Linux. On Windows, set the Authenticode certificate thumbprint required by the packaging script. Copy final packages into `admin_backend/downloads/`; the public page computes and displays SHA-256 values.

## Initial device approval

1. The user opens Security -> Commercial authorization and copies the schema-v2 device request.
2. The customer admin creates an organization and imports the request.
3. The admin exports device CSV and sends requested `deviceKeyId` values to the vendor.
4. The vendor verifies the purchase, adds approved lowercase SHA-256 IDs to `KEYLESSPASS_AUTHORIZED_DEVICE_KEY_IDS`, increments `KEYLESSPASS_ENTITLEMENT_SERIAL`, and runs `cargo run -- issue-customer-entitlement`.
5. The customer replaces the entitlement and restarts the service.
6. The admin can now issue an offline bundle or allow HTTPS online activation.

Device requests contain a public key and proof of possession but no password secrets.

## Online activation

The user enters the HTTPS server URL, organization activation code, and seat label. The server verifies device proof, vendor approval, organization policy, revocation, and availability, then allocates the seat and stores the grant in one SQLite `BEGIN IMMEDIATE` transaction.

Loopback HTTP is accepted for testing. An unapproved device, invalid code, exhausted pool, or non-loopback HTTP request is rejected.

## Offline and managed activation

After vendor approval, select devices in the admin UI, issue the bundle, and import it from the client authorization panel. Managed deployment paths are:

| Platform | Path |
| --- | --- |
| macOS | `/Library/Application Support/KeyLessPass/license-bundle.json` |
| Windows | `C:\ProgramData\KeyLessPass\license-bundle.json` |
| Linux | `/etc/keylesspass/license-bundle.json` |

## Lifecycle

- `seat_allocations` provides transactional active/expired/revoked states.
- Revocation disables all active grants for the device and prevents reactivation with the revoked identity.
- Offline revocation takes effect only when a new bundle arrives or the old grant expires.
- Rehosting requires a new device key and a higher-serial vendor entitlement.
- Clearing the local license leaves password data and anti-rollback state intact.
- Older entitlement serials and older bundle issue times are rejected.

## Security boundary

Licensing never receives mnemonic phrases, `Kmaster`, factor secrets, service passwords, derived passwords, CDR secrets, or recovery keys. License failure does not delete password data.

Local software is not unbreakable DRM. A machine owner can attempt to patch the client, and pure offline software cannot reliably distinguish a complete VM snapshot clone. Strict floating concurrency against a hostile customer requires vendor-online leases, a TPM/HSM-backed server, or a commercial hardware licensing system such as Sentinel LDK or CodeMeter.
