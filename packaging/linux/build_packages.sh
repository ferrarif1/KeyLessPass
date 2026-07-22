#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
FLUTTER_BIN="${FLUTTER_BIN:-flutter}"
APP_NAME="${APP_NAME:-KeyLessPass}"
APP_ID="${APP_ID:-com.keylesspass.desktop}"
PACKAGE_NAME="${PACKAGE_NAME:-keylesspass}"
ARCH="${ARCH:-amd64}"

if [[ "${KEYLESSPASS_REQUIRE_LICENSE:-}" != "1" && "${KEYLESSPASS_ALLOW_EVALUATION_PACKAGE:-}" != "1" ]]; then
  echo "Refusing to package an unlicensed build. Use tools/commercial/build_commercial_release.sh or set KEYLESSPASS_ALLOW_EVALUATION_PACKAGE=1 for an explicitly marked evaluation artifact." >&2
  exit 1
fi
if [[ "${KEYLESSPASS_REQUIRE_LICENSE:-}" == "1" && -z "${KEYLESSPASS_LICENSE_PUBLIC_KEY_B64:-}" ]]; then
  echo "KEYLESSPASS_LICENSE_PUBLIC_KEY_B64 is required for a commercial package." >&2
  exit 1
fi
if [[ "${KEYLESSPASS_REQUIRE_LICENSE:-}" == "1" && -z "${KEYLESSPASS_LINUX_GPG_KEY_ID:-}" && "${KEYLESSPASS_ALLOW_UNSIGNED:-}" != "1" ]]; then
  echo "Commercial Linux packages require KEYLESSPASS_LINUX_GPG_KEY_ID. Use KEYLESSPASS_ALLOW_UNSIGNED=1 only for local testing." >&2
  exit 1
fi

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

APP_VERSION="${APP_VERSION:-}"
if [[ -z "$APP_VERSION" ]]; then
  APP_VERSION="$(
    sed -nE 's/^version:[[:space:]]*([^+[:space:]]+).*/\1/p' "$ROOT/flutter_app/pubspec.yaml" |
      head -n 1
  )"
fi
APP_VERSION="${APP_VERSION:-0.1.0}"

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

BINARY_NAME="$APP_NAME"
if [[ ! -x "$BUNDLE/$BINARY_NAME" ]]; then
  if [[ -x "$BUNDLE/keylesspass_desktop" ]]; then
    BINARY_NAME="keylesspass_desktop"
  else
    echo "Linux Flutter executable was not found in bundle: $BUNDLE" >&2
    exit 1
  fi
fi

DIST="$ROOT/dist/linux"
rm -rf "$DIST/stage" "$DIST/deb-root" "$DIST/AppDir"
mkdir -p "$DIST"

TAR_ROOT="$DIST/stage/$APP_NAME-linux-x64-$APP_VERSION"
mkdir -p "$TAR_ROOT"
cp -a "$BUNDLE/." "$TAR_ROOT/"
cp "$ROOT/LICENSE" "$TAR_ROOT/LICENSE"
cat >"$TAR_ROOT/run-keylesspass.sh" <<EOF
#!/usr/bin/env bash
set -euo pipefail
cd "\$(dirname "\${BASH_SOURCE[0]}")"
exec "./$BINARY_NAME" "\$@"
EOF
chmod +x "$TAR_ROOT/run-keylesspass.sh"
TAR_OUT="$DIST/$APP_NAME-linux-x64-$APP_VERSION.tar.gz"
rm -f "$TAR_OUT"
tar -C "$DIST/stage" -czf "$TAR_OUT" "$APP_NAME-linux-x64-$APP_VERSION"
echo "Linux tar.gz output: $TAR_OUT"

DEB_DESKTOP_FILE_CONTENT="[Desktop Entry]
Type=Application
Name=$APP_NAME
Comment=Storage-free local password derivation client
Exec=/opt/keylesspass/$BINARY_NAME
Icon=$APP_ID
Terminal=false
Categories=Utility;Security;
StartupWMClass=$APP_ID
"

APPIMAGE_DESKTOP_FILE_CONTENT="[Desktop Entry]
Type=Application
Name=$APP_NAME
Comment=Storage-free local password derivation client
Exec=AppRun
Icon=$APP_ID
Terminal=false
Categories=Utility;Security;
StartupWMClass=$APP_ID
"

