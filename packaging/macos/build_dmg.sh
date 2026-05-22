#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT/rust_core"
cargo build --release
cd "$ROOT/flutter_app"
flutter build macos --release

APP="$ROOT/flutter_app/build/macos/Build/Products/Release/keylesspass_desktop.app"
mkdir -p "$APP/Contents/Frameworks"
cp "$ROOT/rust_core/target/release/libkeylesspass_core.dylib" "$APP/Contents/Frameworks/"

echo "macOS .app output: $APP"
echo "Create DMG with hdiutil/create-dmg after code signing and notarization are configured."
