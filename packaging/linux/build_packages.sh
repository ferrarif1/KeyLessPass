#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
FLUTTER_BIN="${FLUTTER_BIN:-flutter}"

if [[ "$(uname -s)" != "Linux" ]]; then
  echo "Linux packaging must run on a Linux host." >&2
  exit 1
fi

if ! command -v cargo >/dev/null 2>&1; then
  echo "cargo was not found in PATH." >&2
  exit 1
fi

if ! command -v "$FLUTTER_BIN" >/dev/null 2>&1; then
  echo "Flutter executable was not found: $FLUTTER_BIN" >&2
  exit 1
fi

if [[ ! -d "$ROOT/flutter_app/linux" ]]; then
  echo "No Linux desktop project configured at flutter_app/linux." >&2
  echo "Run 'flutter create --platforms=linux .' inside flutter_app first." >&2
  exit 1
fi

cd "$ROOT/rust_core"
cargo build --release

CORE_SO="$ROOT/rust_core/target/release/libkeylesspass_core.so"
if [[ ! -f "$CORE_SO" ]]; then
  echo "Rust Core shared library was not created: $CORE_SO" >&2
  exit 1
fi

cd "$ROOT/flutter_app"
"$FLUTTER_BIN" build linux --release

BUNDLE="$ROOT/flutter_app/build/linux/x64/release/bundle"
if [[ ! -d "$BUNDLE/lib" ]]; then
  echo "Linux Flutter bundle lib directory was not created: $BUNDLE/lib" >&2
  exit 1
fi

cp "$CORE_SO" "$BUNDLE/lib/"

echo "Linux bundle output: $BUNDLE"
echo "Use fpm or distro packaging to produce .deb/.rpm/AppImage."
