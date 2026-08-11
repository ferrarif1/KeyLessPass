# Final self-check — 2026-08-09

1. **Which existing results do not need rerunning?** Exact corpus compilation,
   published Dichopile transcription, cold/warm EPSCD measurements, cycle-walk
   corpus measurements, and the original lifecycle model remain valid because
   their code paths did not change. Exposure, recovery, and freshness evidence
   was added separately.
2. **Why do public P/L/N/Rank/Unrank not reveal Password_g?** They define the
   public domain and bijection, but the keyed permutation rank remains unknown.
3. **What does one known Password_g reveal?** Its public rank and thus one
   generation/rank pair for the credential permutation.
4. **Why is that not Kcred?** Recovering Kcred from such pairs is the backend's
   computational key-recovery problem; a pair is not the key.
5. **After q pairs, what is the ideal distribution?** An unseen input is uniform
   over the `N_P-q` unused outputs, conditioned on q distinct consistent pairs.
6. **Why only a computational real-backend claim?** A concrete PRP is not a
   literal uniformly sampled permutation; equivalence is bounded by its
   distinguishing advantage and query/domain conditions.
7. **What gives no-repeat versus unpredictability?** Permutation injectivity
   gives no-repeat. PRP security and secret Kcred support unpredictability.
8. **Does policyEpoch repair Kcred compromise?** No, because policyEpoch is
   public tweak/context state under the same compromised credential key.
9. **What rekey semantics is used?** Generate a fresh credential salt, derive a
   new Kcred and lineage identifier, retain authenticated prior descriptors,
   and exclude required recent history during remote rotation.
10. **Why does shareSetGeneration++ not repair Kroot compromise?** It changes
    shares of the same exposed value. A new random Root Key and remote
    credential rotations are required.
11. **How does rollback bypass the distinct-generation theorem?** It restores a
    previously used generation input; the theorem requires distinct inputs.
12. **How is an old authenticated CDR detected?** The independently anchored CAS
    compares CDR ancestry and per-credential policy epoch, generation, and
    lineage; an older tuple is rollback and equal counters with differing
    lineage/digest are forks.
13. **Why is recovery not automatically a second innovation?** Shamir,
    distributed recovery, approval, heterogeneous trust, and freshness are
    published prior art. Only the scoped closure analysis and lifecycle profile
    remain candidate system contribution.
14. **What is in Closure(D)?** Endpoint files and secure storage, API/TLS
    credentials, cookies, request signatures, automatic calls, and every
    response honest infrastructure releases to those capabilities.
15. **Why may network 3-of-5 still collapse?** If D can invoke three honest
    nodes unattended, their combined S_N is in Closure(D), which already
    contains S_D.
16. **What does A provide and not provide?** It provides independent Ed25519
    authorization over a session/generation-bound ticket; it provides no Root
    share and no password value.
17. **Why is A not a fourth Shamir share?** No polynomial evaluation or Root-Key
    material is stored by A; its signature only gates N's release.
18. **Why is a fully copied U insufficient?** It yields only S_U and no A signing
    authority. It cannot reconstruct S_N or Kroot alone.
19. **Ordinary loss versus compromise recovery?** Ordinary loss re-shares the
    same Root Key and advances shareSetGeneration. Root compromise samples a new
    Kroot, advances both generations, and rotates all affected remote passwords.
20. **Difference from closest work?** MFKDF derives/recover keys from factors;
    TOPPSS gates secret sharing with threshold OPRF; SafetyPin, SVR3, secure
    account recovery, and SPHINX cover distributed/heterogeneous recovery. This
    work narrowly analyzes whether endpoint-callable releases collapse an
    existing heterogeneous Root-Key access structure and binds remediation to
    the credential lifecycle. MFDPG is not used in the new paper because its
    peer-reviewed publication status was not verified.
21. **If prior art defeats the recovery claim?** Downgrade recovery to a system
    profile/evaluation section, restore an EPSCD-focused title, and retain only
    the exact exposure-aware sequence as the primary contribution.
22. **Is the combined paper coherent?** Yes: both axes preserve the intended
    credential-security state across exposure, rotation, recovery, and rollback.
    The paper would become incoherent if recovery privacy/P2P features were
    reintroduced without that lifecycle link.
23. **Which claims have which support?** Compliance, exact count, rank/unrank,
    injectivity, and ideal residual/exclusion results are mathematical;
    unpredictability and context separation depend on PRP/HKDF assumptions;
    corpus coverage, timings, adversarial paths, and bounded transitions are
    empirical/model-checking evidence.
24. **Was completed work repeated?** No full corpus recompilation, Dichopile
    reconstruction, MFDPG experiment, or old performance run was repeated. The
    existing exact counts were only reclassified for security floors.
25. **Most likely JISA desk-reject reason now?** Insufficient novelty or
    maturity: established components are carefully specialized, while the
    candidate recovery contribution lacks a deployed multi-host evaluation and
    the concrete finite-domain backend remains bounded.
