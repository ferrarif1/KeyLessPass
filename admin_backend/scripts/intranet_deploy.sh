#!/usr/bin/env bash
set -eu

cd "$(dirname "$0")/.."

if ! command -v openssl >/dev/null 2>&1; then
  echo "openssl is required to generate the admin token and signing seed." >&2
  exit 1
fi

if docker compose version >/dev/null 2>&1; then
  COMPOSE="docker compose"
elif command -v docker-compose >/dev/null 2>&1; then
  COMPOSE="docker-compose"
else
  echo "Docker Compose is required for one-click intranet deployment." >&2
  exit 1
fi

if [ ! -f .env ]; then
  ADMIN_TOKEN="$(openssl rand -hex 32)"
  SIGNING_KEY="$(openssl rand -base64 32)"
  SITE_KEY_ID="keylesspass-site-$(openssl rand -hex 8)"
  cat > .env <<EOF
KEYLESSPASS_ADMIN_TOKEN=${ADMIN_TOKEN}
KEYLESSPASS_LICENSE_SIGNING_KEY_B64=${SIGNING_KEY}
KEYLESSPASS_LICENSE_KEY_ID=${SITE_KEY_ID}
KEYLESSPASS_LICENSE_ISSUER=KeyLessPass Customer Site
KEYLESSPASS_VENDOR_KEY_ID=keylesspass-vendor-root-2026
KEYLESSPASS_VENDOR_PUBLIC_KEY_B64=replace-with-vendor-root-public-key
KEYLESSPASS_ADMIN_PORT=8787
EOF
  chmod 600 .env
  echo "Created .env with a fresh admin token and license signing seed."
else
  ADMIN_TOKEN="$(sed -n 's/^KEYLESSPASS_ADMIN_TOKEN=//p' .env | tail -n 1)"
  echo "Using existing .env."
fi

mkdir -p downloads license
$COMPOSE build

SITE_KEY="$($COMPOSE run --rm --no-deps keylesspass-admin site-public-key)"
if [ ! -s license/customer-entitlement.json ] || grep -q '^KEYLESSPASS_VENDOR_PUBLIC_KEY_B64=replace-' .env; then
  echo
  echo "The customer site is initialized but cannot issue licenses yet."
  echo "Send the values below and the requested seat/device list to the KeyLessPass vendor:"
  echo "$SITE_KEY"
  echo "The vendor must return:"
  echo "  1. the vendor root public key for KEYLESSPASS_VENDOR_PUBLIC_KEY_B64"
  echo "  2. license/customer-entitlement.json signed by the offline vendor root"
  echo "After installing both values, run this script again."
  exit 2
fi

$COMPOSE up -d

PORT="$(sed -n 's/^KEYLESSPASS_ADMIN_PORT=//p' .env | tail -n 1)"
if [ -z "${PORT}" ]; then
  PORT=8787
fi

echo
echo "KeyLessPass Admin is running."
echo "URL:   http://127.0.0.1:${PORT}"
echo "Token: ${ADMIN_TOKEN}"
echo
echo "Public downloads do not require login. Administrative authorization operations require the token."
