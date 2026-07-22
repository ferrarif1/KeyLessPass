# KeyLessPass Admin Backend

This is the intranet management backend for commercial KeyLessPass device
authorization. It signs offline `.klp-license-bundle` files and supports online
activation from the desktop commercial authorization panel.

The service stores and signs commercial metadata only:

- organizations, plans, seat counts, expiry dates, and feature names;
- device authorization requests exported by desktop clients;
- signed license bundle history;
- grant revocation records.
- append-only administration audit events.

It must never receive or store mnemonic phrases, `Kmaster`, `deviceSecret`,
`usbSecret`, service passwords, derived passwords, CDR secrets, or recovery
wrapper keys.

## One-Click Intranet Deployment

Prerequisites: Docker and Docker Compose on an intranet host.

```bash
cd admin_backend
./scripts/intranet_deploy.sh
```

The script creates `.env` on first run, generates a random admin token and a
random Ed25519 signing seed, builds the container, starts it, and prints the
local URL and token.

Open:

```text
http://127.0.0.1:8787
```

For LAN access, replace `127.0.0.1` with the server address and keep the port
inside the intranet firewall.

Online desktop activation requires HTTPS unless the service is running on the
same computer. Put the service behind the organization's TLS reverse proxy and
apply request rate limits to `/api/activation/activate` before LAN or internet
exposure.

## Manual Local Run

```bash
cd admin_backend
cargo run -- generate-key
export KEYLESSPASS_ADMIN_TOKEN="$(openssl rand -hex 32)"
export KEYLESSPASS_LICENSE_SIGNING_KEY_B64="<generated seed>"
export KEYLESSPASS_LICENSE_KEY_ID="keylesspass-license-2026-q3"
export KEYLESSPASS_ADMIN_DB="./keylesspass-admin.sqlite3"
cargo run
```

## Commercial Client Public Key

The admin backend signs bundle payload bytes with Ed25519. The desktop client
verifies those signatures with an embedded public key.

After deployment, log in to the admin page and copy `publicKeyB64` or
`publicKeyB64Url`. Commercial KeyLessPass builds must embed that value for the
same `KEYLESSPASS_LICENSE_KEY_ID`. Do not ship the private signing seed with a
client.

## Workflow

1. Create an organization with max seats, plan, features, and expiry.
2. In each KeyLessPass desktop client, export a device authorization request.
3. Paste the request into this admin backend and assign it to the organization.
4. Select devices and issue a signed license bundle.
5. Copy or download the bundle and import it into the desktop client.
6. When revoking a grant, issue a fresh bundle so clients can import the updated
   revocation list.

For online activation, securely deliver the organization's generated activation
code. The desktop user enters the HTTPS service URL and activation code; the
service consumes a seat and returns a device-bound signed bundle.

## Roles and audit

`KEYLESSPASS_ADMIN_TOKEN` remains supported as a single full administrator.
For separate accounts, set `KEYLESSPASS_ADMIN_USERS_JSON` to a JSON array of
users with unique tokens of at least 24 characters:

```json
[
  {"name":"admin","role":"admin","token":"..."},
  {"name":"license-operator","role":"operator","token":"..."},
  {"name":"audit-reader","role":"auditor","token":"..."}
]
```

- `admin`: create organizations and revoke grants;
- `operator`: import devices, bulk import CSV, issue bundles, and view activation codes;
- `auditor`: read service status and export device/audit CSV files.

Mutation events record actor, role, action, target, and time without recording
password secrets or administrator tokens.

Bulk device import accepts UTF-8 CSV with the headers below. Because the device
request is JSON, use a standards-compliant CSV writer so quotes and commas are
escaped correctly.

```csv
requestJson,organizationId,seatLabel
"{""schemaVersion"":1,""requestId"":""req-..."",...}",org-acme,Finance laptop 01
```

## Signing-key rotation

Generate a new signing seed with `cargo run -- generate-key`, deploy it under a
new `KEYLESSPASS_LICENSE_KEY_ID`, and build clients with both old and new public
keys in `KEYLESSPASS_LICENSE_TRUSTED_KEYS_JSON`. After all supported clients
trust the new key, switch the backend to the new seed. Keep the old public key
during the overlap window; private seeds never go into the client or browser.

## API

Administrative `/api/*` endpoints require:

```text
Authorization: Bearer <KEYLESSPASS_ADMIN_TOKEN>
```

Main endpoints:

- `GET /api/status`
- `GET /api/snapshot`
- `GET /api/organizations`
- `POST /api/organizations`
- `POST /api/device-requests/import`
- `POST /api/device-requests/import.csv`
- `GET /api/devices`
- `GET /api/devices.csv`
- `POST /api/licenses/issue`
- `POST /api/grants/{grantId}/revoke`
- `GET /api/audit.csv`

`POST /api/activation/activate` is authenticated by the organization activation
code rather than an administrator token. `GET /healthz` does not require a
token.
