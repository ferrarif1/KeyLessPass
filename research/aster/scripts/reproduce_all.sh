#!/usr/bin/env bash
set -euo pipefail

ROOT=$(cd "$(dirname "$0")/../../.." && pwd)
MODE=${1:---quick}
PYTHON_BIN=${PYTHON_BIN:-python3}

if [[ "$MODE" != "--quick" && "$MODE" != "--full" ]]; then
  echo "Usage: $0 [--quick|--full]" >&2
  exit 2
fi

cd "$ROOT"
"$PYTHON_BIN" research/aster/scripts/run_test_suite.py

cd "$ROOT/rust_core"
cargo run --release --features research --example aster_experiments -- \
  ../research/aster/results/raw/semantic_results.json

cd "$ROOT"
"$PYTHON_BIN" research/aster/adapters/run_fault_matrix.py \
  --output research/aster/results/raw/rq5_fault_traces.jsonl \
  --summary research/aster/results/generated/rq5_summary.json \
  --repetitions 3
"$PYTHON_BIN" research/aster/experiments/run_scalability.py \
  --output research/aster/results/raw/rq7_scalability.json
research/aster/scripts/run_tla.sh

if [[ "$MODE" == "--full" ]]; then
  cd "$ROOT/rust_core"
  cargo run --release --features research --example aster_rq1 -- \
    ../experiments/real_policy_corpus/translated_corpus.json \
    ../research/aster/results/raw/rq1_policy_results.jsonl
  cd "$ROOT"
  if ! docker image inspect aster-mpspdz:mal-shamir-bmr-max5 >/dev/null 2>&1; then
    research/aster/mpc/build_mpspdz_image.sh
  fi
  research/aster/mpc/aster_exact_domain_reference.py \
    --output research/aster/results/raw/rq6_reference_vector.json >/dev/null
  research/aster/mpc/run_mpspdz_experiment.py --repetitions 3 \
    --output research/aster/results/raw/rq6_mpspdz.json
  "$PYTHON_BIN" research/aster/adapters/run_openldap_smoke.py \
    --output research/aster/results/raw/rq5_openldap.json
fi

cd "$ROOT"
"$PYTHON_BIN" research/aster/scripts/generate_results.py
"$PYTHON_BIN" research/aster/scripts/run_security_scan.py

echo "ASTER reproduction completed in $MODE mode"
