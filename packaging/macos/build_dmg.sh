#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
APP_NAME="${APP_NAME:-KeyLessPass}"
CODESIGN_IDENTITY="${CODESIGN_IDENTITY:--}"
FLUTTER_BIN="${FLUTTER_BIN:-flutter}"

cd "$ROOT/rust_core"
cargo build --release

cd "$ROOT/flutter_app"
"$FLUTTER_BIN" build macos --release

APP="$ROOT/flutter_app/build/macos/Build/Products/Release/$APP_NAME.app"
ENTITLEMENTS="$ROOT/flutter_app/macos/Runner/Release.entitlements"
mkdir -p "$APP/Contents/Frameworks"
cp "$ROOT/rust_core/target/release/libkeylesspass_core.dylib" "$APP/Contents/Frameworks/"
codesign --force --sign "$CODESIGN_IDENTITY" "$APP/Contents/Frameworks/libkeylesspass_core.dylib"
codesign --force --deep --sign "$CODESIGN_IDENTITY" --entitlements "$ENTITLEMENTS" "$APP"

echo "macOS .app output: $APP"

if [[ "${CREATE_DMG:-0}" == "1" ]]; then
  DIST="$ROOT/dist/macos"
  mkdir -p "$DIST"
  DMG="$DIST/$APP_NAME.dmg"
  rm -f "$DMG"
  hdiutil create -volname "$APP_NAME" -srcfolder "$APP" -ov -format UDZO "$DMG"
  echo "DMG output: $DMG"
else
  echo "Set CREATE_DMG=1 to create a local unsigned DMG with hdiutil."
fi

echo "For distribution, sign with Developer ID Application and submit the DMG for notarization."
