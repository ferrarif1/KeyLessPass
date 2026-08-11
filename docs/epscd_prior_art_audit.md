# EPSCD Prior-Art and Claim Audit

Date: 2026-08-10

## Search protocol

The audit used title, abstract, citation-chain, venue, and official artifact searches across deterministic password generation, policy-compliant generation, regular-language sampling, format-preserving encryption, credential managers, and legacy password rotation. Published works are separated from preprints and products. Absence from this search is not proof of global priority.

## Closest published work

| Work | Published contribution | Overlap with EPSCD | Boundary retained by EPSCD |
|---|---|---|---|
| Ross et al., PwdHash, USENIX Security 2005 | Site-specific deterministic passwords without server changes. | Deterministic service-specific credentials for legacy login. | Does not define an exact accepted policy language, exact-domain no-repeat generation sequence, or evidence-bounded remote rotation protocol. |
| Chiasson et al., Password Multiplier evaluation, USENIX Security 2006 | Evaluates a deterministic site-password tool; rotation changes a site-name argument. | Regenerable site-specific passwords and version-like rotation. | No exact policy-space bijection or failure-safe remote commit semantics. |
| Horsch et al., PALPAS, ARES 2015 | High-entropy secret and per-service salts, policy-aware password generation, synchronized non-secret metadata. | Non-stored service credentials and service-policy metadata. | No exact policy-space permutation sequence and no remote-evidence rotation state machine. |
| Ferreira et al., Verified Password Generation, iFM 2022 | Machine-checked policy compliance and uniform random generation, integrated into Bitwarden. | Formal policy-compliant generation and uniformity. | Random generation rather than a deterministic, keyed, no-repeat lineage sequence; no legacy remote rotation transaction. |
| Gautam et al., SOUPS 2022 | Password-composition policy description language, real policy corpus, generation libraries. | Policy representation and enterprise policy heterogeneity. | Does not provide keyed exact-domain sequence semantics or remote rotation evidence. |
| Bellare et al., FPE, SAC 2009 | General FPE definitions; rank-then-encipher and cycle-walking analysis. | Rank a complex finite domain, encipher the rank, unrank. | EPSCD does not claim this cryptographic construction; it instantiates and binds it to credential lineage and rotation metadata. |
| Black and Rogaway, CT-RSA 2002 | Provable ciphers on arbitrary-size integer domains. | Keyed permutation on `[0,N)`. | EPSCD does not claim arbitrary-domain ciphers as new. |
| Martínez et al., Dichopile, TCS 2013 | Exact counting and uniform random generation for regular languages. | Completion counts and exact decoding. | EPSCD does not claim counting or unranking as new. |

Official sources include the [PwdHash USENIX page](https://www.usenix.org/conference/14th-usenix-security-symposium/stronger-password-authentication-using-browser-extensions), the [Password Multiplier evaluation](https://www.usenix.org/legacy/events/sec06/tech/full_papers/chiasson/chiasson_html/index.html), the [SOUPS 2022 policy-language paper](https://www.usenix.org/conference/soups2022/presentation/gautam), the [Verified Password Generation paper](https://joaoff.com/publication/2022/iFM/iFM22-verifiedPwdGen.pdf), the [Black--Rogaway arbitrary-domain paper](https://www.cs.ucdavis.edu/~rogaway/papers/subset.htm), and the [Bellare et al. FPE paper](https://eprint.iacr.org/2009/251.pdf).

## Important non-comparison context

- AutoPass specifies site-policy-aware deterministic password generation and forced changes, but the located manuscript is a preprint rather than a confirmed peer-reviewed publication. It is discussed as prior context, not used as an experimental baseline.
- MFDPG is a 2023 preprint. It is mandatory related context but not described as a published baseline.
- HashiCorp Vault and LessPass are operational systems, not peer-reviewed comparison papers. Product documentation may motivate deployment differences but cannot support a paper-to-paper superiority claim.
- CETS is an independent unpublished research line in this repository. It is not a baseline, predecessor result, or comparison object in the EPSCD manuscript.

## Claim-by-claim novelty decision

### Contribution 1: Exact Policy-Space Credential Sequence

**Decision: PARTIAL GO.**

The mathematical components are prior art: bounded regular-language representation, exact completion counting, ranking/unranking, and rank-then-encipher. The defensible contribution is the credential-specific construction and contract that binds:

1. a canonical bounded policy and its exact cardinality;
2. an explicit credential lineage and policy epoch;
3. an audited keyed permutation on the exact rank domain;
4. generation numbers to a deterministic sequence with exact compliance, exact entropy, and strict same-lineage non-repetition.

The paper must call this a construction or method, not a new primitive, cipher, FPE mode, counting theorem, or regular-language algorithm.

### Contribution 2: Evidence-Bounded Failure-Safe Legacy Rotation

**Decision: GO, contingent on artifact evidence.**

The closest password generators describe how to compute or change a password, but the audit did not identify a published system that couples exact deterministic generations to a durable remote-evidence state machine for password-only targets and explicitly preserves both old and new reconstructability under uncertain update outcomes. This is a scoped literature finding, not a claim that no such system can exist.

The claim remains defensible only if the artifact provides adapter capabilities, fault injection, durable crash recovery, and a dedicated formal model showing that local generation commits require sufficient evidence.

## Prohibited novelty statements

The manuscript must not claim invention of:

- regular-language policy modeling;
- exact completion counting or ranking/unranking;
- uniform finite-language generation;
- format-preserving encryption, cycle walking, or arbitrary-domain PRPs;
- HKDF, HMAC, canonical JSON, Shamir sharing, or TLA+ verification;
- deterministic site-specific passwords or policy-aware password generation;
- generic sagas, write-ahead logging, idempotency, or evidence lattices.

## Overall verdict

**PARTIAL GO.** The research is worth completing as a two-contribution systems-security paper. Acceptance depends more on a precise claim boundary and fault evidence than on presenting the standard mathematical components as original.

