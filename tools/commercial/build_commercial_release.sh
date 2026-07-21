#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
TARGET_PLATFORM="${1:-$(uname -s)}"

if [[ -z "${KEYLESSPASS_LICENSE_PUBLIC_KEY_B64:-}" ]]; then
  cat >&2 <<'EOF'
KEYLESSPASS_LICENSE_PUBLIC_KEY_B64 is required.

Deploy admin_backend first, open the admin page, copy publicKeyB64, then run:
  KEYLESSPASS_LICENSE_PUBLIC_KEY_B64='<public key>' tools/commercial/build_commercial_release.sh macos
EOF
  exit 1
fi

export KEYLESSPASS_REQUIRE_LICENSE=1
export KEYLESSPASS_BUILD_CHANNEL="${KEYLESSPASS_BUILD_CHANNEL:-commercial}"
export KEYLESSPASS_LICENSE_KEY_ID="${KEYLESSPASS_LICENSE_KEY_ID:-keylesspass-license-2026-q3}"

case "$TARGET_PLATFORM" in
  macos|darwin|Darwin)
    exec "$ROOT/packaging/macos/build_dmg.sh"
    ;;
  linux|Linux)
    exec "$ROOT/packaging/linux/build_packages.sh"
    ;;
  windows|Windows)
    cat >&2 <<'EOF'
Run the Windows commercial build from PowerShell so Authenticode and Inno Setup are available:

  $env:KEYLESSPASS_REQUIRE_LICENSE="1"
  $env:KEYLESSPASS_BUILD_CHANNEL="commercial"
  $env:KEYLESSPASS_LICENSE_KEY_ID="keylesspass-license-2026-q3"
  $env:KEYLESSPASS_LICENSE_PUBLIC_KEY_B64="<public key from admin_backend>"
  packaging\windows\build_installer.ps1
EOF
    exit 2
    ;;
  *)
    echo "Unsupported platform: $TARGET_PLATFORM" >&2
    echo "Use one of: macos, linux, windows" >&2
    exit 2
    ;;
esac
