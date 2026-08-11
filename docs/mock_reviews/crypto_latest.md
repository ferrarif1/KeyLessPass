# Hostile cryptography review — latest manuscript

## Recommendation

**Major revision / weak reject.** The formal claim boundaries are substantially
better, but the novelty is a protocol-level specialization of existing
finite-language and FPE machinery.

1. **I know one password. Why can I not recover the permutation key?**  One
   observation supplies one input/output pair. Key recovery is ruled out only
   computationally by the selected PRP's security and key size, not by
   injectivity or counting.
2. **I know q passwords. What remains?**  In the ideal random-permutation model,
   conditioning on q distinct consistent pairs leaves an unseen input uniform
   over the `N_P-q` unused outputs. This is not forward secrecy.
3. **Are injectivity and PRP security confused?**  The latest text separates
   them: injectivity proves no repetition; PRP security supports
   unpredictability.
4. **Why can I not enumerate the policy language?**  I can. `L_P`, `N_P`, and
   rank/unrank are public. A compromised verifier permits direct testing of
   that finite language.
5. **Why does a 256-bit Root Key matter if N_P is small?**  It protects the key
   hierarchy and cross-credential compromise boundary; it does not enlarge a
   service password's actual guessing space.
6. **What if Kcred leaks?**  The complete sequence for that credential lineage
   is exposed.
7. **Does policyEpoch repair Kcred compromise?**  No. The implementation changes
   the credential salt to derive a new Kcred lineage and excludes authenticated
   history during the first new-lineage rotations.
8. **Does re-sharing repair Kroot compromise?**  No. Root-Key replacement and
   remote rotation of every affected credential are required.
9. **Can rollback force password reuse?**  Yes, by restoring an old valid
   generation input. External freshness is therefore a premise of deployment
   no-repeat.
10. **Is FF1 properly bounded?**  The manuscript enforces the one-million
    minimum, a 512-bit ceiling, and a cycle-walk cap and labels the backend a
    prototype. It does not prove total arbitrary-domain availability.

## Principal objection

The ideal residual-space theorem is a standard conditional symmetry of random
permutations. The publishable value must therefore come from the complete
credential-exposure/lifecycle definition and evidence, not from presenting the
lemma as new mathematics. A proof-matched backend or reduction with explicit
query bounds would materially strengthen the paper.
