# MFDPG prior-art and artifact audit

Audit date: 2026-08-09  
Work audited: Vivek Nair and Dawn Song, “MFDPG: Multi-Factor
Authenticated Password Management With Zero Stored Secrets,” arXiv:2306.14746
(submitted 26 June 2023).  
Official artifact: <https://github.com/multifactor/mfdpg>, pinned at
`6c26096dd22ff2b18aa5d8e4c3d5b0caf7b45bb7`.

## Publication status

| Field | Verified finding |
|---|---|
| Venue | No peer-reviewed venue could be verified. The arXiv PDF prints “Network and Distributed System Security (NDSS) Symposium 2024” but also retains the placeholder DOI `10.14722/ndss.2024.23xxx`. MFDPG is absent from the official NDSS 2024 accepted-paper list. |
| Year | 2023 preprint. |
| Peer reviewed | Not established by the available primary sources. The manuscript must not cite it as an accepted NDSS 2024 paper. |
| Primary publication source | <https://arxiv.org/abs/2306.14746> |
| Venue cross-check | <https://www.ndss-symposium.org/ndss2024/accepted-papers/> |
| Artifact status | The authors' GitHub repository is public and was archived by its owner on 28 September 2025. |

The requested “MFDPG (NDSS 2024)” reference is therefore factually unsafe.
MFDPG remains the closest technical work and should be discussed as a preprint
with an official artifact. A comparison table may include it only if
publication status is explicit and the caption distinguishes it from formally
published baselines.

## Research design

| Topic | Paper or artifact finding |
|---|---|
| Research goal | A deterministic password manager derived from multiple factors, with no stored service-specific credential secret, broad regular-policy support, portability, privacy of service use, and password revocation. |
| Threat model | The argument considers compromise of persisted public parameters and service passwords, factor guessing, and deterministic regeneration. It does not give a modern game-based proof for the complete password generator; Section V-D describes semi-formal reductions to underlying components. Active endpoint compromise remains outside its protection. |
| Storage model | The artifact exports MFKDF public policy parameters and a fixed-capacity Cuckoo filter. It avoids a per-service password or visible service identifier; the filter contains revoked preimages mixed with deterministic fictitious entries. The design is not stateless. |
| Policy representation | A JavaScript regular expression supplied by the caller. The paper describes regex-to-NFA, subset construction to a DFA, and Xeger-style randomized traversal. The released artifact delegates generation directly to `randexp` rather than exposing a separately constructed DFA. |
| Deterministic generation | The service domain and a counter are processed with Argon2id under the derived MFKDF key. The first preimage not present in the filter seeds the regex generator. Reusing factors, state, service, and policy regenerates the same password. |
| Random generator | The paper specifies HMAC-DRBG. The pinned artifact uses `random-seed` 0.3.0 and assigns `intBetween` to `RandExp.randInt`. This is a paper/artifact difference, not evidence that either complete design is insecure. |
| Revocation and rotation | `revoke(domain)` finds the currently active, non-filtered Argon2id preimage and inserts it into the Cuckoo filter while removing one fictitious entry. A later `generate` increments the private counter until it finds a non-filtered preimage. The filter hides an explicit per-service counter but permits false-positive skips and has a fixed revocation capacity. |
| Multi-factor component | MFKDF derives a 16-byte master key from one or more supported factors. The paper also discusses MFKDF threshold factor recovery. |
| Compatibility evaluation | The preprint reports a survey of 45 deterministic password generators and a policy-compatibility study over 100 popular Web applications. These are contributions of MFDPG and must be acknowledged. |
| Public artifact | JavaScript package with `index.js`, 12 unit tests, benchmarks, and a lockfile. The upstream unit suite completes on the pinned commit under Node 22.23.1. |

## Properties claimed by MFDPG

The preprint explicitly claims or argues:

- deterministic regeneration for a fixed factor set, exported state, service,
  and policy;
- multi-factor derivation through MFKDF;
- no stored service password and no explicit service identifier in persisted
  state;
- output matching regular-expression policies;
- revocation without a user-maintained visible per-service counter;
- portability through exported public parameters;
- factor recovery when an appropriate threshold MFKDF policy is used.

MFDPG should not be described as a failed method. It tackles a broader
multi-factor password-management problem than EPSCD.

## Properties not established by the paper

The paper does not define:

- exact cardinality of the accepted password language;
- a canonical rank/unrank bijection for that language;
- a proof that one fixed version is uniform over every accepted word;
- a generation-indexed keyed permutation of the accepted language;
- a without-replacement credential sequence;
- structural non-repetition over revocations;
- policy epochs or cross-policy authenticated history exclusion;
- evidence semantics for committing a password change at a remote legacy
  service.

“Not established” is the correct comparison term. It does not mean impossible,
insecure, or broken.

## Source-derived artifact observation

For the exact regular expression `a|b[0-9]`, `randexp` parses two root
alternatives. Its `_randSelect` chooses an alternative with
`randInt(0, options.length-1)`, and its character-set branch chooses one of ten
digits in the same way. Under uniform integer draws, the artifact therefore
assigns probability `1/2` to `a` and `1/20` to each `b0` through `b9`, rather
than `1/11` to each accepted word. The analytic total-variation distance to the
uniform distribution is `9/22` (approximately 0.40909).

This is a property of the pinned artifact and one enumerable expression. It is
not a claim that every Xeger/DFA walk is non-uniform or that MFDPG is insecure.

## Empirical artifact results

The minimal distribution harness executed the locked `randexp` 0.5.3 and
`random-seed` 0.3.0 packages using the same `randInt` override as MFDPG. Ten
independently labelled seed batches produced 100,000 samples. Aggregate counts
were 50,052 for `a` and between 4,899 and 5,168 for each `b`-digit word; TVD was
0.40961, the Pearson statistic was 203,020.86 with 10 degrees of freedom, and
empirical min-entropy was 0.9985 bits. The labelled SHA-256 seeds replace the
Argon2id preimage stage, so this isolates output selection and is not an
end-to-end performance result.

A separate end-to-end harness created three genuine one-factor MFKDF instances
and called the official `generate`/`revoke` methods for 12 versions on the same
11-word language. The three sequences contained 5, 6, and 6 distinct values,
respectively (19 repeated outputs across 36 within-set versions). Because 12
draws exceed an 11-word space, at least one repetition is unavoidable for a
with-replacement mechanism. The purpose is only to confirm that the released
revocation procedure does not implement a permutation-based non-repetition
guarantee; the toy-domain counts do not model a real password attack.

Raw data and exact commands are under `experiments/mfdpg_official/`.

## Narrow difference from EPSCD

MFDPG solves the broader problem of factor-authenticated deterministic password
management with private revocation state. EPSCD studies a narrower sequence
property: it counts and canonically indexes the accepted finite language, maps
credential generation through one keyed permutation, and thereby obtains a
without-replacement sequence inside a policy epoch. Authenticated descriptors
extend this sequence across policy changes through bounded history exclusion,
while remote evidence controls local commit. This positioning does not claim
that regular expressions, automata, deterministic policies, revocation, or
multi-factor recovery were introduced by EPSCD.
