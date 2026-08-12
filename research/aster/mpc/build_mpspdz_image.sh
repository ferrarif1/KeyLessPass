#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR=$(cd "$(dirname "$0")" && pwd)
MP_SPDZ_COMMIT=9d809599ea6ce627216a389ca7d984fbb75d0cb9
IMAGE=aster-mpspdz:mal-shamir-bmr-max5
BUILD_DIR=$(mktemp -d "${TMPDIR:-/tmp}/aster-mpspdz.XXXXXX")

cleanup() {
  rm -rf "$BUILD_DIR"
}
trap cleanup EXIT

git clone --filter=blob:none https://github.com/data61/MP-SPDZ.git "$BUILD_DIR/MP-SPDZ"
git -C "$BUILD_DIR/MP-SPDZ" checkout "$MP_SPDZ_COMMIT"
git -C "$BUILD_DIR/MP-SPDZ" submodule update --init Programs/Circuits
git -C "$BUILD_DIR/MP-SPDZ" apply "$SCRIPT_DIR/mp-spdz-docker-retries.patch"
git -C "$BUILD_DIR/MP-SPDZ" apply "$SCRIPT_DIR/mp-spdz-bullseye-barrier.patch"
git -C "$BUILD_DIR/MP-SPDZ" apply "$SCRIPT_DIR/mp-spdz-max-parties.patch"

DOCKER_BUILDKIT=0 docker build \
  --target machine \
  --tag "$IMAGE" \
  --build-arg machine=mal-shamir-bmr-party.x \
  "$BUILD_DIR/MP-SPDZ"

docker image inspect "$IMAGE" --format '{{.Id}} {{.Size}}'
