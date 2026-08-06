# Reproducibility

## Toolchain

- Rust toolchain pinned by `rust-toolchain.toml`; the 2026-08-06 run used `rustc 1.87.0` and `cargo 1.87.0` on macOS.
- Secret sharing: `vsss-rs 5.4.0`.
- Canonical JSON: `serde_json_canonicalizer 0.3.2`.
- Recovery dictionary: `bip39 2.2.2` English word list; the phrase format itself is KLRP v1.
- Exact transitive versions are in `rust_core/Cargo.lock`.

## Verification commands

```bash
cd KeyLessPass/rust_core
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-targets
cargo run --release --example research_evaluation -- ../experiments/results/quick-2026-08-06
KEYLESSPASS_FULL=1 cargo run --release --example research_evaluation -- ../experiments/results/full-2026-08-06
cd ../models
java -XX:+UseParallelGC -jar /path/to/tla2tools.jar -config PasswordRotation.cfg PasswordRotation.tla
```

Raw results are JSON and CSV under `experiments/results/`. Each result embeds the command, OS, architecture, package version, full/quick mode, and randomness description. Secret-sharing randomness comes from the OS CSPRNG. Password inputs use fixed CDR fields and `credentialGeneration = iteration + 1`. Timed loops report mean, median, P95, and population standard deviation; one-shot CDR bulk workloads report totals rather than invented distributions.

The current local verification completed 58 Rust tests with no failures and passed rustfmt and Clippy with warnings denied. TLC generated 17 states, found 16 distinct states to depth 6, exhausted the queue, and reported no invariant violation; exact model scope and tool version are recorded in `models/MODEL_CHECK_RESULTS.md`.

## Experiment coverage

- Shamir split and all three recovery combinations.
- Serialized factor/manifest sizes and paper-recovery-share word count.
- Deterministic password encoder latency, character counts, position observations, and password-space upper bound.
- SQLite write/load/query and database size at 100, 1,000, 10,000, and 100,000 CDRs in a full run.
- Conflict classification and all four unknown-outcome reconciliation results.
- Persistent SQLite compare-and-set freshness updates and read-after-restart verification.

The current script does not simulate human subjects, target-system network behavior, energy use, or hardware-backed provider latency. Windows and Linux quantitative rows must come from real CI runners; absence of a row means not measured.
