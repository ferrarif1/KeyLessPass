# Mock editor review

## Scope

The manuscript fits an applied information-security journal: it addresses
deterministic credential management for password-only services, combines a
formal sequence construction with lifecycle semantics, and includes a working
artifact and systems evaluation. The subject is narrower than a general
password manager and broader than a pure FPE paper.

## Is novelty assessable?

Yes. The revised manuscript concedes that exact regular-language counting,
rank/unrank, uniform generation, and rank-then-encipher are prior art. The
remaining claim is specific: credential generation indexes one keyed
permutation of the accepted policy space, with authenticated history exclusion
and evidence-bounded remote commit. Reviewers can now judge that specialization
without first correcting primitive-level novelty claims.

## Is the closest prior art present?

The main deterministic generators, verified random password generation,
uniform regular-language generation, arbitrary-domain FPE, MFKDF, and MFDPG are
present. MFDPG is accurately marked as an arXiv preprint because the claimed
NDSS venue cannot be verified. This is preferable to omitting the closest work
or assigning it a false venue.

## Editorial decision

**Send to review, with a substantial risk of major revision.** The submission
is now coherent enough for specialist assessment and the full-corpus experiment
removes the most obvious selective-evaluation concern. Desk rejection remains
plausible at a venue demanding a wholly new primitive: the algorithmic core is
a careful specialization of established rank-then-encipher machinery, and the
systems side lacks production adapters. JISA-like applied scope makes external
review defensible, but acceptance would require reviewers to value the sequence
and lifecycle formulation as more than engineering composition.