if command -v dpkg-deb >/dev/null 2>&1; then
  DEB_ROOT="$DIST/deb-root"
  mkdir -p \
    "$DEB_ROOT/DEBIAN" \
    "$DEB_ROOT/opt/keylesspass" \
    "$DEB_ROOT/usr/share/applications" \
    "$DEB_ROOT/usr/share/icons/hicolor/512x512/apps" \
    "$DEB_ROOT/usr/share/doc/keylesspass"
  cp -a "$BUNDLE/." "$DEB_ROOT/opt/keylesspass/"
  cp "$ROOT/LICENSE" "$DEB_ROOT/usr/share/doc/keylesspass/LICENSE"
  printf "%s" "$DEB_DESKTOP_FILE_CONTENT" >"$DEB_ROOT/usr/share/applications/$APP_ID.desktop"
  cp "$ROOT/flutter_app/assets/logo.png" \
    "$DEB_ROOT/usr/share/icons/hicolor/512x512/apps/$APP_ID.png"
  cat >"$DEB_ROOT/DEBIAN/control" <<EOF
Package: $PACKAGE_NAME
Version: $APP_VERSION
Section: utils
Priority: optional
Architecture: $ARCH
Maintainer: KeyLessPass Project Contributors <revanton@icloud.com>
Depends: libc6, libstdc++6, libgtk-3-0, liblzma5
Description: KeyLessPass desktop password derivation client
 KeyLessPass is a local-only desktop client for deriving enterprise passwords
 on demand without storing target-system plaintext passwords or maintaining an
 encrypted service-password vault.
EOF
  chmod -R go-w "$DEB_ROOT"
  DEB_OUT="$DIST/${PACKAGE_NAME}_${APP_VERSION}_${ARCH}.deb"
  rm -f "$DEB_OUT"
  dpkg-deb --build "$DEB_ROOT" "$DEB_OUT"
  echo "Linux deb output: $DEB_OUT"
else
  echo "dpkg-deb not found; skipped .deb package."
fi

APPIMAGETOOL="${APPIMAGETOOL:-}"
if [[ -z "$APPIMAGETOOL" ]] && command -v appimagetool >/dev/null 2>&1; then
  APPIMAGETOOL="appimagetool"
fi

if [[ -n "$APPIMAGETOOL" ]]; then
  APPDIR="$DIST/AppDir"
  mkdir -p \
    "$APPDIR/usr/lib/keylesspass" \
    "$APPDIR/usr/share/applications" \
    "$APPDIR/usr/share/icons/hicolor/512x512/apps"
  cp -a "$BUNDLE/." "$APPDIR/usr/lib/keylesspass/"
  cp "$ROOT/flutter_app/assets/logo.png" "$APPDIR/$APP_ID.png"
  cp "$ROOT/flutter_app/assets/logo.png" \
    "$APPDIR/usr/share/icons/hicolor/512x512/apps/$APP_ID.png"
  printf "%s" "$APPIMAGE_DESKTOP_FILE_CONTENT" >"$APPDIR/$APP_ID.desktop"
  printf "%s" "$APPIMAGE_DESKTOP_FILE_CONTENT" >"$APPDIR/usr/share/applications/$APP_ID.desktop"
  cat >"$APPDIR/AppRun" <<EOF
#!/usr/bin/env bash
set -euo pipefail
HERE="\$(dirname "\$(readlink -f "\${BASH_SOURCE[0]}")")"
cd "\$HERE/usr/lib/keylesspass"
exec "./$BINARY_NAME" "\$@"
EOF
  chmod +x "$APPDIR/AppRun"
  APPIMAGE_OUT="$DIST/$APP_NAME-linux-x64-$APP_VERSION.AppImage"
  rm -f "$APPIMAGE_OUT"
  ARCH=x86_64 "$APPIMAGETOOL" "$APPDIR" "$APPIMAGE_OUT"
  echo "Linux AppImage output: $APPIMAGE_OUT"
else
  echo "appimagetool not found; skipped AppImage package."
fi

find "$DIST" -maxdepth 1 -type f \
  \( -name '*.tar.gz' -o -name '*.deb' -o -name '*.AppImage' \) \
  -print0 | sort -z | xargs -0 sha256sum >"$DIST/SHA256SUMS"
if [[ -n "${KEYLESSPASS_LINUX_GPG_KEY_ID:-}" ]]; then
  if ! command -v gpg >/dev/null 2>&1; then
    echo "gpg is required to sign commercial Linux checksums." >&2
    exit 1
  fi
  gpg --batch --yes --armor --detach-sign \
    --local-user "$KEYLESSPASS_LINUX_GPG_KEY_ID" \
    --output "$DIST/SHA256SUMS.asc" "$DIST/SHA256SUMS"
fi
echo "Linux checksum manifest: $DIST/SHA256SUMS"
