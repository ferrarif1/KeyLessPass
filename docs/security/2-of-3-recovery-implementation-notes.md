# KeyLessPass 2-of-3 Local Recovery Implementation Notes

This note records the security boundary for the strict pairwise-wrapper recovery
schema. It is implementation guidance for the Rust core and UI wording.

## macOS Local Factor Source

- The macOS Keychain item remains in use:
  - service/location: `com.keylesspass.local-factor`
  - account: `keylesspass`
- This item is not `Kmaster` and is not the mnemonic.
- It is the platform-protected local factor source, equivalent to the paper's
  `deviceSecret` input.
- The item must only feed computer factor derivation:

```text
FC = KDF(deviceSecret || deviceID || userID || saltC)
```

- It must not be used to decrypt a local payload that persists plaintext
  `Kmaster`.

## Persistent Package Boundaries

- Local factor packages must not persist plaintext `Kmaster`.
- Local factor packages must not persist `usbSecret`.
- USB factor packages must not persist plaintext `Kmaster`.
- USB factor packages must not persist `deviceSecret`.
- The mnemonic phrase is never persisted.
- A mnemonic verifier may be persisted only for validation; it is not a recovery
  secret and must not become the only condition for recovering `Kmaster`.

## Pairwise Wrappers

`Kmaster` is a random 256-bit root secret. At rest, it may only appear as
ciphertext inside these pairwise wrappers:

```text
K_MC = HKDF(FM || FC, "KeyLessPass/wrap/MC")
K_MU = HKDF(FM || FU, "KeyLessPass/wrap/MU")
K_CU = HKDF(FC || FU, "KeyLessPass/wrap/CU")

W_MC = AES-256-GCM(K_MC, Kmaster)
W_MU = AES-256-GCM(K_MU, Kmaster)
W_CU = AES-256-GCM(K_CU, Kmaster)
```

Each wrapper must carry enough metadata for authenticated decryption, including
wrapper type, version, nonce, ciphertext, tag, and authenticated associated data
or enough stable fields to rebuild the AAD exactly.

## Recovery Invariant

Any two factors can recover the same `Kmaster`:

- mnemonic + computer: derive `FM` and `FC`, decrypt `W_MC`.
- mnemonic + USB: derive `FM` and `FU`, decrypt `W_MU`.
- computer + USB: derive `FC` and `FU`, decrypt `W_CU`.

Any single factor must fail:

- only mnemonic: fail.
- only computer/local package: fail.
- only USB package: fail.

## USB Factor Boundary

- USB is ordinary copyable storage, not an uncopyable hardware key.
- Copying the USB package copies the USB factor.
- A copied USB factor alone cannot recover `Kmaster`; it still needs either the
  mnemonic factor or the matching computer factor.
- The USB package must not be encrypted as a whole with a key derived only from
  the mnemonic factor.
- The USB package is a USB-factor container. It may store `usbId`, `saltU`,
  `usbSecret` or equivalent USB factor material, `W_MU`, `W_CU`, wrapper
  nonce/tag/version/AAD metadata, schema version, and integrity metadata.

## Local Factor Boundary

- The local package stores local package metadata such as `userId`, `deviceId`,
  `saltC`, `mnemonicSalt`, mnemonic verifier, `W_MC`, an optional `W_CU` copy,
  schema version, recovery generation, and password derivation algorithm.
- The local package does not store plaintext `Kmaster`.
- The local package does not store `usbSecret`.
- The platform secret from `com.keylesspass.local-factor` remains outside the
  JSON payload and is used as the source material for `FC`.

## Legacy Schema

Legacy packages that persist master-key payloads or mnemonic-encrypt the whole
USB payload do not satisfy this model. If automatic migration is unavailable,
the implementation must return a clear error:

```text
legacy factor package stores master-key payload and does not support strict pairwise-wrapper recovery; please migrate with the old mnemonic available.
```

