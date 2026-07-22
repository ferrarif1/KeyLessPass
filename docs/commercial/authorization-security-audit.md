# Commercial Authorization Security Audit

Chinese: [authorization-security-audit.zh-CN.md](authorization-security-audit.zh-CN.md)

Audit date: 2026-07-22

Scope: Rust authorization/enforcement, Flutter library loading, backend trust/authentication/seats/revocation, commercial packaging, and deployment documentation.

## Pure-intranet automatic-authorization delta audit

- UDP 8788 responses and `keylesspass-client-config.json` are location hints, not trust roots. Forging either cannot create a client-verifiable grant.
- Public `/api/automatic/activate` receives licensing metadata and device-key proof only; it receives no password secret, maintenance token, or signing private key.
- A `deviceKeyId` absent from the vendor entitlement can only remain pending. A modified site service cannot bypass the vendor-root check in the commercial client.
- Batch export/import requires the deployment-maintenance token. Import verifies vendor signature, customer, delegated site key, validity, increasing serial, and registered device set before atomic replacement and restart.
- The added residual risk is metadata/disk denial of service by a malicious intranet host generating valid self-signed identities. The automatic queue defaults to `max(32, purchased devices x 4)`; also restrict TCP 8787 and UDP 8788 to endpoint subnets and optionally rate-limit at the reverse proxy. Rate limiting is not an authorization trust boundary.

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
| Clock/license rollback | Protected maximum time, entitlement serial, and bundle issue time retained across clear |
| Release loading a debug library | Product mode loads only the packaged fixed library path |
| Accidental free commercial package | License-root preflight plus required platform/GPG signing, with explicit local-test override only |
| Unguarded auxiliary functions | Commercial guard covers password workflows, lists, USB tools, and mnemonic generation |
| Persistent browser Admin token | Tab-scoped `sessionStorage`, constant-time token digest comparison, RBAC and tenant scope |
| Login required for downloads | Download routes are public while administrative routes remain token protected |

## Verification

- Rust Core: 49 tests passed, including device mismatch, clone resistance, signature tampering, vendor allowlist, feature/grace escalation, rollback, and commercial compile enforcement.
- Admin backend: 6 tests passed, including proof verification, cross-organization identity, duplicate key, and seat-cap enforcement.
- Flutter: 11 tests passed and static analysis reported no issues.
- Live smoke test: downloads and health returned 200; admin status returned 401 without a token and 200 with the valid token.
- Shell syntax and diff whitespace validation passed.

## Residual boundaries

1. A machine owner can attempt to patch the client. Platform signing, published checksums, signed updates, customer watermarking, and contract audits remain necessary.
2. Pure offline software cannot reliably distinguish two complete VM/vTPM/system-state clones that never contact a common authority.
3. A hostile owner of a floating license server can fork concurrent state. Strict concurrency requires vendor-online short leases, a TPM/HSM-anchored server, or a commercial hardware licensing system.
4. Offline revocation takes effect only after an updated bundle arrives or the old grant expires.

Platform assurance differs: Windows currently uses machine-scope DPAPI software keys, macOS uses Keychain-protected software Ed25519, and Linux uses file/AEAD fallback. High-assurance editions should use TPM 2.0, Secure Enclave, PKCS#11/HSM, Sentinel LDK, or CodeMeter.

## Production requirements

Keep the vendor root offline and out of customer systems, ordinary CI variables, clients, and source control. Maintain a dual-control issuance ledger and increment `entitlementSerial` for every device approval change. Use HTTPS, activation rate limiting, restricted admin ingress, backups, and redacted logs. Ship only platform-signed/notarized or GPG-manifest-verified artifacts. Do not promise absolute software-only DRM or describe ordinary USB media as an uncopyable dongle.
