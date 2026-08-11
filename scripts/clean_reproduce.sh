#!/usr/bin/env bash
set -euo pipefail

repo_dir=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
cd "$repo_dir"

command -v cargo >/dev/null
command -v python3 >/dev/null
command -v java >/dev/null

cargo test --manifest-path rust_core/Cargo.toml
cargo run --release --manifest-path rust_core/Cargo.toml --example policy_space_evaluation
cargo build --release --manifest-path rust_core/Cargo.toml --example compile_policy_worker
python3 experiments/real_policy_corpus/run_full_corpus.py
cargo run --release --quiet --manifest-path rust_core/Cargo.toml --example walk_corpus
cargo run --release --quiet --manifest-path rust_core/Cargo.toml --example epscd_mainline_evaluation \
  > experiments/epscd_mainline.json
mkdir -p experiments/epscd_rotation
cargo run --release --quiet --manifest-path rust_core/Cargo.toml --example epscd_rotation_evaluation \
  > experiments/epscd_rotation/fault_matrix.json
./tla/run_models.sh
python3 artifact/generate_results.py

echo "EPSCD artifact reproduced under artifact/results and artifact/generated"
