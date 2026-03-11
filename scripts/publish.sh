#!/usr/bin/env bash
# Manual publish fallback — normally CI handles this via release.yml.
# Usage: VERSION=0.2.0 ./scripts/publish.sh
set -euo pipefail

if [[ -z "${VERSION:-}" ]]; then
  echo "Error: VERSION is not set. Example: VERSION=0.2.0 ./scripts/publish.sh"
  exit 1
fi

if [[ -z "${NPM_TOKEN:-}" ]]; then
  echo "Error: NPM_TOKEN is not set."
  exit 1
fi

echo "Publishing version $VERSION..."

PACKAGES=(
  "repo-intel-linux-x64"
  "repo-intel-darwin-arm64"
  "repo-intel-darwin-x64"
  "repo-intel-win32-x64"
  "repo-intel"
)

for PKG in "${PACKAGES[@]}"; do
  echo "Publishing $PKG..."
  cd "packages/$PKG"
  npm version "$VERSION" --no-git-tag-version
  npm publish --access public
  cd ../..
  echo "✓ $PKG published"
done

echo ""
echo "All packages published at v$VERSION"
