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

Application downloads at `/download` require no login. In the recommended mode, endpoints locate the service, register, and retrieve approved grants automatically. The deployment token is reserved for customer IT batch exchange and troubleshooting.

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

The first run creates a deployment token and customer-site key, prints the site key ID/public key, and pauses. The vendor issues a bootstrap entitlement delegated to those values. Install the returned vendor root public key in `.env` and the signed file at `license/customer-entitlement.json`, then run the script again.

```text
/download   public artifact list
/           token-authenticated batch exchange and maintenance
/healthz    public health check
```

The script exposes TCP 8787 and UDP 8788 to the intranet and generates `KEYLESSPASS_PUBLIC_BASE_URL`. Override it for multiple NICs, NAT, fixed DNS, or a reverse proxy. Never expose the ports to the Internet.

Clicking any installer on `/download` also downloads a dynamically generated `keylesspass-client-config.json`. The client reads the newest matching file in Downloads; UDP is only a fallback when config is absent. The config only locates the server and cannot replace the vendor-root Ed25519 authorization chain.

## Commercial client build

Embed the vendor root, not the customer-site key:

```bash
KEYLESSPASS_LICENSE_KEY_ID='keylesspass-vendor-root-2026' \
KEYLESSPASS_LICENSE_PUBLIC_KEY_B64='<vendor root public key>' \
CODESIGN_IDENTITY='Developer ID Application: Your Company (TEAMID)' \
tools/commercial/build_commercial_release.sh macos
```

Use `linux` with `KEYLESSPASS_LINUX_GPG_KEY_ID` on Linux. On Windows, set the Authenticode certificate thumbprint required by the packaging script. Copy final packages into `admin_backend/downloads/`, then run `cargo run -- issue-release-manifest > downloads/release-manifest.json` with the offline vendor root. The page lists only files covered by the signed manifest whose size and SHA-256 still match.

## Recommended pure-intranet batch approval

1. Users click an installer on `/download`; the page delivers both the installer and server config. They install and start once without entering authorization data.
2. Each client reads the newest downloaded or managed config. It discovers UDP port 8788 only when config is missing.
3. `/api/automatic/activate` verifies proof of the device private key, records an unapproved device, and returns a pending state.
4. Once approved, the server issues a 24-hour lease by default. The client renews every 30 minutes and retains only the last signed lease plus the default one-day grace when the server is unavailable.
5. Customer IT opens `/` with the deployment token, clicks **Export batch request**, and sends the JSON to the vendor.
6. The vendor checks the contract and runs on its offline workstation:

```bash
export KEYLESSPASS_VENDOR_SIGNING_KEY_B64='<vendor root seed>'
export KEYLESSPASS_VENDOR_KEY_ID='keylesspass-vendor-root-2026'
export KEYLESSPASS_DEVICE_BATCH_REQUEST_FILE='<customer batch request JSON>'
export KEYLESSPASS_CUSTOMER_VALID_UNTIL='2027-12-31T23:59:59Z'
cargo run -- issue-customer-entitlement > customer-entitlement.json
```

7. Customer IT imports that file on the same page. The service restarts.
8. Each polling client retrieves and imports its own signed grant. Revocation stops renewal and takes effect no later than the remaining lease plus grace.

The batch embeds the previous vendor-signed entitlement. The signer verifies it and inherits its customer, site key, contract limit, dates, and features, so an edited batch quota is rejected. Renewal or expansion requires explicit vendor-side overrides. If the collected list is larger than the signed limit, the vendor must set the reviewed `KEYLESSPASS_AUTHORIZED_DEVICE_KEY_IDS` subset.

## Automatic companion config and managed paths

Deploy the generated configuration to:

| Platform | Path |
| --- | --- |
| macOS | `/Library/Application Support/KeyLessPass/keylesspass-client-config.json` or the equivalent user path |
| Windows | `%PROGRAMDATA%\KeyLessPass\keylesspass-client-config.json` or the `%APPDATA%` equivalent |
| Linux | `/etc/keylesspass/client-config.json` or `~/.config/keylesspass/client-config.json` |
| Any | beside the executable/current directory, or set `KEYLESSPASS_CLIENT_CONFIG` |

TCP 8787 must remain reachable. If policy blocks TCP as well, no network path exists; allow it or deploy a signed offline bundle through MDM.

## Manual fallback: initial device approval

1. The user opens Security -> Commercial authorization and copies the schema-v2 device request.
2. The customer admin creates an organization and imports the request.
3. The admin exports device CSV and sends requested `deviceKeyId` values to the vendor.
4. The vendor verifies the purchase, adds approved lowercase SHA-256 IDs to `KEYLESSPASS_AUTHORIZED_DEVICE_KEY_IDS`, increments `KEYLESSPASS_ENTITLEMENT_SERIAL`, and runs `cargo run -- issue-customer-entitlement`.
5. The customer replaces the entitlement and restarts the service.
6. The admin can now issue an offline bundle or allow HTTPS online activation.

Device requests contain a public key and proof of possession but no password secrets.

## Manual fallback: online activation

The user enters the HTTPS server URL, organization activation code, and seat label. The server verifies device proof, vendor approval, organization policy, revocation, and availability, then allocates the seat and stores the grant in one SQLite `BEGIN IMMEDIATE` transaction.

Loopback HTTP is accepted for testing. An unapproved device, invalid code, exhausted pool, or non-loopback HTTP request is rejected.

## Manual fallback: offline and managed activation

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
