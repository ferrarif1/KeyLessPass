#!/usr/bin/env bash
set -euo pipefail

ROOT=$(cd "$(dirname "$0")/../../.." && pwd)
TLA_DIR="$ROOT/research/aster/tla"
JAR=${TLA2TOOLS_JAR:-$ROOT/tmp/tla2tools-v1.7.4.jar}

if [[ ! -f "$JAR" ]]; then
  echo "Missing tla2tools jar: $JAR" >&2
  exit 1
fi

cd "$TLA_DIR"
java -cp "$JAR" tlc2.TLC -cleanup -deadlock -config ASTER.cfg ASTER.tla >main.log 2>&1

for config in negative_drop_candidate negative_expiry negative_freshness \
  negative_generation negative_http negative_replay negative_retirement negative_root; do
  set +e
  java -cp "$JAR" tlc2.TLC -cleanup -deadlock -config "$config.cfg" ASTER.tla >"$config.log" 2>&1
  status=$?
  set -e
  if [[ $status -eq 0 ]]; then
    echo "Negative control unexpectedly succeeded: $config" >&2
    exit 1
  fi
  if ! grep -q 'Error: Invariant' "$config.log"; then
    echo "Negative control did not report an invariant counterexample: $config" >&2
    exit 1
  fi
done

echo "TLC positive model and eight negative controls completed"
