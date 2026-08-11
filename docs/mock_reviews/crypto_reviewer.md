# Mock cryptography review

## Summary assessment

The formal layer is considerably improved. Compliance and within-epoch
non-repetition are functional consequences of two bijections. The fixed-
generation uniform marginal and history-exclusion result are correctly limited
to an ideal random permutation, while context separation is computational.

## Theorems

Theorem 5 is correct under its revised conditions: the authenticated history is
deduplicated to a set, the scanned generation inputs are distinct, and the set
is fixed independently of the ideal permutation. The ordered permutation images
are a sample without replacement. Symmetry gives a uniform first admissible
rank, and the negative-hypergeometric expectation is
`(N+1)/(N-e+1)`. The `e+1` worst-case bound follows because only `e` excluded
ranks can precede success.

The cycle-walk equation is also correct with the stated convention:
`Pr[W>k]=(M-N)_k/(M)_k`, zero after the outside set is exhausted. The
power-of-two case and cap off-by-one are now handled explicitly.

## Cryptographic assumptions

The abstract `DomainPermutation` gives clean theorem statements, but the
concrete FF1 backend remains the weakest part. It is partial, enforces a draft
minimum domain, rejects domains above 512 bits, and caps cycle walking. The
ideal tail is not a concrete availability theorem. The paper also does not give
a quantitative reduction from the selected FF1 implementation to its ideal-
permutation games or analyze timing leakage from the walk count.

## History exclusion

The marginal is uniform for a fixed authenticated set, but this does not model
adaptive exclusion sets correlated with the fresh permutation, undocumented
server history, or unavailable old Root-Key generations. The manuscript states
these boundaries. It should retain them during copy editing.

## Recommendation

**Major revision / weak reject for a cryptography venue; potentially acceptable
for an applied security journal.** The proofs appear sound, but the primitive
construction itself is established and the concrete backend is not yet
proof-matched to the abstract interface. A stronger submission would provide a
reviewed total arbitrary-domain instantiation or a quantitative concrete
security and leakage analysis.
