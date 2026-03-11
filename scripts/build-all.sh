#!/usr/bin/env bash
# Build Rust binaries for all 4 supported targets.
# Requires the cross-compilation toolchains to be installed.
# See docs/contributing.md for setup instructions.

set -euo pipefail

TARGETS=(
  "x86_64-unknown-linux-gnu"
  "aarch64-apple-darwin"
  "x86_64-apple-darwin"
  "x86_64-pc-windows-gnu"
)

echo "Building repo-intel-core for all targets..."

for TARGET in "${TARGETS[@]}"; do
  echo ""
  echo "── Building for $TARGET ──"
  cargo build --release --target "$TARGET" -p repo-intel-core
  echo "✓ $TARGET done"
done

echo ""
echo "All targets built successfully!"
echo "Run scripts/copy-binaries.sh to copy them into packages/*/bin/"
