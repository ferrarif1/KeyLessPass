# KeyLessPass Admin Backend

This is the intranet management backend for commercial KeyLessPass device
authorization. It signs offline `.klp-license-bundle` files that the desktop
client can import through its commercial authorization panel.

The service stores and signs commercial metadata only:

- organizations, plans, seat counts, expiry dates, and feature names;
- device authorization requests exported by desktop clients;
- signed license bundle history;
- grant revocation records.

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

## API

All `/api/*` endpoints require:

```text
Authorization: Bearer <KEYLESSPASS_ADMIN_TOKEN>
```

Main endpoints:

- `GET /api/status`
- `GET /api/snapshot`
- `GET /api/organizations`
- `POST /api/organizations`
- `POST /api/device-requests/import`
- `GET /api/devices`
- `POST /api/licenses/issue`
- `POST /api/grants/{grantId}/revoke`

`GET /healthz` does not require a token.
