# Crash-Recoverable Password Rotation

This workflow is a **staged password rotation / pending-confirm-reconcile protocol**, not distributed two-phase commit. A legacy target system is not a transaction participant.

## Persistent states

```text
STABLE -> PREPARED -> UPDATE_SENT -> REMOTE_CONFIRMED
                         |                  |
                         v                  v
                  UNKNOWN_OUTCOME     LOCAL_COMMITTED -> STABLE
                         |
                         v
             RECONCILIATION_REQUIRED
              /        |          |          \
 REMOTE_CONFIRMED   ABORTED   AMBIGUOUS   ROLLBACK_REQUIRED
```

Every transition is validated by `transition_rotation`; illegal or repeated events fail. The candidate CDR, operation ID, parent hash, and state are stored in SQLite before remote confirmation. The previous CDR remains active until remote-success evidence is recorded.

## Failure handling

| Failure point | Persisted evidence | Recovery action |
|---|---|---|
| Crash after prepare, before request | `PREPARED` candidate | Resume submission or abort |
| Crash after request submission | `UPDATE_SENT` | If response is missing, mark `UNKNOWN_OUTCOME` |
| Timeout / connection loss / ambiguous error | `UNKNOWN_OUTCOME` | Enter reconciliation; do not guess or auto-commit |
| New password authenticates | Evidence event | `REMOTE_CONFIRMED`, then local commit |
| Old password authenticates | Evidence event | `ABORTED`; retain parent active |
| Both passwords authenticate | Evidence event | `AMBIGUOUS_REMOTE_STATE`; stop automatic commit/rollback |
| Neither authenticates | Evidence event | `ROLLBACK_REQUIRED`; stop automated attempts and require operator handling |
| Remote rejection | Rejection evidence | `ROLLBACK_REQUIRED` / abort |
| Crash after remote success, before local commit | `REMOTE_CONFIRMED` | Idempotently complete local commit |

Reconciliation callers must enforce target-specific lockout budgets. The core state machine records the result but does not perform network authentication itself. Duplicate operation IDs with different records are classified as conflicts.

## Adapter contract

Every target adapter must implement the following behavior even if its concrete
transport is HTML forms, a vendor API, or an interactive automation layer:

```text
submit_update(old, candidate) -> response-or-unknown
classify_response(response) -> rejected | requires-verification | unknown
verify_new(candidate, remainingBudget) -> success | failure | indeterminate
verify_old(old, remainingBudget) -> success | failure | indeterminate
lockout_budget() -> maximum safe authentication attempts
retry_backoff(attempt) -> target-specific delay
```

An HTTP 200, success page, or accepted submission is not sufficient by itself
unless the adapter contract for that target supplies independent evidence that
the new credential authenticates. CAPTCHA, MFA, rate-limit, replication-delay,
or indeterminate results consume or stop the budget; they must not be mapped to
credential failure. The repository contains the contract and state-machine
boundary, not two production target integrations.

## Current API boundary

`rotateCredential` persists `PREPARED`; `confirmRotation` records verified remote-success evidence and performs a crash-visible local commit; `mark_rotation_unknown_with_provider` and `reconcile_rotation_with_provider` expose the uncertain-result paths to adapters. Target-specific connectors and UI attempt-budget controls are not yet implemented. `models/PasswordRotation.tla` independently checks the no-unconfirmed-commit and ambiguity invariants.
