# EPSCD Scheme-Version-1 Property Obligations

Executable checks support implementation correctness within tested bounds; they are not a cryptographic proof.

## Compiler and language

- Every transition target is present in the generated state table.
- Alphabet symbols are unique and retain canonical order.
- Unsupported or empty policies fail closed.
- For bounded toy policies, automaton acceptance equals an independent direct predicate for every enumerated string through `L_max`.

## Exact counting

- Total and per-length counts equal exhaustive enumeration for bounded policies.
- Counts use arbitrary-precision integers and include a case above `2^128`.

## Rank and unrank

- `rank(unrank(r)) == r` for every rank in bounded exhaustive domains.
- `unrank(rank(p)) == p` for every enumerated accepted password.
- `unrank(N)` and rejected strings fail.
- Property-based tests vary alphabet, length, class bounds, edge constraints, and run constraints.

## Finite-domain permutation

- `permute(x) < N` for each tested `x < N`.
- Forward and inverse operations compose to the identity.
- Exhaustive small-domain tests use a test permutation because the reference backend enforces its published minimum domain.
- Reference-backend tests cover cycle-walking inversion and configured fail-closed bounds.

## End-to-end EPSCD

- Every scheme-version-1 output is accepted by its compiled policy.
- Fixed key, context, policy, and generation reproduce the public test vector.
- Distinct generations under one epoch show no collisions through the tested range.
- Generation equal to `N` fails without modulo reduction.
- Changing each domain-separation field changes the canonical key or tweak context.
- Generation is absent from key and tweak encodings and appears only as permutation input.
- Fixed vectors contain `schemeVersion = 1` and only fields defined by the public scheme.

## Epoch and history

- A policy change creates a new epoch context.
- Cross-epoch exclusion re-derives authenticated predecessors and consumes rejected indices.
- Missing or unauthenticated predecessor state blocks an asserted history guarantee.
- For bounded domains, the first non-excluded candidate is found within `e+1` attempts when the exclusion set has size `e < N`.

Lifecycle crash reconciliation is modeled separately. Its model must invoke EPSCD as an abstract deterministic, injective derivation function and remain a standalone public-scheme model.
