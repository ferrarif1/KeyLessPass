# Evidence-Bounded Legacy Rotation Model

Date: 2026-08-10

## Adapter contract

Every adapter declares:

| Capability | Meaning |
|---|---|
| `CAN_VERIFY_NEW` | It can test the candidate without an unsafe side effect. |
| `CAN_VERIFY_OLD_SAFELY` | It can test the old credential within the lockout budget. |
| `HAS_ATOMIC_SUCCESS_EVIDENCE` | The target returns adapter-defined authoritative evidence that the credential transition committed. |
| `HAS_REMOTE_VERSION` | A version/readback value can be compared with the expected transition. |
| `SUPPORTS_IDEMPOTENCY_KEY` | Duplicate submission of the same `opID` does not create a distinct mutation. |
| `UNKNOWN_ONLY` | No safe automatic commit predicate exists. |

The evidence requirement is one of `NEW_ACCEPTANCE`, `NEW_ONLY`, `AUTHORITATIVE_VERSION`, or `UNKNOWN_ONLY`.

## Commit predicate

`CommitEvidence(adapter, observation)` is sufficient only when one of these adapter-declared conditions holds:

1. authoritative atomic evidence is present and supported;
2. the expected remote version is read back and supported;
3. `NEW_ACCEPTANCE` is required and a conclusive new-password probe succeeds;
4. `NEW_ONLY` is required, both probes are safe, the new probe succeeds, and the old probe conclusively fails.

An ordinary success response is recorded but ignored by this predicate. A timeout, indeterminate probe, CAPTCHA/MFA challenge, lockout-budget exhaustion, contradictory observation, or missing authoritative readback yields insufficient or contradictory evidence.

## Durable records

Before submission, `PREPARED` persists `opID`, committed generation `g`, candidate generation `g+1`, lineage and policy identifiers, adapter contract, and enough metadata to deterministically reconstruct both passwords. `SUBMITTED` is persisted before or conservatively recovered after a possible send. Observations are appended before any commit decision. Only then may `committedGeneration` advance.

## Required failure behavior

| Case | Required state/effect |
|---|---|
| Request never reaches target | `ABORTED` only with conclusive old-authoritative evidence; otherwise `UNKNOWN_OUTCOME`. |
| Target changes password but response is lost | `UNKNOWN_OUTCOME`, later commit after sufficient evidence. |
| Timeout or connection reset | `UNKNOWN_OUTCOME`. |
| Duplicate request | Reuse `opID` if supported; otherwise do not infer outcome. |
| Crash before submit | Recover `PREPARED`; no remote assumption. |
| Crash after submit before local persistence | Recover conservatively as submitted/unknown. |
| Crash after remote commit before local commit | Retain both generations and reconcile. |
| Only old accepted | Abort candidate when the evidence contract makes this conclusive. |
| Only new accepted | Commit for `NEW_ACCEPTANCE` or `NEW_ONLY` contracts as applicable. |
| Both accepted | Commit only for an explicit acceptance-only/overlap contract; otherwise ambiguous. |
| Neither accepted | Contradictory/rollback-required manual recovery; never advance. |
| Policy changed | Stop and create a new policy epoch/lineage decision; do not reuse the pending proof. |
| Lockout risk | Skip unsafe old probe and use another contract or remain unknown. |
| Old verification unavailable | Use new acceptance, authoritative evidence, or unknown-only according to adapter declaration. |
| No version/readback | Do not invent authoritative evidence; use probes or remain unknown. |

## Safety invariant

> The locally committed generation never advances unless persisted observations satisfy the adapter-specific evidence predicate for remote acceptance of the corresponding candidate credential.

This is a protocol invariant, not a claim of a new distributed transaction primitive.
