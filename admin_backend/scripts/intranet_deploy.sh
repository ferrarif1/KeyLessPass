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

detect_server_ip() {
  if [ -n "${KEYLESSPASS_SERVER_IP:-}" ]; then
    printf '%s' "$KEYLESSPASS_SERVER_IP"
  elif command -v hostname >/dev/null 2>&1 && hostname -I >/dev/null 2>&1; then
    hostname -I | awk '{print $1}'
  elif command -v ipconfig >/dev/null 2>&1; then
    ipconfig getifaddr en0 2>/dev/null || ipconfig getifaddr en1 2>/dev/null || true
  fi
}

SERVER_IP="$(detect_server_ip)"
if [ -z "$SERVER_IP" ]; then
  SERVER_IP=127.0.0.1
  echo "Warning: could not detect an intranet IP. Set KEYLESSPASS_PUBLIC_BASE_URL in .env." >&2
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
KEYLESSPASS_PUBLIC_BASE_URL=http://${SERVER_IP}:8787
KEYLESSPASS_HOST_BIND=0.0.0.0
KEYLESSPASS_AUTOMATIC_LEASE_HOURS=24
KEYLESSPASS_AUTOMATIC_GRACE_DAYS=1
KEYLESSPASS_AUTOMATIC_REQUESTS_PER_MINUTE=20
EOF
  chmod 600 .env
  echo "Created .env with a fresh admin token and license signing seed."
else
  ADMIN_TOKEN="$(sed -n 's/^KEYLESSPASS_ADMIN_TOKEN=//p' .env | tail -n 1)"
  if [ "${#ADMIN_TOKEN}" -lt 24 ]; then
    ADMIN_TOKEN="$(openssl rand -hex 32)"
    TEMP_ENV="$(mktemp ./.env.XXXXXX)"
    awk -v token="$ADMIN_TOKEN" '
      BEGIN { replaced = 0 }
      /^KEYLESSPASS_ADMIN_TOKEN=/ { print "KEYLESSPASS_ADMIN_TOKEN=" token; replaced = 1; next }
      { print }
      END { if (!replaced) print "KEYLESSPASS_ADMIN_TOKEN=" token }
    ' .env > "$TEMP_ENV"
    chmod 600 "$TEMP_ENV"
    mv "$TEMP_ENV" .env
    echo "Generated a fresh local deployment token."
  fi
  echo "Using existing .env."
fi

if ! grep -q '^KEYLESSPASS_PUBLIC_BASE_URL=' .env; then
  printf '\nKEYLESSPASS_PUBLIC_BASE_URL=http://%s:8787\n' "$SERVER_IP" >> .env
fi
if ! grep -q '^KEYLESSPASS_HOST_BIND=' .env; then
  printf 'KEYLESSPASS_HOST_BIND=0.0.0.0\n' >> .env
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
echo "KeyLessPass intranet service is running."
echo "Download URL: http://${SERVER_IP}:${PORT}/download"
echo "Fallback config: http://${SERVER_IP}:${PORT}/keylesspass-client-config.json"
echo "Maintenance URL: http://${SERVER_IP}:${PORT}/"
echo "Token: ${ADMIN_TOKEN}"
echo
echo "Users download and authorize automatically. Only batch exchange and maintenance require the token."
