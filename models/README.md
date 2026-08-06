# Rotation model

`PasswordRotation.tla` models one password-change operation and the four remote
authentication observations after an unknown transport outcome: only the new
password works, only the old password works, both work, or neither works.

Run it with the official TLA+ command-line tools:

```bash
java -XX:+UseParallelGC -jar /path/to/tla2tools.jar \
  -config PasswordRotation.cfg PasswordRotation.tla
```

The checked invariants require local generation commit to follow `NEW_ONLY`
evidence and prohibit commit from ambiguous, aborted, or rollback-required
states. This is a state-machine model, not a proof of the cryptographic
primitives, target adapter, SQLite implementation, or network service.
