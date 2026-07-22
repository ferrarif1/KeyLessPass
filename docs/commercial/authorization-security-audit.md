# Commercial Authorization Security Audit

Chinese: [authorization-security-audit.zh-CN.md](authorization-security-audit.zh-CN.md)

Audit date: 2026-07-22

Scope: Rust authorization/enforcement, Flutter library loading, backend trust/authentication/seats/revocation, commercial packaging, and deployment documentation.

## Pure-intranet automatic-authorization delta audit

- UDP 8788 responses and `keylesspass-client-config.json` are location hints, not trust roots. Forging either cannot create a client-verifiable grant.
- Public `/api/automatic/activate` receives licensing metadata and device-key proof only; it receives no password secret, maintenance token, or signing private key.
- A `deviceKeyId` absent from the vendor entitlement can only remain pending. A modified site service cannot bypass the vendor-root check in the commercial client.
- Batch export/import requires the deployment-maintenance token. Import verifies vendor signature, customer, delegated site key, validity, increasing serial, and registered device set before atomic replacement and restart.
- A malicious intranet host can still generate valid self-signed identities. The service defaults to 20 requests per source IP per minute and bounds the queue at `max(32, purchased devices x 4)`; also restrict TCP 8787 and UDP 8788 to endpoint subnets. Rate limiting is abuse resistance, not an authorization trust boundary.

## Conclusion

The implementation no longer treats a customer backend as the client trust root. A vendor-signed customer entitlement delegates the site key and explicitly approves device key IDs. Without modifying the official client, possession of the customer backend, database, and site private key is insufficient to authorize an unapproved endpoint.

This closes the direct over-issuance revenue risk but does not make a local application uncrackable. Client patching, complete offline VM cloning, and immediate offline revocation remain inherent limitations.

## Remediated findings

| Risk | Remediation |
| --- | --- |
| Runtime attacker-key injection | Trusted roots are compile-time only; commercial clients embed the vendor root |
| Unlimited issuance with a customer key | Vendor entitlement delegates the site key and allowlists each `deviceKeyId` |
| Cloneable public IDs/fingerprint | Device signing key, proof of possession, HMAC fingerprint, DPAPI/Keychain protection |
| One key registered as many devices | Unique database index plus immutable key/identity and organization checks |
| Concurrent seat over-issuance | `seat_allocations` and SQLite `BEGIN IMMEDIATE` atomic issuance |
| Expired grants retaining seats | Allocations expire before counting |
| Reactivation after revocation | All grants for the device are revoked and the identity cannot reactivate |
| Site expansion of features/dates/grace | Client enforces vendor, organization, and grant constraints including maximum grace |
| Clock/license rollback | Protected maximum time, entitlement serial, bundle issue time, and a separate history marker retained across clear |
| Release loading a debug library | Product mode loads only the packaged fixed library path |
| Accidental free commercial package | License-root preflight plus required platform/GPG signing, with explicit local-test override only |
| Unguarded auxiliary functions | Commercial guard covers password workflows, lists, USB tools, and mnemonic generation |
| Persistent browser Admin token | Tab-scoped `sessionStorage`, constant-time token digest comparison, RBAC and tenant scope |
| Login required for downloads | Download routes are public while administrative routes remain token protected |
| Entitlement expiry bypassing grace | Grant evaluation continues only within the vendor-signed maximum grace window |
| Old bundle overwriting a newer bundle | Signature, device, and rollback checks complete before atomic temporary-file replacement |
| Missing rollback state silently reinitialized | A stored license with missing security state fails closed and requires reapproval |
| Authorized clients not checking revocation | Clients renew every 30 minutes against a 24-hour lease plus one default grace day |
| Automatic registration replay | UUID/time-window validation and a unique database `requestId`, in addition to the bounded queue |
| Same-server self-attested download hashes | Backend verifies an offline vendor Ed25519 release manifest and lists only matching artifacts |
| Inconsistent root key ID defaults | Backend and commercial build entry point require an explicit key ID |

## Verification

- Rust Core: 52 tests passed, including grace behavior, separate history marker, missing security state, device mismatch, clone resistance, signature tampering, vendor allowlist, rollback, and commercial compile enforcement.
- Admin backend: 15 tests passed, including source-IP rate limiting, release-manifest tamper detection, short-lease renewal, stale/replayed requests, proof verification, cross-organization identity, duplicate key, and seat-cap enforcement.
- Flutter: 12 tests passed and static analysis reported no issues.
- Live smoke test: downloads and health returned 200; admin status returned 401 without a token and 200 with the valid token.
- Shell syntax and diff whitespace validation passed.

## Residual boundaries

1. A machine owner can attempt to patch the client. Platform signing, published checksums, signed updates, customer watermarking, and contract audits remain necessary.
2. Pure offline software cannot reliably distinguish two complete VM/vTPM/system-state clones that never contact a common authority.
3. A hostile owner of a floating license server can fork concurrent state. Strict concurrency requires vendor-online short leases, a TPM/HSM-anchored server, or a commercial hardware licensing system.
4. Pure-intranet revocation is bounded, not immediate: by default it takes effect no later than the remaining 24-hour lease plus one grace day.

Platform assurance differs: Windows currently uses machine-scope DPAPI software keys, macOS uses Keychain-protected software Ed25519, and Linux uses file/AEAD fallback. High-assurance editions should use TPM 2.0, Secure Enclave, PKCS#11/HSM, Sentinel LDK, or CodeMeter.

## Production requirements

Keep the vendor root offline and out of customer systems, ordinary CI variables, clients, and source control. Maintain a dual-control issuance ledger and increment `entitlementSerial` for every device approval change. Use HTTPS, activation rate limiting, restricted admin ingress, backups, and redacted logs. Ship only platform-signed/notarized or GPG-manifest-verified artifacts. Do not promise absolute software-only DRM or describe ordinary USB media as an uncopyable dongle.
