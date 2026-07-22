#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
APP_NAME="${APP_NAME:-KeyLessPass}"
CODESIGN_IDENTITY="${CODESIGN_IDENTITY:--}"
FLUTTER_BIN="${FLUTTER_BIN:-flutter}"
MACOS_RUST_TARGETS="${MACOS_RUST_TARGETS:-x86_64-apple-darwin,aarch64-apple-darwin}"

if [[ "$(uname -s)" != "Darwin" ]]; then
  echo "macOS packaging must run on a macOS host." >&2
  exit 1
fi

if ! command -v cargo >/dev/null 2>&1; then
  echo "cargo was not found in PATH." >&2
  exit 1
fi

if ! command -v rustup >/dev/null 2>&1; then
  echo "rustup was not found in PATH; it is required for universal macOS builds." >&2
  exit 1
fi

if ! command -v lipo >/dev/null 2>&1; then
  echo "lipo was not found in PATH; install the full Xcode command-line tools." >&2
  exit 1
fi

if ! command -v "$FLUTTER_BIN" >/dev/null 2>&1; then
  echo "Flutter executable was not found: $FLUTTER_BIN" >&2
  exit 1
fi

APP_VERSION="${APP_VERSION:-}"
if [[ -z "$APP_VERSION" ]]; then
  APP_VERSION="$(
    sed -nE 's/^version:[[:space:]]*([^+[:space:]]+).*/\1/p' "$ROOT/flutter_app/pubspec.yaml" |
      head -n 1
  )"
fi
APP_VERSION="${APP_VERSION:-0.1.0}"

cd "$ROOT/rust_core"
IFS=',' read -r -a RUST_TARGETS <<<"$MACOS_RUST_TARGETS"
CORE_LIBS=()
for RUST_TARGET in "${RUST_TARGETS[@]}"; do
  if ! rustup target list --installed | grep -Fxq "$RUST_TARGET"; then
    echo "Rust target is not installed: $RUST_TARGET" >&2
    echo "Install it with: rustup target add $RUST_TARGET" >&2
    exit 1
  fi
  cargo build --release --target "$RUST_TARGET"
  CORE_LIBS+=("$ROOT/rust_core/target/$RUST_TARGET/release/libkeylesspass_core.dylib")
done

UNIVERSAL_CORE="$ROOT/rust_core/target/macos-universal/libkeylesspass_core.dylib"
mkdir -p "$(dirname "$UNIVERSAL_CORE")"
if [[ "${#CORE_LIBS[@]}" -eq 1 ]]; then
  cp "${CORE_LIBS[0]}" "$UNIVERSAL_CORE"
else
  lipo -create "${CORE_LIBS[@]}" -output "$UNIVERSAL_CORE"
fi

cd "$ROOT/flutter_app"
"$FLUTTER_BIN" build macos --release

APP="$ROOT/flutter_app/build/macos/Build/Products/Release/$APP_NAME.app"
ENTITLEMENTS="$ROOT/flutter_app/macos/Runner/Release.entitlements"
mkdir -p "$APP/Contents/Frameworks"
cp "$UNIVERSAL_CORE" "$APP/Contents/Frameworks/libkeylesspass_core.dylib"
codesign --force --sign "$CODESIGN_IDENTITY" "$APP/Contents/Frameworks/libkeylesspass_core.dylib"
codesign --force --deep --sign "$CODESIGN_IDENTITY" --entitlements "$ENTITLEMENTS" "$APP"

echo "macOS .app output: $APP"

CREATE_DMG="${CREATE_DMG:-1}"
if [[ "$CREATE_DMG" != "1" ]]; then
  echo "Set CREATE_DMG=1 to create a DMG distribution package."
  exit 0
fi

DIST="$ROOT/dist/macos"
DMG_ROOT="$DIST/dmg-root"
DMG="$DIST/$APP_NAME-$APP_VERSION-macos.dmg"
rm -rf "$DMG_ROOT"
mkdir -p "$DMG_ROOT"
cp -R "$APP" "$DMG_ROOT/"
ln -s /Applications "$DMG_ROOT/Applications"
mkdir -p "$DIST"
rm -f "$DMG"
hdiutil create -volname "$APP_NAME" -srcfolder "$DMG_ROOT" -ov -format UDZO "$DMG"
rm -rf "$DMG_ROOT"

if [[ "$CODESIGN_IDENTITY" != "-" ]]; then
  codesign --force --sign "$CODESIGN_IDENTITY" "$DMG"
fi

echo "DMG output: $DMG"
echo "For public distribution, notarize and staple the DMG before release."
