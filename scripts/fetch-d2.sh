#!/usr/bin/env bash
# Fetch the pinned D2 binary for the current Linux amd64/arm64 host.
set -euo pipefail

cd "$(dirname "$0")/.."
VERSION="v0.9.0"
ARCH="$(uname -m)"
case "$ARCH" in
  x86_64|amd64) ASSET="d2-${VERSION}-linux-amd64.tar.gz"; SUM="5669ddc46b99e942cc96078f4a4e36d5e62103348f4c05179ede27802fdd87a9" ;;
  aarch64|arm64) ASSET="d2-${VERSION}-linux-arm64.tar.gz"; SUM="ac2c028697199479acb321db1e3d68caee9f2ba492ed73caa3cd13f3829bf913" ;;
  *) echo "unsupported architecture: $ARCH" >&2; exit 1 ;;
esac

DEST="target/package/d2-${VERSION}"
mkdir -p target/package
if [ ! -x "${DEST}/bin/d2" ]; then
  curl -L --fail --silent --show-error \
    -o "target/package/${ASSET}" \
    "https://github.com/d2lang/d2/releases/download/${VERSION}/${ASSET}"
  echo "${SUM}  target/package/${ASSET}" | sha256sum -c -
  rm -rf "$DEST"
  mkdir -p "$DEST"
  tar -xzf "target/package/${ASSET}" -C "$DEST" --strip-components=1
fi
cp "${DEST}/LICENSE.txt" "target/package/D2-LICENSE.txt"
echo "${DEST}/bin/d2"
