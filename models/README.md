# Rotation model

`PasswordRotation.tla` models one password-change operation for atomic
replacement, overlap-then-revoke, and opaque targets. It refines a set of four
possible remote credential states from old/new authentication probes and covers
lost update and revocation responses.

Run it with the official TLA+ command-line tools:

```bash
java -XX:+UseParallelGC -jar /path/to/tla2tools.jar \
  -config PasswordRotation.cfg PasswordRotation.tla
```

The checked invariants require local generation commit to follow `NEW_ONLY`
evidence, prohibit automatic commit for opaque targets, and permit `BOTH` as an
intermediate state only for overlap targets. This is a state-machine model, not
a proof of the cryptographic primitives, adapter evidence claims, SQLite
implementation, or network service.

`FactorPreservingRecovery.tla` models the separate opaque peer recovery
prototype. It distinguishes the two local Root-Key shares from the network
factor's storage, two-administrator authorization, three-node release, and
freshness requirements. The checked invariants prohibit a network-only Root-Key
recovery, release without every network subcondition, and release of a stale
object epoch. Exact results are in `FACTOR_RECOVERY_MODEL_CHECK_RESULTS.md`.

```bash
java -XX:+UseParallelGC -jar /path/to/tla2tools.jar \
  -config FactorPreservingRecovery.cfg FactorPreservingRecovery.tla
```
