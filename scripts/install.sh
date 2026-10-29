#!/usr/bin/env bash
# Install a released Folisift Linux binary with its sidecars.
set -euo pipefail

say() { printf '%s\n' "$*" >&2; }
fail() { say "error: $*"; exit 1; }

os="$(uname -s)"
[ "$os" = "Linux" ] || fail "prebuilt archives are currently Linux-only; build from source for $os"

case "$(uname -m)" in
  x86_64|amd64) arch="x86_64" ;;
  aarch64|arm64) arch="aarch64" ;;
  *) fail "unsupported architecture: $(uname -m)" ;;
esac

version="${FOLISIFT_VERSION:-}"
if [ -z "$version" ]; then
  version="$(curl -fsSL https://api.github.com/repos/Aylmz0/folisift/releases/latest \
    | sed -n 's/.*"tag_name"[[:space:]]*:[[:space:]]*"\([^"]*\)".*/\1/p' \
    | head -n 1)"
  [ -n "$version" ] || fail "could not resolve the latest release; set FOLISIFT_VERSION"
fi

name="folisift-${version}-linux-${arch}.tar.gz"
base="${FOLISIFT_BASE_URL:-https://github.com/Aylmz0/folisift/releases/download}"
url="${base}/${version}/${name}"
prefix="${FOLISIFT_PREFIX:-$HOME/.local}"
tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT

say "Installing ${name}"
say "  from ${url}"
say "  into ${prefix}/lib/folisift"
curl -fL --retry 3 --silent --show-error -o "${tmp}/${name}" "${url}"
curl -fL --retry 3 --silent --show-error -o "${tmp}/${name}.sha256" "${url}.sha256"
(
  cd "$tmp"
  sha256sum -c "${name}.sha256"
)

mkdir -p "${prefix}/lib/folisift" "${prefix}/bin"
rm -rf "${prefix:?}/lib/folisift"/*
tar -xzf "${tmp}/${name}" -C "${prefix}/lib/folisift" --strip-components=1
chmod +x "${prefix}/lib/folisift/folisift"
ln -sfn "${prefix}/lib/folisift/folisift" "${prefix}/bin/folisift"

say ""
say "Installed ${version}."
say "  binary: ${prefix}/bin/folisift"
say "  docs:   https://github.com/Aylmz0/folisift/blob/main/docs/install.md"
case ":${PATH}:" in
  *":${prefix}/bin:"*) ;;
  *) say "Add ${prefix}/bin to PATH, then run: folisift doctor" ;;
esac
