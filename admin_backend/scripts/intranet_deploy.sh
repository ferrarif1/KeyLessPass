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
  cat > .env <<EOF
KEYLESSPASS_ADMIN_TOKEN=${ADMIN_TOKEN}
KEYLESSPASS_LICENSE_SIGNING_KEY_B64=${SIGNING_KEY}
KEYLESSPASS_LICENSE_KEY_ID=keylesspass-license-2026-q3
KEYLESSPASS_LICENSE_ISSUER=KeyLessPass Commercial Admin
KEYLESSPASS_ADMIN_PORT=8787
EOF
  chmod 600 .env
  echo "Created .env with a fresh admin token and license signing seed."
else
  ADMIN_TOKEN="$(sed -n 's/^KEYLESSPASS_ADMIN_TOKEN=//p' .env | tail -n 1)"
  echo "Using existing .env."
fi

$COMPOSE up -d --build

PORT="$(sed -n 's/^KEYLESSPASS_ADMIN_PORT=//p' .env | tail -n 1)"
if [ -z "${PORT}" ]; then
  PORT=8787
fi

echo
echo "KeyLessPass Admin is running."
echo "URL:   http://127.0.0.1:${PORT}"
echo "Token: ${ADMIN_TOKEN}"
echo
echo "After login, copy the displayed public key into the commercial client build verifier."
