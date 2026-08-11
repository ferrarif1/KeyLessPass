# MFDPG preprint-artifact observation

MFDPG is available as arXiv/CoRR 2306.14746 and an official archived repository. As of 2026-08-09, its manuscript still contains the placeholder DOI `10.14722/ndss.2024.23xxx`, it does not appear on the official NDSS 2024 accepted-paper list, and DBLP lists only the preprint. It is therefore **not a main published baseline**. This directory preserves a supplementary artifact-specific observation because MFDPG remains technically close and has an official implementation.

Pinned inputs:

- repository: `https://github.com/multifactor/mfdpg`
- commit: `6c26096dd22ff2b18aa5d8e4c3d5b0caf7b45bb7`
- lockfile version: 3
- `randexp`: 0.5.3
- `random-seed`: 0.3.0
- measured environment: Node 22.23.1, npm 10.9.8

Reproduction:

```sh
git clone https://github.com/multifactor/mfdpg.git /tmp/mfdpg
git -C /tmp/mfdpg checkout 6c26096dd22ff2b18aa5d8e4c3d5b0caf7b45bb7
npm --prefix /tmp/mfdpg ci --ignore-scripts
npm --prefix /tmp/mfdpg run unit
NODE_PATH=/tmp/mfdpg/node_modules node \
  experiments/mfdpg/official_distribution_probe.js \
  100000 10 epscd-mfdpg-20260809
```

The upstream unit suite passes 12 tests. Its aggregate `npm test` is not used because the pinned repository's lint stage stops on an unused variable in `benchmark/benchmark.js` before executing the unit suite.

The probe uses the exact locked `randexp` and `random-seed` packages and the same `randInt` replacement used by `MFDPG.generate`. It exercises ten independently labeled batches on the enumerable regular expression `a|b[0-9]`. It replaces the expensive Argon2id preimages with SHA-256-labeled deterministic seeds, so the result isolates output selection and is not an end-to-end latency result.

Permitted wording:

> In the tested policy, the released preprint artifact's output-selection dependencies do not weight alternatives by exact completion count.

Forbidden wording includes “MFDPG is broken,” “MFDPG is insecure,” or any implication that this experiment is a peer-reviewed-system benchmark.
