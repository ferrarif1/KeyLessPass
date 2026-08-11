# Hostile editor review — latest manuscript

## Recommendation

**Borderline send to review; high risk of desk rejection if the editor expects
one mature, deployed system contribution.**

The paper is now coherent: both axes ask how deterministic credentials retain
their intended security semantics across rotation, exposure, recovery, and
rollback. The title matches that scope, the abstract is restrained, and the
paper separates standard primitives from its specialization. It no longer uses
unpublished local work or preprints as comparison objects.

## Reasons to send

- The generation-indexed accepted-space sequence has assessable definitions,
  proofs, implementation, a published baseline, and a published policy corpus.
- The recovery section identifies a concrete deployment failure not expressed
  by the algebraic Shamir threshold and gives an executable countermeasure.
- Compromise and freshness semantics connect the two axes instead of appending
  an unrelated recovery feature.

## Reasons to desk reject

- The ingredients of both axes are established; the editor may judge the
  remaining contribution as careful protocol specialization rather than a
  sufficiently new method.
- The recovery system has no real transport, enterprise approver integration,
  human study, fault injection, or multi-host experiment.
- The manuscript is 31 preprint pages and may appear too broad for the amount
  of deployment evidence.
- The concrete FF1 backend is bounded and not the proof-matched arbitrary-domain
  implementation suggested by the generic construction.

## Required pre-submission decision

If a real multi-host recovery deployment and at least one target adapter cannot
be added, keep the current limitation language and submit as a research
prototype. Do not promote factor preservation to a new cryptographic primitive
or claim first-of-kind recovery.
