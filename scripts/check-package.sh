#!/usr/bin/env bash
# Smoke-test the packaged archive from a clean directory.
set -euo pipefail

cd "$(dirname "$0")/.."
ARCHIVE="$(ls -1t target/package/folisift-v*-linux-*.tar.gz | head -1)"
TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT
tar -xzf "$ARCHIVE" -C "$TMP"
BIN="$(find "$TMP" -maxdepth 2 -type f -name folisift | head -1)"
"$BIN" --version
tar -tzf "$ARCHIVE" | grep '/models/potion-multilingual-128M-int8/model.safetensors' >/dev/null || {
  echo "package check failed: semantic model missing" >&2
  exit 1
}
tar -tzf "$ARCHIVE" | grep '/LICENSE$' >/dev/null || {
  echo "package check failed: LICENSE missing" >&2
  exit 1
}
"$BIN" doctor >/dev/null
echo "package check passed: $ARCHIVE"
