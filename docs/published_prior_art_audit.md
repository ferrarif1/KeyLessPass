# Published Prior-Art and Novelty Audit

Status: completed submission audit for EPSCD scheme version 1  
Cut-off checked: 2026-08-09  
Rule: a work is a **main experimental baseline** only if its publication and the exercised implementation can both be identified. Preprints, specifications, and mathematical controls are labeled separately.

## 1. Immediate corrections

### MFDPG is not verified as a published NDSS paper

The public MFDPG manuscript contains an NDSS 2024 cover line but its DOI is still the placeholder `10.14722/ndss.2024.23xxx`. The official [NDSS 2024 accepted-paper list](https://www.ndss-symposium.org/ndss2024/accepted-papers/) contains no MFDPG entry, DBLP indexes it only as CoRR/arXiv 2306.14746, and no record was found for the previously used DOI `10.14722/ndss.2024.24646`. Therefore:

- MFDPG must be cited as a **2023 preprint**, not “NDSS 2024”.
- Its official repository can support an artifact-specific supplementary experiment.
- It must not occupy a row labeled “published baseline” or be the sole SOTA comparator.

### Uniform policy-compliant password generation is already published

Grilo et al. (iFM 2022) specify a password generator, prove policy compliance in EasyCrypt, formalize uniform sampling over the induced password set, and integrate a corresponding implementation with Bitwarden. EPSCD must not claim to be the first policy-compliant or uniformly distributed password generator.

### Regular-language uniform generation and rank-then-encipher are prior art

Hickey and Cohen, Goldwurm, Denise and Zimmermann, Bernardi and Gimenez, and Oudinet et al. establish uniform generation over formal languages with several time/space trade-offs. Bellare et al. explicitly analyze rank-then-encipher FPE and discuss regular-language ranking. EPSCD must not claim a new DFA, counting recurrence, rank/unrank method, uniform regular-language sampler, FPE, or cycle-walking construction.

## 2. Work-by-work audit

| Work | Venue/year | Peer reviewed? | Public artifact? | Closest property already established | What it solves | What it does not claim in the reviewed source | Experiment status |
|---|---|---:|---:|---|---|---|---|
| Ross et al., PwdHash | USENIX Security 2005 | Yes | Historical source/extension reported | Site-separated deterministic passwords without server changes | Browser-mediated phishing resistance and site-specific derivation | Exact accepted-space count, uniform full-policy sampling, without-replacement generations | Literature only |
| Halderman et al., Password Multiplier | WWW 2005 | Yes | Historical implementation reported | Deterministic site passwords | Convenient per-site password management | Exact regular-policy language or permutation sequence | Literature only |
| Horsch et al., PALPAS | ARES 2015 | Yes | Source availability not yet verified | High-entropy secret plus per-site public salts; parameter synchronization | Passwordless password synchronization and service separation | Exact ranking or generation-indexed without-replacement rotation | Literature only |
| Horsch et al., Password Requirements Markup Language | ACISP 2016 | Yes | Open paper; implementation status not verified | Machine-readable password-policy representation | Communicating composition rules to clients | Keyed deterministic generation sequence | Literature only |
| Stajano et al., PMF | PASSWORDS 2014 revised proceedings | Yes | Not verified | Semantic annotations for password managers | Better website/manager interoperability | Exact language indexing or no-repeat sequence | Literature only |
| Gautam, Lalani, Ruoti | SOUPS 2022 | Yes | Paper, libraries, and 270-policy workbook | Public password-composition policy language and corpus | Realistic policy representation, prototype adoption, policy analysis | Keyed deterministic sampling without replacement | Corpus importer implemented; 270 records audited |
| Alroomi and Li | ACM CCS 2023 | Yes | Aggregate results; raw corpus not used here | Large-scale website-policy measurement | External evidence on policy diversity at more than 20,000 sites | Exact generator or reusable raw policy corpus in this artifact | Literature only |
| Grilo et al. | iFM 2022 | Yes | EasyCrypt/Jasmin/Bitwarden code linked | Verified policy compliance and uniform password sampling | Formally verified random password generation for a bounded class policy | Deterministic service-specific sequence, keyed sampling without replacement, policy epochs | Official repository audited at commit `ceeb8988f87b0ac4b6826fc20af4f8acafb3c841`; property comparison only because the repository does not pin the sibling Jasmin/EasyCrypt revisions required for a faithful performance build |
| Hickey and Cohen | SIAM J. Computing 1983 | Yes | No current artifact identified | Uniform fixed-length generation from unambiguous languages | Count-weighted generation with preprocessing/time-space trade-offs | Credential derivation semantics | Literature only |
| Goldwurm | Information Processing Letters 1995 | Yes | No current artifact identified | Space-efficient uniform generation | Linear binary-space generation for algebraic languages | Credential derivation semantics | Literature only |
| Denise and Zimmermann | Theoretical Computer Science 1999 | Yes | No current artifact identified | Floating-point acceleration for uniform combinatorial generation | Controlled near-uniform generation and complexity reduction | Credential derivation semantics | Literature only |
| Bernardi and Gimenez | Algorithmica 2012 | Yes | No public artifact verified | Linear expected-time regular-language sampling | Uniform fixed-length sampling with divide-and-conquer | Deterministic keyed sequence | Literature only |
| Oudinet, Denise, Gaudel | Theoretical Computer Science 2013 | Yes | Authors report Rukia C++ library; no maintained build artifact was required here | Dichopile uniform/near-uniform regular-language generation | Low-space generation of long paths in large automata | Keyed deterministic without-replacement credential sequence | Full 23-page paper read; Algorithm 1 reproduced with exact BigUint arithmetic and tested |
| Goldberg and Sipser | STOC 1985 | Yes | No | Ranking/compression foundations | Ranking formal-language objects | Password lifecycle | Literature only |
| Black and Rogaway | CT-RSA 2002 | Yes | No | Ciphers on arbitrary finite domains; cycle walking | General finite-domain encryption | Password policy or generation lifecycle | Literature only |
| Bellare et al. | SAC 2009 | Yes | Paper/ePrint | Formal FPE and rank-then-encipher | Security definitions, regular-format ranking, FPE constructions | Password-specific generation epochs/history | Literature only |
| NIST SP 800-38G Rev. 1 draft | NIST, 2025 draft | Standardization, not peer review | Specification | FF1 requirements and minimum domain | Concrete FPE interoperability/security requirements | New scientific contribution or credential lifecycle | Reference backend only |
| Durak and Vaudenay | CRYPTO 2017 | Yes | Paper | Small-domain FPE attacks | Security limits of FF3 on small domains | EPSCD policy compilation | Literature only |
| Nair and Song, MFKDF | USENIX Security 2023 | Yes | Distinguished Artifact | Threshold multi-factor KDF and client recovery | Multi-factor-derived keys and recovery | Exact policy-space password sequence | Supporting recovery context only |
| Scarlata, Backendal, Haller | USENIX Security 2024 | Yes | Paper | Cryptanalysis of MFKDF | State-integrity and factor-security weaknesses | Password-policy generation | Supporting caution only |
| Shamir | Communications of the ACM 1979 | Yes | Many standard implementations | Threshold secret sharing | Recovery of a random root key | Password derivation or lifecycle | Supporting mechanism only |
| Nair and Song, MFDPG | arXiv/CoRR 2023 | **No verified peer-reviewed venue** | Official repository pinned | Multifactor zero-secret DPG, regular-policy traversal, revocation | Broad password-management and policy objectives | Exact completion-weighted accepted-space distribution or permutation-indexed no-repeat theorem | Supplementary artifact probe only |
| Al Maqbali and Mitchell, AutoPass | arXiv 2017 | No | Not verified | Automatic password generation/change | Automated service password management | Published peer-reviewed evidence for exact indexing | Related preprint only |
| Spectre | Public specification | No | Public implementations | Site templates and counters | Deterministic site-scoped credentials | Peer-reviewed exact accepted-space guarantee | Specification context only |

## 3. Closest-property matrix

| Property | Published work already covering it | Safe EPSCD distinction |
|---|---|---|
| Policy-compliant generation | PwdHash policy encoding; PCP language; Grilo et al.; MFDPG preprint | Broader exact finite-language subset plus lifecycle binding, not first compliance |
| Uniform password sampling | Grilo et al. iFM 2022 | Deterministic keyed sequence without replacement, not first uniform generator |
| Uniform regular-language generation | Hickey-Cohen; Goldwurm; Denise-Zimmermann; Bernardi-Gimenez; Oudinet et al. | Fixed credential generations map through one keyed permutation |
| Exact counting and rank/unrank | Classical recursive/ranking literature; Goldberg-Sipser | Canonical public encoding and credential context, not new mathematics |
| Rank-then-encipher on a regular format | Bellare et al. | Password-rotation specialization and state semantics |
| Finite-domain/cycle-walking permutation | Black-Rogaway; Bellare et al.; NIST FF1 | Replaceable backend with explicit failure boundary |
| Service-separated deterministic passwords | PwdHash, PALPAS, and others | Adds exact policy indexing and an epoch-local without-replacement sequence |
| Password change/revocation | PwdHash discusses updates; AutoPass and MFDPG preprints address change/revocation | Explicit generation/epoch/history/remote-evidence state model, if fully implemented and model checked |
| Threshold recovery | Shamir; MFKDF | Standard supporting mechanism only |

## 4. Defensible novelty statement

The audit supports only a scoped systems-algorithm claim:

> The reviewed published literature separately establishes policy-compliant uniform password generation, uniform generation and ranking for regular languages, finite-domain rank-then-encipher, and deterministic service-specific passwords. We did not identify in the reviewed published sources a credential generator that binds an exact accepted-language index to a keyed, generation-indexed permutation so that one credential epoch forms a deterministic without-replacement password sequence, together with explicit policy-epoch and authenticated-history semantics.

This is a **prior-art-review conclusion**, not proof of global priority. The manuscript must use “to the best of our reviewed literature” and must not use “first ever,” “new FPE,” “new ranking algorithm,” or “new uniform generator.”

The no-repeat theorem is a useful formal property but mathematically elementary: it follows from permutation and unrank injectivity. Acceptance therefore depends on demonstrating that the credential state semantics, fail-closed boundaries, real-policy compatibility, and crash reconciliation create a meaningful system contribution beyond a straightforward application of rank-then-encipher.

## 5. Main-baseline eligibility

| Candidate | Main published baseline? | Decision |
|---|---:|---|
| Grilo et al. verified password generator | Property baseline | Published iFM 2022; official artifact audited but not performance-tested because its external proof/compiler revisions are not pinned |
| Oudinet et al. dichopile | Yes | Published TCS 2013; exact-arithmetic reproduction explicitly tied to Algorithm 1 |
| Gautam et al. PCP | Corpus/representation baseline | Published SOUPS 2022; use public workbook and exact translation rules |
| MFDPG official artifact | No | Keep only in a clearly labeled preprint-artifact supplement |
| Uniform random rank, identity permutation, hash modulo | No | Mechanistic controls only; never SOTA rows |

## 6. Remaining search risk

This audit searched the named password-generator, password-policy, formal-language sampling/ranking, FPE, rotation, and recovery lines of work. A final submission search should additionally cover patents and non-English literature, forward citations to Bellare et al. and Grilo et al., and 2025-2026 credential-generation work. Until that search is complete, priority language remains qualified.
