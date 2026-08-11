# Hostile recovery-security review — latest manuscript

## Recommendation

**Major revision.** The replacement design fixes the earlier factor-collapsing
architecture, but the second contribution is a system-security abstraction and
deployment profile, not a new secret-sharing algorithm.

1. **Is this just Shamir?**  The fragments are standard Shamir. The proposed
   object of analysis is the deployment capability closure around top-level
   shares.
2. **Is A just 2FA approval?**  Mechanistically it is independent signed
   approval. Its relevance is that D may request but cannot authorize release
   of the second top-level share.
3. **Why do threshold nodes not solve collapse?**  If D can automatically call
   enough honest nodes, their responses are already in `Closure(D)`.
4. **Can a compromised endpoint invoke honest nodes?**  It can submit a request,
   but lacks two A-domain signatures in the stated model.
5. **Can U be cloned?**  Yes; the paper treats it as copyable and gives it no
   approval secret.
6. **What is Closure(X)?**  Direct data plus secure-storage contents,
   credentials, request-signing powers, automatic calls, and all honest-service
   responses authorized by those capabilities.
7. **Why is A not a fourth Root-Key share?**  A stores signing authority only;
   its signatures authorize release but contain no Shamir point and cannot
   reconstruct Kroot.
8. **What prevents stale replay?**  Tickets bind Root-Key/share-set generations,
   short lifetime, random opID, session public key, nodes, and purpose; nodes
   enforce freshness and an idempotency ledger.
9. **What is closest?**  MFKDF, TOPPSS, SafetyPin, SVR3, secure account recovery,
   and SPHINX constrain the claim. Threshold, heterogeneous trust, authorization,
   and freshness are individually prior art.
10. **Is factor preservation genuinely new?**  Not yet established as a new
    general abstraction. The defensible claim is a closure-based analysis and
    lifecycle profile for a particular endpoint-plus-network-share failure.

## Missing evidence

The local 6.036 ms number excludes the dominant costs. A multi-host deployment,
durable node ledger, real independent identities, partitions, malicious or
faulty nodes, and approval workflow evaluation are needed before this can carry
the paper as a mature second contribution.
