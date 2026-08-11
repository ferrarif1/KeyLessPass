# Pinned MFDPG official artifact study

Publication status was checked on 2026-08-09. The PDF at arXiv:2306.14746
prints “NDSS Symposium 2024” together with the placeholder DOI
`10.14722/ndss.2024.23xxx`, but MFDPG does not appear in the official NDSS 2024
accepted-paper list. It is therefore cited as a 2023 arXiv preprint, not as a
peer-reviewed NDSS paper.

Pinned artifact:

- repository: `https://github.com/multifactor/mfdpg`
- commit: `6c26096dd22ff2b18aa5d8e4c3d5b0caf7b45bb7`
- upstream status: archived by its owner on 2025-09-28
- lockfile: `upstream/package-lock.json`, lockfile version 3
- measured runtime: Node 22.23.1, npm 10.9.8, macOS 26.5.2 x86-64
- relevant locked packages: `randexp` 0.5.3, `random-seed` 0.3.0,
  `hash-wasm` 4.9.0, `mfkdf` 1.4.6, `bloom-filters` 3.0.0

Reproduction:

```sh
git clone https://github.com/multifactor/mfdpg.git experiments/mfdpg_official/upstream
git -C experiments/mfdpg_official/upstream checkout \
  6c26096dd22ff2b18aa5d8e4c3d5b0caf7b45bb7
npm --prefix experiments/mfdpg_official/upstream ci --ignore-scripts
npm --prefix experiments/mfdpg_official/upstream run unit
node experiments/mfdpg_official/distribution_harness.js \
  > experiments/mfdpg_official/distribution_results.json
node experiments/mfdpg_official/rotation_harness.js \
  > experiments/mfdpg_official/rotation_results.json
```

`distribution_harness.js` executes the exact locked output-selection
dependencies and MFDPG's `randInt` override. It substitutes labeled SHA-256
seeds for Argon2id preimages to isolate the selection mechanism at 100,000
samples. `rotation_harness.js` separately exercises the public `generate` and
`revoke` methods end to end with real MFKDF factors and Argon2id.
