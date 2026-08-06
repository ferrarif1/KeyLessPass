# Credential Description Record Specification v3

## Serialization and authentication

CDR v3 uses RFC 8785 JSON Canonicalization Scheme through `serde_json_canonicalizer 0.3.2`. The HMAC input is the canonical object with `macTag` set to the empty string. `K_cdr_authentication` is derived from the Root Key with `vaultID`, `rootGeneration`, and `cryptoSuiteVersion`. Ordinary JSON member order is never security-significant.

## Fields

| Field | Meaning | Password-changing? |
|---|---|---:|
| `schemaVersion`, `cryptoSuiteVersion` | Data and crypto interpretation | Suite/derivation migration requires an explicit credential rotation |
| `vaultID` | Owning vault; rejects cross-vault replay | Yes, through key hierarchy and validation |
| `recordID` | Stable logical credential identity across generations | No in derivation v2; authenticated administrative identity |
| `recordSeq` | Human/audit ordering within a vault; not a freshness proof | No in derivation v2; yes only for backward-compatible derivation v1 |
| `serviceID`, `accountID` | Stable non-display service/account identities | Yes |
| `credentialGeneration` | Monotonic service-password generation | Yes |
| `rootGeneration` | Root Key generation authenticating this record | Yes when Root Key changes |
| `policyID`, `policyVersion` | Policy identity and revision | Encoder changes require rotation |
| `encoderVersion`, `derivationVersion` | Algorithm dispatch | Yes |
| `rotationState`, `operationID` | Crash-recovery workflow and idempotency | No until the candidate generation is committed |
| `parentRecordHash` | Hash of the authenticated parent record | No; detects missing parent/forks |
| `replica` | `replicaID`, Lamport clock, and epoch | No |
| display name, service/account hints, notes | User-facing metadata | No |
| `salt`, encoding descriptor | Credential salt and policy-hash inputs | Yes |
| `version` | Storage row version retained for migration and lookup | No in derivation v2 |
| timestamps, retired state | Lifecycle evidence | No |
| `macTag` | Integrity/authenticity | No |

`recordID` answers “which logical credential?” while `recordSeq` provides deterministic local ordering and preserves derivation-v1 compatibility. Neither alone establishes global freshness.

## Password derivation version 2

All UUIDs use lower-case hyphenated text, integers are RFC 8785 JSON numbers,
strings are UTF-8 JSON strings, and the credential salt is canonical padded
Base64 for exactly 16 bytes. Missing required fields, `null`, empty identifiers,
and unsupported versions fail validation rather than being normalized together.

```text
suiteSalt = SHA-256(
  "KeyLessPass/vault-subkey/salt/v1" ||
  vaultID[16] || u64be(rootGeneration) || u32be(cryptoSuiteVersion)
)

Kpwd = HKDF-SHA-256(
  Kroot, suiteSalt,
  "KeyLessPass/vault-subkey/v1/password-derivation", 32
)

policyHash = SHA-256(JCS({
  policyId, policyVersion, encodingDescriptor
}))

input = JCS({
  accountId, credentialGeneration, credentialSalt,
  derivationVersion, domain, encoderVersion, policyHash, serviceId
})

seed = HMAC-SHA-256(Kpwd, input)
password = EncoderV2(seed, encodingDescriptor)
```

The domain string is `KeyLessPass/password-derivation-input/v1`.
`recordID`, `recordSeq`, display fields, operation evidence, and replica fields
are deliberately excluded. Encoder v2 expands `seed` as
`HMAC-SHA-256(seed, "KeyLessPass/password-encoder/v2" || u64be(blockCounter))`;
bounded indices use rejection sampling before modulo reduction. The fixed vector
is `rust_core/test-vectors/password-derivation-v2.json`.

## Rotation and parent rules

A candidate keeps `recordID`, `serviceID`, `accountID`, `policyID`, and `rootGeneration`; increments `credentialGeneration`, `version`, replica Lamport clock, and epoch; generates a new salt and operation ID; and records the authenticated hash of the parent. Display-only edits do not change password inputs.

## Validation and migration

The decoder supplies explicit defaults only for legacy records. Legacy MAC verification reconstructs the exact v2 field layout and JSON order. Migration verifies the legacy MAC before assigning v3 identities and re-authenticating the canonical object. Unknown schema, suite, encoder, or derivation versions must fail closed rather than downgrade.

## Canonical-vector command

```bash
cd rust_core
cargo test canonical -- --nocapture
```

The research evaluation writes canonical envelope and record sizes to `experiments/results/*/measurements.json`.
