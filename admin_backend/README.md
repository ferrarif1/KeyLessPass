# KeyLessPass Pure-Intranet Licensing Service

Chinese: [README.zh-CN.md](README.zh-CN.md)

The recommended workflow is automatic collection followed by one offline vendor approval. End users only download, install, and start the app. They do not enter a server address, activation code, or Admin token.

## Customer workflow

### 1. Start the service

Install Docker, Docker Compose, and OpenSSL, then run:

```bash
cd admin_backend
./scripts/intranet_deploy.sh
```

The first run creates a site Ed25519 key and a deployment-maintenance token. Send the printed site key ID and public key, customer name, purchased device count, and term to the vendor. Install the returned vendor public key and initial `license/customer-entitlement.json`, then run the same script again. For a commercial delivery, the vendor should complete this bootstrap in advance and ship a customer-unique site-key configuration plus initial entitlement, leaving the customer with a single script run. If the delivered config has no valid maintenance token, the script generates one locally so the vendor does not know it.

The script detects the intranet IP. For multiple NICs, NAT, fixed DNS, or a reverse proxy, set the reachable address explicitly in `.env`:

```dotenv
KEYLESSPASS_PUBLIC_BASE_URL=http://10.20.30.40:8787
```

### 2. Let users download

The script prints `http://server-ip:8787/download`. It requires no login. Place signed installers in `admin_backend/downloads/`.

Clicking any installer also downloads a server-generated `keylesspass-client-config.json`. Clients locate the service in this order:

1. the newest `keylesspass-client-config.json` in a managed path or the user's Downloads directory;
2. UDP discovery on port `8788` only when config is missing;
3. if both fail, remain unlicensed and wait for IT remediation—no activation data is requested from the user.

Users normally move nothing. If the browser blocks multiple downloads, allow downloads for this intranet site. For managed rollout, IT may also deploy the config to:

| Platform | Configuration location |
| --- | --- |
| macOS | `/Library/Application Support/KeyLessPass/keylesspass-client-config.json` or the equivalent user path |
| Windows | `%PROGRAMDATA%\KeyLessPass\keylesspass-client-config.json` or the `%APPDATA%` equivalent |
| Linux | `/etc/keylesspass/client-config.json` or `~/.config/keylesspass/client-config.json` |
| Any | beside the executable, or set `KEYLESSPASS_CLIENT_CONFIG` to an absolute path |

The config is only a server-location hint. It grants no rights. The returned license must still pass the vendor-root Ed25519 chain embedded in the client.

### 3. Approve the collected devices

After all target clients have started once, open the service root and enter the deployment token:

1. click **Export batch request**;
2. send `keylesspass-offline-approval-request.json` to the vendor;
3. import the returned higher-serial `customer-entitlement.json`;
4. the service restarts and polling clients authorize themselves.

Authorized clients also renew every 30 minutes. The server defaults to a 24-hour signed lease plus one offline grace day. Revoked or de-approved devices receive no renewal, so the last lease stops no later than the end of that bounded window. `KEYLESSPASS_AUTOMATIC_LEASE_HOURS` and `KEYLESSPASS_AUTOMATIC_GRACE_DAYS` are configurable, but grace cannot exceed the vendor entitlement.

The token is only for batch exchange and maintenance. Public downloads, automatic registration, and automatic activation never receive it.

## Vendor approval

Run this only on a vendor-controlled offline workstation:

```bash
cd admin_backend
export KEYLESSPASS_VENDOR_SIGNING_KEY_B64='<offline vendor root seed>'
export KEYLESSPASS_VENDOR_KEY_ID='keylesspass-vendor-root-2026'
export KEYLESSPASS_DEVICE_BATCH_REQUEST_FILE='/path/keylesspass-offline-approval-request.json'
export KEYLESSPASS_CUSTOMER_VALID_UNTIL='2027-12-31T23:59:59Z'
cargo run -- issue-customer-entitlement > customer-entitlement.json
```

The batch embeds the previous vendor-signed entitlement. The command verifies it and inherits the customer, delegated site key, serial, purchased limit, dates, and features; editing the batch quota is rejected. It increments the serial and refuses more devices than the signed limit. Renewal or expansion requires explicit vendor-side contract overrides. To approve only selected devices, set `KEYLESSPASS_AUTHORIZED_DEVICE_KEY_IDS` to the reviewed comma-separated list.

The initial entitlement can still be produced with the traditional environment variables and an empty allowlist. An empty allowlist permits startup and collection but cannot authorize any device.

## Publish installers

First apply Developer ID/notarization, Authenticode, or Linux GPG signing. Then create an immutable vendor-signed release manifest on the offline vendor workstation:

```bash
export KEYLESSPASS_VENDOR_SIGNING_KEY_B64='<offline vendor root seed>'
export KEYLESSPASS_VENDOR_KEY_ID='<explicit vendor root key ID>'
export KEYLESSPASS_RELEASE_DIRECTORY="$PWD/downloads"
cargo run -- issue-release-manifest > downloads/release-manifest.json
```

At startup the backend verifies this manifest and lists only installers whose name, size, and SHA-256 match. Re-sign the manifest whenever an installer changes. Root key IDs have no production default.

## Ports and security

| Port | Purpose | Required |
| --- | --- | --- |
| TCP 8787 | downloads, generated config, registration, maintenance | yes |
| UDP 8788 | fallback discovery when config is missing | optional |

Expose these only to the customer intranet. Across untrusted segments, use an HTTPS reverse proxy and restrict the root and `/api/offline-approval/*`; never publish the service to the Internet.

Automatic activation defaults to 20 requests per source IP per minute. If a reverse proxy makes many endpoints share one source address, preserve the real source address or size `KEYLESSPASS_AUTOMATIC_REQUESTS_PER_MINUTE` appropriately while retaining endpoint-VLAN ACLs.

The customer-site key can sign only within a vendor-signed quota and device allowlist. Commercial clients trust only the vendor root. Device requests prove possession of a per-device Ed25519 key, and seat allocation is transactional. The backend handles licensing metadata only and must never receive password or factor secrets.

Back up `.env`, the SQLite volume, the current entitlement, and the vendor issuance ledger. This pure-intranet edition is sold by vendor-approved device identity; it does not promise strict floating concurrency against a customer-controlled backend and cannot perfectly detect complete VM clones. Use vendor-online, TPM/HSM, or hardware licensing for high-adversary deployments.
