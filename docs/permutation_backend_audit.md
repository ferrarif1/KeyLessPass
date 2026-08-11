# EPSCD Exact-Domain Permutation Backend Audit

Date: 2026-08-10

## Required contract

For domain size `N`, key `K`, tweak `T`, and input `g`, a backend must implement a deterministic invertible mapping on exactly `[0,N)`. EPSCD needs permutation semantics, not merely a PRF followed by reduction. The implementation must fail closed for unsupported domains and must never silently fall back to modulo reduction.

## Candidates

| Candidate | Status | Suitability |
|---|---|---|
| New custom Feistel/PRP | Rejected | Would create an unaudited cryptographic primitive and an indefensible novelty/safety claim. |
| Black--Rogaway integer-domain ciphers | Cryptographically relevant published construction | Strong prior art, but no mature, directly reusable implementation was identified in the current Rust dependency set. |
| Swap-or-Not | Published small-domain cipher with strong bounds | Attractive research backend, but no mature Rust implementation with the required API and review status was identified. Reimplementing it now would enlarge the cryptographic trusted base. |
| NIST FF1 over the smallest radix-2 superset plus cycle walking | Existing implementation and dependency | Selected. It preserves permutation semantics on the accepted subset and reuses the repository's tested `fpe`/AES implementation. |

Relevant primary sources are [Black and Rogaway's arbitrary-domain construction](https://www.cs.ucdavis.edu/~rogaway/papers/subset.htm), [Swap-or-Not](https://www.cs.ucdavis.edu/~rogaway/papers/shuffle.html), [Bellare et al.'s FPE analysis](https://eprint.iacr.org/2009/251.pdf), and the [NIST SP 800-38G Rev. 1 second public draft](https://csrc.nist.gov/pubs/sp/800/38/g/r1/2pd).

## Selected implementation

`rust_core/src/permutation/mod.rs` uses AES-256 FF1 on the smallest binary domain containing `[0,N)`, then repeats the permutation while the result is outside `[0,N)`. It enforces:

- minimum target domain: `1,000,000`;
- maximum target-domain bit length: `512`;
- 256-bit key;
- configurable positive walk limit, default `1,024`;
- input range checks;
- fail-closed behavior when the walk limit is exceeded.

## Correctness and claim limits

Without the implementation walk cap, cycle walking restricts a permutation on the binary superset to a permutation on `[0,N)`. Thus distinct generations in one lineage map to distinct ranks. The cap converts extremely long walks into explicit derivation failure; it does not preserve totality for every key/tweak/input tuple.

The backend therefore supports these claims:

- exact-domain output when derivation succeeds;
- invertibility when forward and inverse calls succeed;
- no same-lineage collision among successful outputs;
- rejection-free `Unrank` decoding after a rank has been obtained.

It does **not** support these claims:

- constant-time end-to-end derivation;
- zero rejection in the permutation layer;
- production side-channel resistance of the complete application;
- a worst-case finite success bound below the configured cap;
- a new FPE or small-domain cipher.

## Density analysis

Let `M = 2^ceil(log2 N)` and `alpha = N/M`. Under the ideal-permutation heuristic, each FF1 call accepts with probability `alpha`, so the expected walk count is `1/alpha` and the probability of exceeding `w` calls is approximately `(1-alpha)^w`. Because `M < 2N` for `N > 1`, the selected smallest-binary-superset construction has `alpha > 1/2`; sparse-density stress tests down to `10^-4` apply only to synthetic supersets or alternative experiments and must not be attributed to this deployed backend.

## Decision

Retain the existing FF1 bounded cycle-walking backend. Add inversion, injection, walk-count, cap-failure, and fixed-vector tests. Present it as an implementation choice inherited from published FPE techniques, not as EPSCD's innovation.
