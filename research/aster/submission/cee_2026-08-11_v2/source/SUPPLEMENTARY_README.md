# ASTER supplementary research artifact

This archive accompanies the manuscript **ASTER: Exact-Scope Threshold Credentials with Root-Epoch Healing for Legacy Password Systems**.

## Contents

- `research/aster/`: paper-side research materials, formal models, fixed test vectors, experiment scripts, evidence, and generated figures/tables.
- `rust_core/`: Rust implementation used by the evaluated prototype.
- `experiments/real_policy_corpus/`: policy-corpus inputs and supporting experiment material.

The submission package itself is excluded to prevent recursive archives. Rust build outputs, editor caches, and transient rendering files are also excluded.

## Reproduction entry points

Start with the reproduction and artifact-boundary documents under `research/aster/`. Use the quick reproduction path before the full experiment path. Dependency versions and environment assumptions are recorded alongside the scripts and evidence.

## Scope

The artifact supports the claims made in the submitted manuscript. It is a research prototype, not a production credential service. Do not deploy it with live credentials or secrets.
