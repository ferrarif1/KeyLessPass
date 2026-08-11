# Final submission revision report

Date: 2026-08-09  
Manuscript: *Exact Policy-Space Credential Derivation for Legacy Password
Rotation*

## 1. MFDPG positioning

MFDPG is now discussed in the introduction, related work, the descriptive
property table, and the artifact evaluation. Its contributions are stated
positively: multi-factor deterministic derivation, regular-expression policy
support, no stored service password, portable public state, private revocation,
and factor recovery through MFKDF.

The requested citation as “NDSS 2024” was not used. The arXiv PDF contains a
placeholder NDSS DOI, and MFDPG is absent from the official NDSS 2024
accepted-paper list. It is cited as an arXiv 2023 preprint. This prevents an
unverifiable venue claim while retaining the closest technical work.

## 2. Official artifact reproduction

The official repository is pinned at
`6c26096dd22ff2b18aa5d8e4c3d5b0caf7b45bb7`. The lockfile, Node/npm versions,
operating system, archived status, dependency versions, and commands are
recorded under `experiments/mfdpg_official/`. The upstream unit command reports
12 passing tests.

Source inspection identifies a paper/artifact difference: the preprint
describes HMAC-DRBG, whereas the released `index.js` uses `random-seed` 0.3.0
to supply `randexp` 0.5.3 integer choices. The manuscript reports this as an
implementation fact, not a vulnerability claim.

## 3. MFDPG and EPSCD observations

For the exactly enumerable expression `a|b[0-9]`, the pinned output-selection
code chooses the two alternatives uniformly and then one of ten digits in the
second branch. Its source-derived distribution is `Pr[a]=1/2` and
`Pr[bi]=1/20`, with TVD `9/22` from uniform over the 11 accepted words.

The minimal artifact harness uses ten labelled seed batches and 100,000 total
samples. It records TVD 0.40961, chi-square 203,020.86 (10 degrees of freedom),
a 10.217 max/min ratio, and empirical min-entropy 0.9985 bits. The harness
executes the exact locked selection dependencies but replaces Argon2id
preimages with labelled SHA-256 seeds; it is not an end-to-end latency result.

The separate end-to-end harness calls the official `generate` and `revoke`
methods with three real MFKDF factor sets. In three 12-version sequences over
the 11-word language, 17 outputs are distinct within their respective sets and
19 are repeats. The experiment only tests whether the artifact provides a
structural no-repeat sequence. It is not extrapolated to realistic password
security.

## 4. Full 121-policy compilation

The importer no longer emits an experiment-only `Lmax <= 32` eligibility flag.
Every exact translation is passed to an isolated worker under a budget fixed
before execution:

- maximum reachable states: 250,000;
- maximum sampled resident memory: 4 GiB;
- maximum wall time: 60 seconds per policy;
- length prefilter: none.

The 270 source records yield 121 exact translations and 149 semantic
rejections. Of the 121 compilation attempts, 120 complete and one reaches the
wall-time limit. The timed-out record is source row 31, length 8--50, with a
sampled resident set near 2.0 GiB. No attempt reaches the state or memory
limit. Completed cases include maximum length 128.

Across 120 completions:

| Metric | Median | P95 | Maximum |
|---|---:|---:|---:|
| Reachable states | 65 | 284 | 23,227 |
| Compile time | 55.90 ms | 904.38 ms | 9.43 s |
| Count payload | 493.4 KiB | 19,201.6 KiB | 75,590.4 KiB |
| Sampled peak RSS | 5.6 MiB | 55.7 MiB | 381.6 MiB |
| Per-policy rank median | 24.93 us | 109.30 us | 137.03 us |
| Per-policy unrank median | 37.34 us | 137.65 us | 163.88 us |

`resource-skipped` no longer appears in the manuscript or new corpus result
schema.

## 5. Cold and warm timing

The performance experiment now distinguishes method-specific cold and warm
operations over 500 samples on the same policy:

