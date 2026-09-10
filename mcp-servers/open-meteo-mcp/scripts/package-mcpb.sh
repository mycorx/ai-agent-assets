#!/usr/bin/env bash
# Build a release binary and wrap it, with the manifest, into a .mcpb
# bundle for the platform this runs on. Cross-compiling is out of scope:
# each platform's bundle is built on that platform, in CI or by hand.
set -euo pipefail

cd "$(dirname "$0")/.."

case "$(uname -s)" in
  Linux*)  platform=linux;  binary=open-meteo-mcp ;;
  Darwin*) platform=darwin; binary=open-meteo-mcp ;;
  MINGW*|MSYS*|CYGWIN*) platform=win32; binary=open-meteo-mcp.exe ;;
  *) echo "unsupported platform: $(uname -s)" >&2; exit 1 ;;
esac

cargo build --release

staging=$(mktemp -d)
trap 'rm -rf "$staging"' EXIT
mkdir -p "$staging/bin"
cp "target/release/$binary" "$staging/bin/$binary"
cp manifest.json "$staging/manifest.json"

mkdir -p dist
out="$PWD/dist/open-meteo-mcp-$platform.mcpb"
rm -f "$out"
# -j would flatten bin/ into the root; the manifest names bin/, so the
# archive must keep the directory.
(cd "$staging" && zip -qr "$out" manifest.json bin)

echo "wrote $out"
