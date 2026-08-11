# Reproducing the EPSCD Artifact

## Prerequisites

- Rust/Cargo compatible with the crate's declared toolchain;
- Python 3 standard library;
- Java 21 or a compatible JVM;
- `tmp/tla2tools-v1.7.4.jar`.

## One command

From the repository root:

```sh
./scripts/clean_reproduce.sh
```

The full public-corpus pass gives each translated policy a 60-second limit, so
the command can take several minutes. It writes JSON and CSV results to
`artifact/results/` and publication-neutral generated tables/plots to
`artifact/generated/`. It does not require or build an ignored manuscript.

## Result provenance

- `policy_corpus.*`: `experiments/real_policy_corpus/policy_metrics.json`;
- `rejection_density.*` and `sequence.*`: `experiments/epscd_mainline.json`;
- `rotation_faults.*` and `adapters.json`: `experiments/epscd_rotation/fault_matrix.json`;
- `permutation.json`: `experiments/performance/walk_corpus.json`;
- `distribution.json` and `performance.json`: the main policy-space evaluation;
- `tla.json`: parsed from the main and negative-control TLC logs.

Timing values are host-specific prototype measurements. Formal results are bounded exhaustive checks of the abstract model, not a proof of the implementation or complete protocol security.
