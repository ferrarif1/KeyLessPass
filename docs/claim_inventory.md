# Claim inventory for the exposure-and-recovery revision

Inventory date: 2026-08-09  
Purpose: prevent proof, assumption, implementation, and measurement claims from
being conflated.

## A. Mathematical and functional claims

| ID | Claim | Status | Evidence/condition | Manuscript treatment |
|---|---|---|---|---|
| M1 | The compiler count `N_P` equals the number of words accepted by the supported bounded policy representation. | Retain | Dynamic-programming recurrence and compiler/property tests. | Exact for the implemented IR and resource-completed compilation only. |
| M2 | `Rank_P` and `Unrank_P` are inverse bijections. | Retain | Existing Lemma 1 and property tests. | Functional theorem; no PRP assumption. |
| M3 | Successful derivation is policy compliant. | Retain | Range of `Unrank_P`. | Functional theorem. |
| M4 | For fixed key, tweak, policy, and distinct inputs, successful outputs do not repeat. | Retain | Injectivity of the permutation and unrank. | This is non-repetition, not unpredictability. |
| M5 | Under an ideal random permutation, a fixed unobserved generation is marginally uniform over `L_P`. | Retain | Permutation symmetry. | Ideal-model statement only. |
| M6 | Conditioned on q distinct consistent generation/rank pairs, an unseen generation maps uniformly over the `N_P-q` unused ranks. | Add | `(N-q-1)!/(N-q)! = 1/(N-q)`. | Name as an observed-credential residual-uniformity proposition specialized to EPSCD. Explicitly identify it as standard random-permutation symmetry. |
| M7 | `log2(N_P)` is the exact entropy of an ideal uniform draw from the legal policy space. | Retain/clarify | Definition of finite uniform entropy. | Never call a fixed deterministic output an `H=log2 N_P` random variable. |
| M8 | No-repeat in deployment needs both permutation injectivity and freshness of the generation/context state. | Add | A valid rollback reuses an old permutation input. | System theorem/invariant, not a primitive property. |

## B. Computational claims

| ID | Claim | Status | Assumption/bound | Required wording |
|---|---|---|---|---|
| C1 | Known password/rank pairs do not reveal unseen ranks beyond computational PRP advantage. | Add | Security of the selected finite-domain PRP under known/adaptive query exposure and its concrete domain/query bounds. | Give a game hop from the real backend to an ideal random permutation. Do not call this information-theoretic secrecy. |
| C2 | Different credential contexts are computationally separated. | Retain | HKDF PRF and tweakable-PRP assumptions plus injective canonical context encoding. | Distinct contexts can still coincidentally output the same string. |
| C3 | One known service password does not by itself reveal `Kcred`. | Add | PRP key-recovery resistance, not merely one-wayness of HKDF. | The observation yields one known `(g, Rank(password))` pair. |
| C4 | q known service passwords do not expose another service/account sequence. | Add | Per-credential HKDF context separation plus PRP security. | Bound by credential-key and root-key compromise cases. |
| C5 | An authenticated session ticket cannot be altered or rebound without detection. | Add | EUF-CMA signature security and authenticated canonical ticket encoding. | Prototype claim only for the chosen signing implementation. |
| C6 | Network fragments are confidential in transit to the ephemeral recovery client. | Add | Public-key encryption/KEM-DEM security and authenticated session binding. | Encryption does not replace approval. |

## C. Information disclosed by each exposure level

| Exposure | What the adversary obtains | What remains protected under assumptions | Mandatory consequence |
|---|---|---|---|
| Public policy/context | `P`, `L_P`, `N_P`, rank/unrank, identifiers, salts, generations, hashes, scheme versions | Permutation key and generation outputs | The adversary can enumerate `L_P`; secrecy is not based on hiding the language. |
| One service password | One pair `(g, Rank_P(password_g))` | Unseen ranks under PRP security | Injectivity alone is insufficient for this claim. |
| q service passwords in one context | q distinct input/output pairs | Unseen outputs computationally match a draw without replacement from `N_P-q` values | Ideal and real statements must be separated. |
| Server verifier compromise | Policy, salt, hash parameters, and verifier | Nothing prevents direct enumeration of `L_P` against the verifier | A 256-bit Root Key does not imply 256-bit service-password guessing strength. |
| `Kcred` compromise | Full current credential permutation for every public tweak usable with that key | Other credential keys and Root Key under HKDF assumptions | Current `policyEpoch` change does not repair compromise. A new credential-key lineage is required. |
| `Kroot` compromise | Every credential key and password derivable in that Root-Key generation | Future credentials only after true Root-Key replacement and remote rotation | Re-sharing the same Root Key is not remediation. |
| Endpoint compromise while secrets are available | Accessible local share, Root Key/Kcred/passwords in memory, local tokens and callable authority | Independent offline/network/approval domains only if their capabilities are absent from closure | Platform secure storage is hardening, not an exception to the compromise model. |

## D. Recovery claim boundary

| Claim | Proposed status | Boundary |
|---|---|---|
| Standard top-level 2-of-3 Shamir recovery | Component, not innovation | Shamir 1979 and MFKDF already cover threshold recovery concepts. |
| 3-of-5 storage of `S_N` | Availability/security mechanism, not innovation | Threshold storage alone does not preserve the deployment threshold. |
| Independent release authorization A | System design element, not a new factor or primitive | A signs release authorization; it holds no Root-Key share. |
| Deployment capability closure | Candidate systems abstraction | Must be presented as a scoped analysis method unless a broader search establishes stronger novelty. |
| Factor preservation | Candidate property | For each single protected domain X, `|Closure(X) intersect {S_D,S_U,S_N}| < 2`. Assumes D/U/N/A administration is actually independent. |
| Ordinary recovery re-share | Lifecycle property | Same `rootGeneration`; increment `shareSetGeneration`; stale old shares rejected. |
| Root compromise recovery | Lifecycle property | Replace `Kroot`, increment `rootGeneration`, install a new share set, and rotate affected remote credentials. |

## E. Empirical claims

Existing measurements retained without rerun:

- 120 of 121 exact corpus translations compile under the declared budget; one
  times out.
- warm complete EPSCD median is 198.52 microseconds on the recorded host;
  cold median is 7.806 ms.
- 96 corpus policies fit the prototype FF1 domain bounds; measured cycle walks
  have median 1, P95 3, P99 5, and maximum 9 over 3,072 observations.
- the pinned MFDPG artifact result is preprint-artifact evidence, not a
  peer-reviewed experimental comparison.
- the existing rotation model has 1,006,128 distinct states and no reported
  invariant violation within its stated bounds.

New empirical claims require new artifacts:

- q-exposure toy support checks;
- credential-space floor classification from existing raw corpus counts;
- adversarial recovery release tests;
- multidimensional rollback tests;
- recovery availability/latency with approval delay separated from crypto and
  transport.

## F. Forbidden formulations

- “Exposure theorem proves FF1 cannot be broken.”
- “Injectivity makes future passwords unpredictable.”
- “Policy epoch rotation repairs a leaked credential key.”
- “Re-sharing repairs a leaked Root Key.”
- “3-of-5 network storage prevents one endpoint from obtaining the network share.”
- “A is a fourth recovery share.”
- “256-bit Root Key gives every service password 256-bit security.”
- “Factor preservation is the first such construction” before a conclusive
  prior-art search.

