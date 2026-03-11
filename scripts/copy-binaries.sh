#!/usr/bin/env bash
# Copy compiled binaries from target/<triple>/release into packages/*/bin/
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

copy_binary() {
  local target="$1"
  local pkg="$2"
  local binary="$3"

  local src="$ROOT_DIR/target/$target/release/$binary"
  local dst="$ROOT_DIR/packages/$pkg/bin/$binary"

  if [[ ! -f "$src" ]]; then
    echo "WARN: Binary not found at $src — skipping $pkg"
    return
  fi

  mkdir -p "$(dirname "$dst")"
  cp "$src" "$dst"
  echo "✓ Copied $target → packages/$pkg/bin/$binary"
}

copy_binary "x86_64-unknown-linux-gnu"  "repo-intel-linux-x64"    "repo-intel"
copy_binary "aarch64-apple-darwin"      "repo-intel-darwin-arm64" "repo-intel"
copy_binary "x86_64-apple-darwin"       "repo-intel-darwin-x64"   "repo-intel"
copy_binary "x86_64-pc-windows-gnu"     "repo-intel-win32-x64"    "repo-intel.exe"

echo ""
echo "Done copying binaries."
