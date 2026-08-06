# Architecture Diagnosis Before the v3 Refactor

Date: 2026-08-06

## Exact pre-refactor recovery data flow

The v2 implementation generated one random 256-bit `K_master` and three factor values: `F_M` from a user-entered mnemonic processed by Argon2id and HKDF, `F_C` from the platform/device secret, and `F_U` from USB package material. It then derived three pair keys and encrypted the complete `K_master` three times:

```text
K_MC = HKDF(F_M || F_C, "KeyLessPass/wrap/MC") -> W_MC = AEAD(K_MC, K_master)
K_MU = HKDF(F_M || F_U, "KeyLessPass/wrap/MU") -> W_MU = AEAD(K_MU, K_master)
K_CU = HKDF(F_C || F_U, "KeyLessPass/wrap/CU") -> W_CU = AEAD(K_CU, K_master)
```

`W_MC` was stored in the local payload, `W_MU` and `W_CU` in the USB payload, and a copy of `W_CU` was also available to the local recovery workflow. Each successful pair decrypted a complete Root Key. No polynomial shares existed.

## Classification

This is a **pairwise recovery-wrapper construction**, not Shamir secret sharing, not general threshold cryptography, and not threshold MPC. Functionally it implements three authorized pairs, but cryptographically each authorization path opens a complete-key ciphertext.

## Mnemonic facts

The current code had already replaced direct HKDF processing with Unicode NFKC normalization, Argon2id (19 MiB, two iterations, one lane), and a final domain-separated HKDF. This is better than the rejected manuscript described. It remains a user-entered, guessable factor and is not a human-readable encoding of a uniformly random recovery share.

## Largest paper/code gaps found

- The manuscript described an “operational 2-of-3” construction while repeatedly using threshold-adjacent language that could be read as secret sharing.
- The manuscript called rotation “two-phase commit,” but the target service never participated in a transaction and the implementation immediately activated the new CDR.
- CDR MAC input used ordinary JSON serialization rather than a specified canonical representation.
- Password character selection and required-position selection used `% alphabet.len()`, introducing modulo bias; default required characters were placed at fixed, predictable positions.
- CDRs lacked explicit vault, root generation, policy, encoder, derivation, parent-hash, operation, and replica fields.
- Local/USB HMACs and record sequence checks did not provide global freshness or detect rollback of all valid copies.
- No production secret-sharing library, share envelope, share-set identity, factor generation, KCV, committed recovery manifest, or pairwise-to-share migration existed.
- The old prototype tests were strong for wrapper tamper detection but tested the wrong recovery primitive.

## Refactor boundary

The v3 core makes authenticated, version-bound Shamir 2-of-3 shares the selectable recovery schema and keeps the old wrapper reader only for migration. A v3 committed manifest takes precedence during password derivation. Archived v2 files are explicitly labelled deprecated. Full desktop UX conversion from “mnemonic” to “recovery share phrase” remains a deployment task and is not claimed as completed; see `LIMITATIONS.md`.
