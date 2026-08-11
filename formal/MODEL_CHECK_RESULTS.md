# EPSCD Lifecycle Model-Checking Record

Date: 2026-08-09

## Scope

`lifecycle.tla` is a standalone bounded model of EPSCD credential rotation. It includes:

- same-policy rotation;
- policy-epoch change;
- authenticated history exclusion represented as candidate generation indices;
- persistence before submission;
- submission and lost responses;
- `UnknownOutcome` reconciliation;
- `NewOnly`, `OldOnly`, `Both`, and `Neither` observations;
- evidence-gated commit, abort, ambiguous-state escalation, crash, and recovery.

The cryptographic derivation is abstracted by the injective `(policyEpoch, credentialGeneration)` index. The model does not prove HKDF or PRP security, database durability, adapter correctness, clock behavior, remote linearizability, or Byzantine-node behavior.

## Bounds

```text
MaxEpoch = 2
MaxGeneration = 3
```

History exclusion nondeterministically ranges over every subset of the four bounded generation indices. Exploration is exhaustive within these bounds only.

## Checked invariants

- `TypeInvariant`: every variable remains in its declared finite abstraction;
- `NoCommitWithoutNewOnly`: every post-initial commit records `NewOnly` evidence;
- `CommittedGenerationMatchesCandidate`: committed epoch/generation equals the last committed candidate;
- `CredentialSaltStableWithinLineage`: all pending candidates retain the active credential salt;
- `PolicyChangeAdvancesEpoch`: policy changes advance exactly one epoch;
- `SamePolicyRotationPreservesEpoch`: same-policy rotation preserves the epoch;
- `NoCommittedHistoryReuse`: a committed candidate is outside its authenticated exclusion set;
- `UnknownOutcomeDoesNotCommit`: entering or recovering into unknown outcome leaves the active credential and commit counter unchanged;
- `PendingBeforeSubmit`: submitted, observed, ambiguous, and recovery states retain a persisted pending candidate.

## Reproduction

```text
curl -L https://github.com/tlaplus/tlaplus/releases/latest/download/tla2tools.jar \
  -o /tmp/epscd-tla2tools.jar
java -XX:+UseParallelGC -cp /tmp/epscd-tla2tools.jar tlc2.TLC \
  -config lifecycle.cfg lifecycle.tla
```

TLC artifact SHA-256:

```text
936a262061c914694dfd669a543be24573c45d5aa0ff20a8b96b23d01e050e88
```

## Result

TLC 2.19 completed breadth-first exploration with no invariant violation:

```text
3,129,888 states generated
1,006,128 distinct states found
0 states left on queue
complete graph depth 56
maximum outdegree 31
```

The final verification run used fingerprint seed `-5988333856285425141`.
TLC reported an optimistic fingerprint-collision omission estimate of `1.2e-7`
and an estimate based on actual fingerprints of `2.9e-7`. The result supports
only the listed invariants in the bounded abstraction; it is not a proof of the
complete implementation or remote service.
