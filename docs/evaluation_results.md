# Latest research evaluation results

Date: 2026-08-09

## Exact policy space

- SOUPS 2022 corpus: 121 policies translated exactly into the supported IR;
  120 complete within the fixed compilation budget.
- 96 of the 120 completed spaces fit the prototype backend's domain bounds.
- All 120 exceed illustrative 40- and 60-bit credential-space floors; 116
  exceed 80 bits. These are configurable profiles, not universal guidance.
- Warm complete EPSCD derivation median: 198.52 microseconds on the reported
  macOS x86-64 host.

## Published comparison

The only executable comparison in the new manuscript is the published
Dichopile regular-language generator (Oudinet, Denise, and Gaudel, TCS 2013).
On the enumerable 11-word language with 100,000 samples, its TVD is 0.00399;
the EPSCD test-permutation composition has TVD 0.00234. No speed-up or
superiority claim is inferred because their sequence semantics differ.

No unpublished local encoder, prior manuscript, or preprint artifact is used
as a paper comparison.

## Known-credential exposure

For q=0..5 known pairs in an ideal 11-element permutation, 100,000 conditional
samples per q produce support exactly 11-q and no known-output recurrence. TVD
from uniform on the remaining support ranges from 0.00213 to 0.00666. This is
an implementation consistency check, not concrete FF1 key-recovery evidence.

## Factor-preserving recovery

- Every one of the ten 3-of-5 node combinations reconstructs the same
  authenticated network share.
- Negative tests cover insufficient/duplicate approval, too few fragments,
  expiry, session-key rebinding, replay, wrong node, stale share-set, mixed
  generation, and ciphertext tampering.
- In 100 release-mode local iterations, two Ed25519 signatures take a mean
  369.42 microseconds; three encrypted node releases plus reconstruction take
  6.036 ms. Network and human approval latency are not measured.

## Verification

- Rust: 85 tests passed; strict all-target/all-feature Clippy passed.
- Lifecycle TLA+: 1,006,128 distinct states, depth 56.
- Recovery-access TLA+: 852,704 distinct states, depth 32.
- Integrated freshness/compromise TLA+: 40,292 distinct states, depth 16.

All final bounded runs exhausted their configured queues without a listed
invariant violation. See `formal/EXTENDED_MODEL_CHECK_RESULTS.md` for bounds and
non-claims.
