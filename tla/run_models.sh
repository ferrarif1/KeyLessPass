#!/usr/bin/env bash
set -euo pipefail

model_dir=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
repo_dir=$(CDPATH= cd -- "$model_dir/.." && pwd)
jar_path="$repo_dir/tmp/tla2tools-v1.7.4.jar"

if [[ ! -f "$jar_path" ]]; then
  echo "missing $jar_path" >&2
  exit 1
fi

java -cp "$jar_path" tlc2.TLC -config "$model_dir/epscd_rotation.cfg" "$model_dir/epscd_rotation.tla" \
  | tee "$model_dir/epscd_rotation.log"

negative_control() {
  local config=$1
  local invariant=$2
  local log_file="$model_dir/${config%.cfg}.log"
  if java -cp "$jar_path" tlc2.TLC -config "$model_dir/$config" "$model_dir/epscd_rotation.tla" >"$log_file" 2>&1; then
    echo "negative control $config unexpectedly passed" >&2
    exit 1
  fi
  if ! grep -q "Invariant $invariant is violated" "$log_file"; then
    echo "negative control $config failed for an unexpected reason" >&2
    sed -n '1,120p' "$log_file" >&2
    exit 1
  fi
  echo "negative control $config detected $invariant"
}

negative_control epscd_rotation_negative_http.cfg CommitRequiresEvidence
negative_control epscd_rotation_negative_drop.cfg UncertaintyKeepsBoth
