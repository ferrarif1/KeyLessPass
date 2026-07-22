# KeyLessPass Admin Backend

Chinese: [README.zh-CN.md](README.zh-CN.md)

This intranet service provides public application downloads and authenticated device, seat, activation, offline-bundle, and revocation administration.

- `/download`, `/api/downloads`, and `/downloads/*` are public;
- the admin UI and administrative APIs require an Admin token;
- a customer-site key may sign only within a vendor-signed entitlement;
- commercial clients embed only the vendor root public key;
- a device absent from the vendor-approved `deviceKeyId` allowlist cannot be activated by a modified customer backend.

The service handles licensing metadata only. It must never receive mnemonic phrases, `Kmaster`, factor secrets, CDR secrets, service passwords, or derived passwords.

## Trust chain

```text
offline vendor root private key (vendor only)
  -> signs customer-entitlement.json
       -> customer limits, dates, features, and versions
       -> delegated customer-site public key
       -> vendor-approved deviceKeyId allowlist
          -> constrains device bundles signed at the customer site
```

A `maxSeats` field alone cannot stop a modified customer server from issuing forked one-device bundles. Strict mode therefore also requires every granted device key to be approved by the vendor signature.

## Customer-site deployment

Install Docker, Docker Compose, and OpenSSL, then run:

```bash
cd admin_backend
./scripts/intranet_deploy.sh
```

The first run creates a random Admin token, a customer-site Ed25519 key, a unique site key ID, and a mode-`600` `.env`. It intentionally stops until the vendor supplies:

1. the vendor root key ID and public key;
2. a signed `license/customer-entitlement.json` delegated to the printed site public key.

Install both and run the script again. The resulting endpoints are:

```text
Public downloads: http://127.0.0.1:8787/download
Admin UI:        http://127.0.0.1:8787/
Health check:    http://127.0.0.1:8787/healthz
```

Use an intranet address for LAN clients. Except for loopback testing, put online activation behind HTTPS and restrict administrative ingress.

## Vendor entitlement issuance

Run this only on a vendor-controlled workstation. Never deliver the root private key to a customer:

```bash
cd admin_backend
export KEYLESSPASS_VENDOR_SIGNING_KEY_B64='<offline vendor root seed>'
export KEYLESSPASS_VENDOR_KEY_ID='keylesspass-vendor-root-2026'
export KEYLESSPASS_CUSTOMER_ID='customer-001'
export KEYLESSPASS_CUSTOMER_NAME='Example Customer'
export KEYLESSPASS_SITE_KEY_ID='<site key ID printed by deployment>'
export KEYLESSPASS_SITE_PUBLIC_KEY_B64='<site public key printed by deployment>'
export KEYLESSPASS_MAX_REGISTERED_DEVICES='50'
export KEYLESSPASS_MAX_CONCURRENT_DEVICES='50'
export KEYLESSPASS_MAX_OFFLINE_BORROWED='0'
export KEYLESSPASS_MAX_OFFLINE_GRACE_DAYS='14'
export KEYLESSPASS_CUSTOMER_FEATURES='desktop-client,channel:commercial'
export KEYLESSPASS_ALLOWED_MAJOR_VERSIONS='1'
export KEYLESSPASS_ENTITLEMENT_SERIAL='1'
export KEYLESSPASS_CUSTOMER_VALID_UNTIL='2027-12-31T23:59:59Z'
export KEYLESSPASS_AUTHORIZED_DEVICE_KEY_IDS=''
cargo run -- issue-customer-entitlement > customer-entitlement.json
```

An empty allowlist permits initial server startup and request collection but no device issuance. After the admin imports requests and exports the device CSV, the vendor verifies the purchase, sets the approved lowercase SHA-256 key IDs as a comma-separated `KEYLESSPASS_AUTHORIZED_DEVICE_KEY_IDS` value, increments `KEYLESSPASS_ENTITLEMENT_SERIAL`, and issues a replacement file. The customer installs it and runs:

```bash
docker compose restart keylesspass-admin
```

## Commercial client build

Embed the vendor root, not the customer-site public key shown by the admin UI:

```bash
cd ..
KEYLESSPASS_LICENSE_KEY_ID='keylesspass-vendor-root-2026' \
KEYLESSPASS_LICENSE_PUBLIC_KEY_B64='<vendor root public key>' \
CODESIGN_IDENTITY='Developer ID Application: Your Company (TEAMID)' \
tools/commercial/build_commercial_release.sh macos
```

Neither private key belongs in a client. Production packages also require platform code signing.

## Administrator workflow

1. Put distributable artifacts in `admin_backend/downloads/`; users download without login.
2. Open `/` and authenticate with the Admin token.
3. Create an organization within the vendor-signed limits.
4. Each client copies its device request from Security -> Commercial authorization.
5. Import requests. Newly seen device keys require vendor approval and a higher-serial entitlement.
6. After the entitlement update and restart, issue an offline bundle or use HTTPS online activation.
7. Import the bundle on the client and confirm the authorized state.

Online activation succeeds only for vendor-approved devices. An organization activation code is not an Admin token and is scoped to activation for that organization.

## Roles and API

Use `KEYLESSPASS_ADMIN_TOKEN` for a single administrator or `KEYLESSPASS_ADMIN_USERS_JSON` for separate `admin`, `operator`, and `auditor` accounts. Administrative APIs use `Authorization: Bearer <token>`. Downloads, health checks, and organization-code activation do not accept the Admin token.

Back up `.env`, the SQLite volume, the current entitlement, and the vendor issuance ledger. Pure offline software cannot reliably detect a fully cloned VM snapshot or receive immediate revocation. High-adversary deployments require vendor-side per-device issuance, a TPM/HSM-backed license server, or a commercial hardware licensing product.
