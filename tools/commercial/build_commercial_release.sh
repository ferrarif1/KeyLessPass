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

if ! command -v python3 >/dev/null 2>&1; then
  echo "python3 is required to validate commercial license public keys." >&2
  exit 1
fi

python3 - <<'PY'
import base64
import json
import os

def decode(value):
    return base64.b64decode(value) if "+" in value or "/" in value or "=" in value else base64.urlsafe_b64decode(value + "=" * (-len(value) % 4))

current = os.environ["KEYLESSPASS_LICENSE_PUBLIC_KEY_B64"]
if len(decode(current)) != 32:
    raise SystemExit("KEYLESSPASS_LICENSE_PUBLIC_KEY_B64 must decode to 32 bytes")

ring = os.environ.get("KEYLESSPASS_LICENSE_TRUSTED_KEYS_JSON")
if ring:
    keys = json.loads(ring)
    if not isinstance(keys, dict) or not keys:
        raise SystemExit("KEYLESSPASS_LICENSE_TRUSTED_KEYS_JSON must be a non-empty JSON object")
    if any(not key or len(decode(value)) != 32 for key, value in keys.items()):
        raise SystemExit("every trusted key must have an ID and decode to 32 bytes")
    active_id = os.environ.get("KEYLESSPASS_LICENSE_KEY_ID", "keylesspass-license-2026-q3")
    if active_id in keys and decode(keys[active_id]) != decode(current):
        raise SystemExit("trusted key ring conflicts with the active license key ID")
PY

export KEYLESSPASS_REQUIRE_LICENSE=1
export KEYLESSPASS_BUILD_CHANNEL="${KEYLESSPASS_BUILD_CHANNEL:-commercial}"
export KEYLESSPASS_LICENSE_KEY_ID="${KEYLESSPASS_LICENSE_KEY_ID:-keylesspass-license-2026-q3}"
export KEYLESSPASS_APP_MAJOR_VERSION="${KEYLESSPASS_APP_MAJOR_VERSION:-1}"

if [[ "$KEYLESSPASS_BUILD_CHANNEL" == "desktop" || "$KEYLESSPASS_BUILD_CHANNEL" == "evaluation" ]]; then
  echo "Commercial builds require a non-evaluation KEYLESSPASS_BUILD_CHANNEL." >&2
  exit 1
fi
if [[ ! "$KEYLESSPASS_APP_MAJOR_VERSION" =~ ^[0-9]+$ ]]; then
  echo "KEYLESSPASS_APP_MAJOR_VERSION must be a non-negative integer." >&2
  exit 1
fi

case "$TARGET_PLATFORM" in
  macos|darwin|Darwin)
    export KEYLESSPASS_MANAGED_LICENSE_FILE="${KEYLESSPASS_MANAGED_LICENSE_FILE:-/Library/Application Support/KeyLessPass/license-bundle.json}"
    exec "$ROOT/packaging/macos/build_dmg.sh"
    ;;
  linux|Linux)
    export KEYLESSPASS_MANAGED_LICENSE_FILE="${KEYLESSPASS_MANAGED_LICENSE_FILE:-/etc/keylesspass/license-bundle.json}"
    exec "$ROOT/packaging/linux/build_packages.sh"
    ;;
  windows|Windows)
    cat >&2 <<'EOF'
Run the Windows commercial build from PowerShell so Authenticode and Inno Setup are available:

  $env:KEYLESSPASS_REQUIRE_LICENSE="1"
  $env:KEYLESSPASS_BUILD_CHANNEL="commercial"
  $env:KEYLESSPASS_LICENSE_KEY_ID="keylesspass-license-2026-q3"
  $env:KEYLESSPASS_LICENSE_PUBLIC_KEY_B64="<public key from admin_backend>"
  $env:KEYLESSPASS_APP_MAJOR_VERSION="1"
  $env:KEYLESSPASS_MANAGED_LICENSE_FILE="C:\ProgramData\KeyLessPass\license-bundle.json"
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
