# EncoderV2 to Exact Policy-Space V3 Migration

## Compatibility rule

Software installation never changes an existing credential's password. Dispatch is an exact pair:

```text
derivationVersion=2, encoderVersion=2 -> frozen EncoderV2
derivationVersion=3, encoderVersion=3 -> exact policy-space derivation
anything else                         -> fail closed
```

Legacy pre-CDR-v3 records retain their historical derivation path. The existing v2 fixed vector remains authoritative.

## Explicit migration

1. Authenticate and load the current committed v2 CDR.
2. Compile the chosen v3 policy and reject empty/unsupported/state-exploding policies.
3. Create a pending successor with `policyEpoch=1`, epoch-local `credentialGeneration=0`, and versions `(3,3)`.
4. Preserve the credential salt. Changing it on every rotation would choose a different permutation and void the intra-epoch non-repetition argument.
5. Re-derive the configured password-history window. If the v3 candidate equals a predecessor, consume the next generation index until a non-equal candidate is found or the domain is exhausted.
6. Present the candidate through the existing target-adapter/evidence interface.
7. Commit only after the contract establishes `new succeeds && old conclusively fails`; otherwise the v2 record remains committed.

The migration function creates only a pending CDR. It does not submit a browser form, automate a Web page, or infer a successful remote password change from an HTTP status alone.

## Subsequent v3 rotation

- Same canonical policy: keep `policyEpoch` and salt; increment `credentialGeneration`.
- Substantive policy change: increment `policyEpoch`, reset epoch-local generation to zero, retain authenticated predecessors, and run cross-epoch history exclusion.
- Domain exhaustion: do not apply modulo. Start a new authenticated policy epoch/domain and perform history exclusion, or require an operator-selected policy/root migration.
- Root-Key rotation: requires coordinated remote rotation of every affected credential. Historical values from an unavailable old Root Key cannot be asserted as checked.

## Fixed vectors

V2 vectors are immutable. V3 adds independent vectors for canonical policy bytes/hash, credential key, tweak (which excludes `credentialGeneration`), finite-domain permutation rank and final password. A new compiler/canonicalization rule requires a new encoder version, not replacement of a vector.
