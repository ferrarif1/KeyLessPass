#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT/rust_core"
cargo build --release
cd "$ROOT/flutter_app"
flutter build linux --release

BUNDLE="$ROOT/flutter_app/build/linux/x64/release/bundle"
cp "$ROOT/rust_core/target/release/libkeylesspass_core.so" "$BUNDLE/lib/"

echo "Linux bundle output: $BUNDLE"
echo "Use fpm or distro packaging to produce .deb/.rpm/AppImage."
