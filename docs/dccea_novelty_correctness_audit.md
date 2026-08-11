# Novelty and correctness audit: credential exposure threshold spectra

Date: 2026-08-09  
Decision: proceed to a standalone method-paper draft; do not describe the work
as a new cryptographic primitive or as submission-ready evidence.

## 1. Final method under review

For finite deployment domains `D`, canonical credential contexts `C`,
automatically exercisable monotone rules `R`, compromised domains `X`, and an
attacker-observed set of legitimately issued context tickets `T`, compute one
least fixed point:

```text
Cl_R(X,T) = lfp(S -> C0(X,T) union consequences_R(S)).
```

The analysis returns both Root capability and credential-output reachability:

```text
M_R(X,T) = 1 iff RootCapability is in Cl_R(X,T)
E_R(X,T) = {c in C | Derived(c) is in Cl_R(X,T)}.
```

Its central quantitative output is the credential exposure threshold spectrum
(CETS):

```text
tau_E(k,q) = min |X| such that some |T| <= q gives
             |E_R(X,T) minus T| >= k.
```

The weighted spectrum `rho_E(k,q)` independently minimizes deployment-domain
compromise cost.  This is not a probability estimate; it is a finite
reachability threshold under an explicit model.

## 2. Correctness findings

### 2.1 Reachable states

For credentials deterministically derived from a Root Key by a public
algorithm, Root access dominates context exposure:

```text
M_R(X,T) = 1  implies  E_R(X,T) = C.
```

The earlier four-independent-quadrant interpretation is therefore invalid for
this profile.  The corrected model has three reachable abstract outcomes:

1. no Root collapse and no authorization amplification;
2. authorization amplification without Root access; and
3. Root access with full-context exposure.

The useful separation is below the Root threshold: two deployments can have
the same Root-access predicate but different callable-output exposure.

### 2.2 Proven properties

Within the stated finite monotone model:

- `tau_E(k,q)` is nondecreasing in `k`;
- it is nonincreasing in the approval budget `q`, because budgets are upper
  bounds and therefore enlarge the feasible ticket family;
- if `tau_M` is the effective Root threshold, Root dominance gives
  `tau_E(k,q) <= tau_M` for every `1 <= k <= |C|`;
- for projection-bound tickets, the exposed set equals the target's projection
  equivalence class;
- exact one-ticket non-amplification holds iff that projection is injective on
  the finite context set; and
- if projection-class sizes are `s_1 >= ... >= s_p`, the exact maximum spill
  under at most `q` tickets is
  `sum_{i=1}^{min(q,p)} (s_i - 1)`.

The implementation separately optimizes cardinality and weighted cost; the
minimum-cost witness need not be the minimum-cardinality witness.

### 2.3 Scope and assumptions

The conclusions require all of the following:

- `C0(X,T)` contains tickets observed or callable by the attacker, not every
  approval issued anywhere in the system;
- contexts are canonical, finite, and enumerable for analysis;
- the rule system is monotone and omits revocation, timing, probability, and
  non-automatic human decisions unless explicitly encoded;
- Root dominance assumes public derivation from the Root Key for every modeled
  credential; and
- DPRF security, ticket unforgeability, and endpoint isolation remain external
  assumptions rather than analyzer outputs.

Exhaustive analysis costs approximately
`O(2^|D| * sum_{i=0}^q binom(|C|,i) * closure_cost)`.  The reference tool is
therefore for small trust-domain models and bounded ticket budgets, not for
internet-scale attack graphs.

## 3. Published-work search and overlap assessment

Searches covered exact names and concepts, official proceedings, publisher
records, Crossref, OpenAlex, and open bibliographic indexes. No exact match was found for
`Credential Exposure Threshold Spectrum` or for the complete pair
`(Root-capability reachability, approval-budget-indexed unauthorized
credential-output threshold)`.  This is evidence for continuing the work, not
proof of worldwide priority or plagiarism clearance.

