#!/usr/bin/env bash
# Build a Linux x86_64 release archive with the binary and PDFium sidecar.
set -euo pipefail

cd "$(dirname "$0")/.."

VERSION="$(cargo metadata --no-deps --format-version 1 | python3 -c 'import json,sys; print(json.load(sys.stdin)["packages"][0]["version"])')"
ARCH="$(uname -m)"
NAME="folisift-v${VERSION}-linux-${ARCH}"
STAGE="target/package/${NAME}"

cargo build --release -p folisift-cli

rm -rf "$STAGE"
mkdir -p "$STAGE"
cp target/release/folisift "$STAGE/"
cp -R styles domains "$STAGE/"

PDFIUM="${FOLISIFT_PDFIUM_DIR:-target/release/deps}"
if [ -f "${PDFIUM}/libpdfium.so" ]; then
  cp "${PDFIUM}/libpdfium.so" "$STAGE/"
else
  echo "warning: libpdfium.so not found in ${PDFIUM}; archive has no sidecar" >&2
fi

tar -C target/package -czf "target/package/${NAME}.tar.gz" "$NAME"
(
  cd target/package
  sha256sum "${NAME}.tar.gz" > "${NAME}.tar.gz.sha256"
)

echo "target/package/${NAME}.tar.gz"
echo "target/package/${NAME}.tar.gz.sha256"
