# EPSCD Architecture

Date: 2026-08-10

## Scope

EPSCD is compatibility technology for enterprise systems that still accept only passwords. It does not recommend composition rules for modern authentication and does not modify the target verifier.

## Persistent metadata

The baseline stores no service-password value and no previous service-password value. It stores:

- credential, vault, service, and account identifiers;
- a random `credentialSalt` and explicit public `lineageID`;
- policy identifier, canonical policy hash, policy version, and policy epoch;
- root generation and committed credential generation;
- pending operation identifier, old generation, candidate generation, adapter type, and evidence journal;
- replica ancestry and optional freshness checkpoint metadata.

The no-persistent-password statement does not mean plaintext never exists. The old or candidate password exists transiently when displayed, submitted, or verified and may be exposed by an already-compromised endpoint during that operation.

## Derivation pipeline

1. Parse a bounded policy into a canonical `PolicySpec`.
2. Compile it into a finite deterministic transition system.
3. Compute exact suffix counts with arbitrary-precision integers and obtain `N_P = |L_P|`.
4. Derive a credential key from the root and canonical credential/lineage/policy context.
5. Apply the audited keyed permutation to generation `g` in `[0,N_P)`.
6. Unrank the resulting integer into exactly one accepted password.

`Rank` and `Unrank` are inverse bijections. A fixed keyed permutation and a fixed lineage therefore give a deterministic, injective sequence until `g = N_P`. The policy decoder does not generate and reject candidate strings. The selected permutation backend may perform bounded cycle walking and fail closed.

## Lineage boundary

Scheme v2 makes `lineageID` explicit and binds it, `credentialSalt`, root generation, policy version, and policy epoch into key/tweak separation. Same-lineage generations have strict non-repetition. A policy change or rekey starts a new independent lineage; overlap with earlier passwords is quantified rather than prevented by a stored history filter.

For new-domain size `N`, historical overlap `h = |H ∩ L_new|`, and `m` future outputs, the reported union bound is:

`Pr[collision] <= min(1, h*m/N)`.

The system also reports `log2 N`. Deployments can configure a minimum accepted number of effective bits; EPSCD warns rather than pretending to repair a weak remote verifier.

## Rotation architecture

The durable rotation path is:

`ACTIVE(g) -> PREPARED(g,g+1,opID) -> SUBMITTED -> VERIFYING`

and then exactly one of:

- `COMMITTED(g+1)` after adapter-defined sufficient evidence;
- `ABORTED(g)` after conclusive evidence that the old credential remains authoritative;
- `UNKNOWN_OUTCOME` when the evidence cannot distinguish the remote outcome.

Generic HTTP success, TCP delivery, or request submission is not sufficient evidence. While the outcome is uncertain, the journal retains the metadata required to reconstruct both generations. Reconciliation adds evidence; it does not guess or silently advance the generation.

## Existing-code mapping

The architecture reuses `policy`, `epscd`, `permutation`, `domain::rotation`, the SQLite CDR store, and deterministic regeneration. It does not require a new directory hierarchy or a new cryptographic dependency. CETS remains a separate optional deployment analysis and is outside the derivation and rotation safety proof.
