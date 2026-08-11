# Reproducibility

## Toolchain

- Rust toolchain pinned by `rust-toolchain.toml`;
- exact transitive versions in `rust_core/Cargo.lock`;
- Shamir implementation: `vsss-rs 5.4.0`;
- canonical JSON: `serde_json_canonicalizer 0.3.2`;
- TLA+ TLC 2.19, with JAR digest recorded in the model-check result files.

## Code verification

```bash
cd rust_core
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-targets --all-features
cargo check --all-targets --features research,peer-recovery
```

## Research experiments

```bash
# Published SOUPS 2022 policy corpus and exact spaces
cargo build --release --example compile_policy_worker
cd ..
python3 experiments/real_policy_corpus/run_full_corpus.py
python3 experiments/scripts/recompute_space_adequacy.py

# EPSCD and published Dichopile baseline
cd rust_core
cargo run --release --example policy_space_evaluation
cargo run --release --example walk_corpus

# Known-credential ideal-permutation consistency check
cargo run --release --example known_credential_exposure

# Local recovery cryptographic baseline; no network or human latency
cargo run --release --features peer-recovery --example peer_recovery_experiment -- \
  ../experiments/results/factor-recovery-quick-2026-08-09
```

The public-corpus importer records the source URL, workbook SHA-256, source
row, original PCP JSON, translation decision, and rejection reason. The compiler
runs each translated policy in an isolated worker with predeclared state,
memory, and wall-time limits and no length prefilter.

The optional MFDPG artifact probe keeps its harness and recorded output in the
repository but excludes the pinned upstream checkout and `node_modules`.
`experiments/mfdpg_official/README.md` records the exact clone, commit, install,
and execution commands.

## ASTER artifact

```bash
./research/aster/scripts/reproduce_all.sh --quick
./research/aster/scripts/reproduce_all.sh --full
```

The quick target runs deterministic semantic, capability, migration, adapter,
and bounded-model checks. The full target additionally regenerates the policy
corpus, OpenLDAP interoperability result, scalability measurement, summaries,
tables, and provenance map. MP-SPDZ fixed-vector runs are deliberately separate
because each execution is expensive; see `research/aster/mpc/README.md`.

Manuscript sources, rendered figures, and journal submission bundles are not
part of the versioned reproducibility boundary. They are excluded by
`.gitignore`; machine-readable inputs, scripts, raw results, and model files
remain versioned.

## Model checking

```bash
cd formal
java -cp ../tmp/tla2tools-v1.7.4.jar tlc2.TLC \
  -config lifecycle.cfg lifecycle.tla
java -cp ../tmp/tla2tools-v1.7.4.jar tlc2.TLC \
  -config recovery_access.cfg recovery_access.tla
java -cp ../tmp/tla2tools-v1.7.4.jar tlc2.TLC \
  -config integrated_model.cfg integrated_model.tla
```

Recorded results:

- lifecycle: 3,129,888 generated, 1,006,128 distinct, depth 56;
- recovery access: 6,145,889 generated, 852,704 distinct, depth 32;
- integrated freshness/compromise: 287,151 generated, 40,292 distinct,
  depth 16.

All three runs exhausted their configured state spaces with no listed invariant
violation. Exact versions, bounds, seeds, and assumptions are recorded in
`formal/MODEL_CHECK_RESULTS.md` and `formal/EXTENDED_MODEL_CHECK_RESULTS.md`.

## Measurement boundary

Raw JSON records sample counts, seeds, operation boundaries, and result units.
The recovery experiment measures local signing, encryption, share release, and
reconstruction only. It does not estimate network RTT or human approval delay.
The artifact contains no human-subject usability result, target-service adapter
benchmark, Windows/Linux performance result, or production freshness-service
measurement.
