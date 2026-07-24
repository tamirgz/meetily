#!/usr/bin/env bash
set -euo pipefail

REPOSITORY_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
IMAGE_NAME="${MEETILY_LINUX_BUILD_IMAGE:-meetily-linux-builder:0.4.0}"
OUTPUT_DIR="${REPOSITORY_ROOT}/release-artifacts/linux"

mkdir -p "${OUTPUT_DIR}"

docker build \
  --platform linux/amd64 \
  --file "${REPOSITORY_ROOT}/packaging/linux/Dockerfile" \
  --tag "${IMAGE_NAME}" \
  "${REPOSITORY_ROOT}"

docker run \
  --rm \
  --platform linux/amd64 \
  --volume "${OUTPUT_DIR}:/out" \
  "${IMAGE_NAME}"

echo "Linux installers are available in ${OUTPUT_DIR}"
