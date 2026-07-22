# Commercial Device Authorization Design

Chinese: [device-batch-authorization.zh-CN.md](device-batch-authorization.zh-CN.md)

For deployment and UI steps, use the [implementation and usage guide](device-batch-authorization-implementation.md).

## Goals and boundary

Licensing controls customer identity, endpoint counts, features, versions, and dates. It is outside password derivation. No licensing component may receive mnemonic phrases, `Kmaster`, factor secrets, CDR secrets, service passwords, derived passwords, or recovery keys. License failure must not delete password data.

## Trust model

```text
vendor root public key embedded in client
  verifies customer entitlement
    delegates customer-site public key
    constrains counts, dates, features, versions
    approves deviceKeyId allowlist
      verifies customer-site device bundle
        matches local device key and protected fingerprint
```

The vendor root private key remains offline. A customer holds only the site key. A valid site signature is insufficient by itself: the site key must be delegated by the vendor and the grant's device key must be in the vendor-signed allowlist.

This closes the fundamental loophole where a customer-controlled server changes `maxSeats` or issues many forked one-device bundles.

## Objects

- Customer entitlement: customer, site public key, monotonic serial, registered/concurrent/offline limits, maximum offline grace, dates, features, versions, and approved device keys.
- Organization license: customer-internal organization, plan, seats, features, versions, and grace.
- Device request: device public key/key ID, platform metadata, and proof of possession.
- Device grant: device key ID/public key, commercial ID, HMAC fingerprint, dates, and features.
- Bundle: vendor entitlement, organization license, grants, revocations, and site signature.
- Security state: maximum entitlement serial, newest bundle issue time, maximum observed local time, and a separately stored protected history marker retained across local-license clearing.

The current schema version is `2`.

## Device identity

An Ed25519 private key is the primary software identity; public UUIDs are auxiliary. Windows protects the identity and rollback state with DPAPI, macOS uses Keychain-backed protection, and Linux uses the current platform-provider fallback. Requests prove private-key possession. The backend rejects one key registered as several identities and silent key replacement.

Higher-assurance editions should replace software keys with TPM 2.0, Secure Enclave, or PKCS#11/HSM non-exportable keys and attestation.

## Seats and revocation

`seat_allocations` records active, expired, and revoked state. SQLite `BEGIN IMMEDIATE` performs expiry cleanup, counting, allocation, and bundle/grant persistence atomically. Revoking a device revokes all of its active grants and blocks reactivation with that identity.

Automatic intranet clients renew every 30 minutes against a 24-hour lease plus one default grace day, bounding but not eliminating revocation delay. Manually exported static offline grants stop only after an updated bundle arrives or the old grant expires.

## Strict node locking versus floating use

The secure default is a vendor-approved device allowlist, which strictly controls how many endpoints may receive valid grants. Customer admins still collect requests and distribute bundles, but a new endpoint requires a higher-serial vendor entitlement.

If a hostile customer controls the intranet server, code, and site key, a signed `maxConcurrentDevices` number cannot strictly enforce floating concurrency: the server can fork its state. Strict floating licensing needs vendor-online short leases and heartbeat, a TPM/HSM-anchored license server, or a commercial hardware licensing system such as Sentinel LDK or CodeMeter.

## Irreducible limitations

Local software cannot absolutely prevent a machine owner from patching checks or cloning a complete offline VM with all protected platform state. Never-online devices cannot receive immediate revocation. Platform code signing, published checksums, signed updates, vendor issuance records, customer watermarking, support gates, and contract audits remain necessary. Ordinary USB media is not an uncopyable licensing dongle.