| Operation | Median | P95 | P99 | SD |
|---|---:|---:|---:|---:|
| EPSCD cold compile and derive | 7.806 ms | 8.507 ms | 9.566 ms | 0.755 ms |
| EPSCD warm cached derive | 198.52 us | 332.16 us | 425.16 us | 55.94 us |
| Dichopile cold DFA/init/generate | 13.794 ms | 15.085 ms | 17.141 ms | 2.609 ms |
| Dichopile warm cached generate | 5.626 ms | 6.253 ms | 7.005 ms | 0.568 ms |

Cold EPSCD constructs the DFA and full count table. Cold Dichopile constructs
the common DFA, its length weights, and one output. The paper explicitly avoids
a speed-up claim because the algorithms provide different semantics and the
reproduction uses exact integers rather than the published optimized numerical
configuration.

## 6. Full-corpus cycle-walk observations

Of 120 compiled policies, 96 lie within the concrete backend's one-million to
512-bit domain interval. The other 24 exceed 512 bits and are recorded as
backend-domain-limit cases rather than compiler failures. Thirty-two
generations per supported policy yield 3,072 walk observations:

- mean 1.505;
- median 1;
- P95 3;
- P99 5;
- maximum 9;
- cap hits 0.

Domain density ranges from 0.516 to 0.973. A new paper figure plots density
against each policy's mean walk count and overlays the large-domain ideal
reference. The manuscript states the exact ideal expectation `(M+1)/(N+1)`.

## 7. Formal analysis

Equation (7) was independently rederived. For `W` primitive calls and a random
permutation on a binary superset of size `M`,

`Pr[W > k] = (M-N)_k / (M)_k`.

The numerator is zero when `k > M-N`; a power-of-two domain has `W=1`. A cap at
1,024 fails on the event `W > 1024`. The ideal tail is not presented as an
unconditional FF1 availability guarantee.

Theorem 5 now defines history as a deduplicated set, fixes authenticated
history independently of the ideal permutation, and requires distinct scanned
generation inputs. The appendix proves ordered sampling without replacement,
symmetry over the complement, the `e+1` attempt bound, and expectation
`(N+1)/(N-e+1)` using a tail sum and the hockey-stick identity.

## 8. Abstract and style revision

The abstract is 230 words. It no longer lists the source-record total, the
2,000-generation implementation check, the exact model state count, or the
number of invariants. It retains only the full translated-corpus outcome and
warm derivation latency as quantitative anchors.

The manuscript was searched for project-report and templated phrases including
“all tests passed,” “all policies,” “all invariants,” “these results establish,”
“Importantly,” “Notably,” and repeated “we do not claim” constructions. The
remaining text states assumptions and evidence directly. The conclusion is one
paragraph centered on the sequence property and the central remaining backend
and deployment limitation.

## 9. Fairness of the comparison table

The former checkmark/property matrix is replaced by descriptive columns:
generation model, policy model, space characterization, sequence semantics,
and rotation semantics. Missing paper claims are written as “not specified.”
MFDPG is visibly labelled `arXiv 2023`; the caption explains that it is included
as the closest preprint rather than a peer-reviewed baseline.

## 10. References and evidence classes

The bibliography adds the verified arXiv MFDPG record without inventing a
venue or DOI. The evaluation distinguishes:

- MFDPG: closest deterministic-system preprint and official artifact;
- Dichopile: published uniform regular-language algorithmic baseline;
- mathematical oracles: supplementary controls;
- EPSCD: proposed credential-sequence construction.

## 11. Remaining issues

- No production target adapter or real remote fault-injection experiment.
- No Windows/Linux timing or current enterprise policy sample.
- One exact translation times out under the fixed budget.
- Twenty-four compiled corpus policies exceed the current FF1 backend ceiling.
- The concrete permutation remains a bounded research backend rather than a
  proof-matched total arbitrary-domain implementation.
- TLA+ verification is bounded and abstracts cryptography, storage code,
  lockout timing, and the network.

## 12. Claims that remain unavailable

The manuscript cannot claim a new automata, counting, rank/unrank, uniform
regular-language generation, rank-then-encipher, FPE, KDF, MAC, or
secret-sharing primitive. It cannot claim universal policy compatibility,
production readiness, unconditional uniformity for concrete FF1, or
cross-epoch non-repetition without authenticated history. It also cannot cite
MFDPG as an NDSS 2024 publication on the evidence currently available.
