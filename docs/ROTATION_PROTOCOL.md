# Evidence-Bounded Credential Rotation

This workflow is a staged, evidence-bounded credential-rotation protocol. It is
not distributed two-phase commit: a password-only target is not a transaction
participant and a successful update response is not sufficient commit evidence.

## Research question and claim boundary

The core question is when a client may safely replace its last known-good local
credential after observing only target-specific authentication results. The
protocol does not claim that local probes reveal every replica. An adapter may
mark a password failure as `conclusive_failure` only when its target contract
justifies that interpretation; timeouts, CAPTCHA, MFA, rate limiting, unknown
endpoint routing, and unbounded replication delay are `indeterminate`.
Evidence intersection also assumes one observation round is quiescent with
respect to credential mutations other than the operation being reconciled. If
another administrator or backend process may rotate the same credential during
the round and the adapter cannot detect or serialize that activity, the target
must be treated as opaque and the observations cannot authorize auto-commit.

The design models four possible remote states:

```text
OLD_ONLY    only the parent credential authenticates
NEW_ONLY    only the candidate credential authenticates
BOTH        both credentials authenticate
NEITHER     neither credential authenticates
```

The persisted evidence starts as the full set of possibilities. Every sound
probe intersects that set:

```text
new succeeds           -> {NEW_ONLY, BOTH}
new conclusively fails -> {OLD_ONLY, NEITHER}
old succeeds           -> {OLD_ONLY, BOTH}
old conclusively fails -> {NEW_ONLY, NEITHER}
indeterminate          -> no refinement
```

Mutually inconsistent probes are rejected. The complete probe list, endpoint
identity, observation time, target contract, and remaining possibility set are
stored in the candidate CDR and covered by its MAC.

## Target contracts

### `atomic_replacement`

The target is specified to replace one password with another. Local commit is
allowed only after evidence converges to `NEW_ONLY`. `BOTH` is ambiguous (for
example, replica lag or an incorrect target description) and stops automation.

### `overlap_then_revoke`

The target deliberately permits multiple active credentials. `BOTH` is an
expected intermediate state:

```text
PREPARED -> UPDATE_SENT -> OVERLAP_ESTABLISHED
                             |
                             v
                    OLD_REVOCATION_SENT
                       /             \
              response known       response lost
                     |                   |
                     v                   v
              REMOTE_CONFIRMED   OLD_REVOCATION_UNKNOWN
                     ^                   |
                     +---- NEW_ONLY -----+
```

Requesting old-credential revocation resets the evidence set because the remote
mutation may have changed the true state. The client must obtain fresh
`NEW_ONLY` evidence before local commit.

### `opaque_replacement`

The adapter cannot establish target-wide coverage or a bounded convergence
condition. Even a locally observed `NEW_ONLY` state becomes
`EVIDENCE_INSUFFICIENT`; automatic local commit is prohibited. Operator action
may be implemented outside the automatic protocol, but must not be reported as
target-wide proof.

## Persistent states

```text
STABLE -> PREPARED -> UPDATE_SENT -> UNKNOWN_OUTCOME
                                      |
                                      v
                            RECONCILIATION_REQUIRED
                              /       |       \
                    NEW_ONLY     OLD_ONLY     NEITHER
                       |             |           |
                       v             v           v
              REMOTE_CONFIRMED   ABORTED   ROLLBACK_REQUIRED

ATOMIC + BOTH  -> AMBIGUOUS_REMOTE_STATE
OVERLAP + BOTH -> OVERLAP_ESTABLISHED -> OLD_REVOCATION_SENT
OPAQUE + any singleton -> EVIDENCE_INSUFFICIENT

REMOTE_CONFIRMED -> LOCAL_COMMITTED -> STABLE
```

Illegal and replayed transitions fail. The parent CDR remains active until the
candidate reaches `REMOTE_CONFIRMED` with a persisted `NEW_ONLY` evidence set.
`confirmRotation` no longer manufactures remote-success evidence; it only
performs the local commit after this precondition has already been met.

## Adapter contract

```text
contract() -> atomic_replacement | overlap_then_revoke | opaque_replacement
submit_update(old, candidate) -> response-or-unknown
classify_response(response) -> rejected | requires-verification | unknown
probe_new(candidate, endpoint, budget) -> success | conclusive_failure | indeterminate
probe_old(old, endpoint, budget) -> success | conclusive_failure | indeterminate
request_old_revocation(old) -> response-or-unknown        # overlap targets only
coverage_claim() -> endpoints/quorum/convergence bound or none
lockout_budget() -> maximum safe authentication attempts
retry_backoff(attempt) -> target-specific delay
```

The core exposes `recordRotationProbe` and `requestOldRevocation` through the
JSON FFI. Production target connectors must supply the transport, endpoint
identity, lockout controls, and evidence-classification justification.

## Checked invariants

`models/PasswordRotation.tla` explores all three target contracts, both probe
orders, lost update/revocation responses, the overlap stage, and local commit.
The checked safety properties are:

1. local generation advances only with `NEW_ONLY` evidence;
2. an opaque target never commits automatically;
3. `OVERLAP_ESTABLISHED` is reachable only under the overlap contract with
   `BOTH` evidence;
4. an atomic target observed in `BOTH` never commits.

The model is a finite control-state proof. It does not prove adapter soundness,
network coverage, cryptographic implementations, SQLite durability, or target
behavior.