| Published line of work | Established result | Overlap and boundary |
|---|---|---|
| Harrison--Ruzzo--Ullman, CACM 1976 | Safety/right acquisition in protection systems | Establishes access-control reachability; CETS must not claim to invent safety analysis. |
| MulVAL, USENIX Security 2005 | Datalog/Horn closure for multistage network attacks | Establishes logic closure and witness generation; the present closure engine is an application, not an algorithmic novelty. |
| Wang--Noel--Jajodia, Computer Communications 2006 | Minimum-cost network hardening using attack graphs | Establishes cost-based reachability; `rho_E` is novel only, if at all, as a credential-output-specific curve with an approval budget. |
| Maffeis--Mitchell--Taly, IEEE S&P 2010 | Capability and authority safety | Establishes non-amplification-style authority reasoning; the present paper must use the narrower term credential-output amplification. |
| Boneh--Waters, ASIACRYPT 2013 | Constrained PRFs | Establishes keys restricted to input subsets; context restriction is not new cryptography. |
| Macaroons, NDSS 2014 | Contextual caveats and attenuated authorization | Establishes context-bound capability tokens; full-context ticket binding is a repair, not the contribution. |
| Pythia, USENIX Security 2015 | A PRF service for password hardening and key rotation | Establishes remotely evaluated password-related PRFs; it does not report the CETS metric. |
| TOPPSS, ACNS 2017 | Threshold OPRF for password-protected secret sharing | Establishes threshold password recovery mechanisms; no new threshold primitive is claimed here. |
| SafetyPin, OSDI 2020 | Distributed encrypted backup recovery with HSM compromise analysis | Establishes system-scale recovery analysis; it studies backup-key recovery rather than per-context derivation authorization. |
| PolyScope, USENIX Security 2021 | Permission expansion and authorized attack operations under composed policies | Closest generic analysis precedent.  It substantially narrows any novelty claim to deterministic credential-output cardinality and Root dominance. |
| MFKDF, USENIX Security 2023; cryptanalysis, USENIX Security 2024 | Multi-factor derivation and failures caused by state integrity/underspecification | Establishes threshold factor-derived keys and the importance of precise state integrity; not a callable-output exposure spectrum. |
| LaKey, USENIX Security 2024 | Scalable distributed key derivation using lattice DPRFs | Decisive prior art against claiming non-reconstructed master keys or DPRF-based derivation as new. |
| Stateful Least Privilege Authorization, USENIX Security 2024 | Stateful privilege attenuation and token-abuse blast-radius reduction | Establishes least-privilege authorization and blast-radius motivation; it does not combine Root reachability with a credential-output threshold spectrum. |
| SVR3 and Flock, OSDI 2024 | Distributed secret recovery and privacy-preserving key management | Establish recovery-system precedents; they are not evidence that CETS is already defined. |
| KSDAuth, Textile \& Leather Review 2026 | Threshold signing over a canonical identity context | Further establishes context-bound key sharding as prior art; it does not define a password-output exposure spectrum. |

Primary records used in the audit:

- HRU: https://doi.org/10.1145/360303.360333
- MulVAL: https://www.usenix.org/conference/14th-usenix-security-symposium/mulval-logic-based-network-security-analyzer
- minimum-cost attack graphs: https://doi.org/10.1016/j.comcom.2006.06.018
- object-capability authority safety: https://doi.org/10.1109/SP.2010.16
- constrained PRFs: https://crypto.stanford.edu/~dabo/pubs/abstracts/dumbledore.html
- Macaroons: https://www.ndss-symposium.org/ndss2014/ndss-2014-programme/macaroons-cookies-contextual-caveats-decentralized-authorization-cloud/
- Pythia: https://www.usenix.org/conference/usenixsecurity15/technical-sessions/presentation/everspaugh
- TOPPSS: https://doi.org/10.1007/978-3-319-61204-1_3
- SafetyPin: https://www.usenix.org/conference/osdi20/presentation/dauterman
- PolyScope: https://www.usenix.org/conference/usenixsecurity21/presentation/lee-yu-tsung
- MFKDF: https://www.usenix.org/conference/usenixsecurity23/presentation/nair-mfkdf
- MFKDF cryptanalysis: https://www.usenix.org/conference/usenixsecurity24/presentation/scarlata
- LaKey: https://www.usenix.org/conference/usenixsecurity24/presentation/geihs
- stateful least privilege: https://www.usenix.org/conference/usenixsecurity24/presentation/cao-leo
- SVR3: https://www.usenix.org/conference/osdi24/presentation/connell
- Flock: https://www.usenix.org/conference/osdi24/presentation/kaviani
- KSDAuth: https://doi.org/10.31881/TLR.2026.6000

## 4. Novelty verdict

### Defensible contribution

The work can be presented as a domain-specific security-analysis method that:

1. computes Root capability and context-indexed credential-output exposure from
   the same deployment closure;
2. proves that Root access dominates all context outputs while a Root-only
   threshold is incomplete below that point;
3. reports an approval-budget-indexed cardinality and cost spectrum rather than
   a single all-or-nothing threshold; and
4. gives exact projection-class exposure identities and automatically generated
   witnesses for under-bound authorization.

### Claims that must not appear

- new Shamir sharing, DPRF, OPRF, PRF, MAC, ticket, attack graph, fixed-point,
  minimum-cost path, least-privilege, or context-binding primitive;
- a four-quadrant independence theorem;
- proof that no similar unpublished or unindexed work exists;
- conclusions about a published system that were not obtained from a faithful,
  independently reviewed encoding; or
- production security from a finite symbolic model.

## 5. Scores and publication decision

| Dimension | Assessment |
|---|---:|
| Mathematical correctness after correction | 8/10 |
| Cryptographic-primitive novelty | 0/10 |
| Generic security-analysis novelty | 3/10 |
| Credential-derivation-specific method novelty | 6/10 |
| Current empirical evidence | 5/10 |
| Standalone-paper potential | 6/10 |
| Present submission readiness | 4/10 |

**Decision:** the corrected method is sufficiently coherent and differentiated
to justify a standalone paper draft.  It is not yet safe to claim strong
priority or a high acceptance probability.  Submission maturity requires
faithful executable encodings of published protocols or real deployment
profiles, independent review of those encodings, and evaluation beyond the
current controlled finite cases.  The paper should therefore be labelled a
research manuscript, use only published work in comparisons, and keep all
claims within the boundary above.
