# Mock systems-security review

## Real-policy evidence

Running every exact translation under a fixed per-policy budget is a material
improvement. The result is informative rather than uniformly favorable: 120
compilations complete, one times out near 2 GiB sampled RSS, and 24 completed
spaces exceed the FF1 backend's 512-bit ceiling. Per-policy raw outcomes and
budget enforcement are reproducible. The corpus is nevertheless historical
website data, not a contemporary enterprise deployment sample.

## Scalability and performance

Cold and warm paths are now separated. The comparison is reasonably transparent:
both cold paths begin from the same policy IR, but EPSCD constructs a full count
table whereas Dichopile constructs its own generator state. The manuscript
correctly avoids a speed-up claim against the published optimized Dichopile
configuration. Single-host macOS results do not establish cross-platform
behavior, allocator sensitivity, or sustained multi-account workloads.

## Remote rotation

The `NewOnly`, `OldOnly`, `Both`, and `Neither` observations address an important
legacy-service ambiguity, and the commit rule is defensible. However, there are
no production adapters, real lockout budgets, replicated authentication
backends, or crash/network fault-injection results. The state machine is more
mature than its deployment evidence.

## Reproducibility

The source corpus digest, converter, isolated worker, fixed budgets, raw JSON,
pinned MFDPG artifact, exact Dichopile transcription, performance operation
definitions, TLA+ model, and commands are present. Peak RSS is sampled and may
miss short spikes; that limitation should remain visible.

## Recommendation

**Major revision.** The evaluation is now fair enough to assess, and the full-
corpus run supports prototype feasibility. Acceptance would be more convincing
with two real service adapters, crash and network fault injection, Windows and
Linux runs, and a current policy sample. For JISA, the paper could be accepted
after revision if reviewers regard the formal sequence semantics as sufficient
algorithmic contribution; for a top systems-security venue, the deployment
evidence is still too limited.
