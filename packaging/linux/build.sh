#!/usr/bin/env bash
set -euo pipefail

TARGET="x86_64-unknown-linux-gnu"
PROFILE="release"
OUTPUT_DIR="/out"

mkdir -p \
  "frontend/src-tauri/binaries" \
  "${OUTPUT_DIR}"

cargo build \
  --release \
  --package llama-helper \
  --target "${TARGET}"

cp \
  "target/${TARGET}/${PROFILE}/llama-helper" \
  "frontend/src-tauri/binaries/llama-helper-${TARGET}"

pnpm --dir frontend tauri build \
  --config src-tauri/tauri.ci.conf.json \
  --target "${TARGET}" \
  --bundles deb,appimage \
  -- \
  --features openblas

find "target/${TARGET}/${PROFILE}/bundle" \
  -type f \
  \( -name '*.deb' -o -name '*.AppImage' \) \
  -exec cp {} "${OUTPUT_DIR}/" \;

if ! find "${OUTPUT_DIR}" -maxdepth 1 -type f \
  \( -name '*.deb' -o -name '*.AppImage' \) \
  -print -quit | grep -q .; then
  echo "No Linux installer was produced." >&2
  exit 1
fi

cd "${OUTPUT_DIR}"
sha256sum ./*.deb ./*.AppImage > SHA256SUMS-linux.txt
ls -lh
