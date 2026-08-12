# ASTER threat model

## Adversary

The adversary may read endpoint metadata, observe an authorized plaintext during its intended output window, capture or replay capabilities, crash and restart the client, delay/drop/duplicate transport, return ambiguous target responses, steal an old Root-Epoch in the healing experiment, or compromise fewer evaluator domains than the configured threshold.

Compromise of the Approval Authority and compromise of a threshold of evaluator domains are analyzed separately because either can expand derivation authority. Malicious denial of service is in scope for availability discussion but not prevented.

## Assumptions

- The bounded policy compiler and Rank/Unrank implementation are correct for accepted Policy IR inputs.
- Every evaluator validates the identical canonical request and capability checks.
- The Approval Authority signing key is independent of evaluator Root-Epoch state.
- Root-Epoch replacement uses independent randomness; share refresh does not count as healing after complete root disclosure.
- Adapter evidence predicates are target-specific and may return `UNKNOWN_OUTCOME`.
- The eventual MPC claim requires a reviewed backend with its stated corruption model; the semantic evaluator does not satisfy this assumption.

## Goals

Exact policy compliance, deterministic reconstruction, same-lineage non-repetition, exact capability confinement, no endpoint Root-Epoch/reusable lineage-key API, failure-safe migration, safe epoch retirement, and progressive healing after independent Root-Epoch replacement.

## Non-goals

Protecting a plaintext already observed during legitimate use, strengthening a weak target policy, anonymous access, availability against malicious evaluators, automatic resolution of inherently ambiguous remote state, or calling recovery infrastructure from normal derivation credentials.
