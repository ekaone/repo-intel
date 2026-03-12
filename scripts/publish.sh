#!/usr/bin/env bash
# ─────────────────────────────────────────────────────────────────────────────
# scripts/publish.sh
#
# Manual fallback for publishing all repo-intel npm packages.
# In normal operation, CI handles this automatically on git tag push.
# Use this script only when you need to publish outside of CI.
#
# Publish order (important — platform packages must exist before main):
#   1. repo-intel-linux-x64
#   2. repo-intel-darwin-arm64
#   3. repo-intel-darwin-x64
#   4. repo-intel-win32-x64
#   5. repo-intel  (main package — depends on the 4 above)
#
# Usage:
#   ./scripts/publish.sh                    # publish all (dry-run by default)
#   ./scripts/publish.sh --live             # actually publish to npm
#   ./scripts/publish.sh --platform-only    # publish only the 4 platform pkgs
#   ./scripts/publish.sh --main-only        # publish only the main package
#
# Requirements:
#   - npm login completed (npm whoami should return your username)
#   - All 4 binaries copied into packages/*/bin/ (run copy-binaries.sh first)
#   - JS built (pnpm build)
# ─────────────────────────────────────────────────────────────────────────────

set -euo pipefail

# ── Colour helpers ────────────────────────────────────────────────────────────
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
BOLD='\033[1m'
RESET='\033[0m'

info()    { echo -e "${BOLD}  →${RESET} $*"; }
success() { echo -e "${GREEN}  ✓${RESET} $*"; }
warn()    { echo -e "${YELLOW}  ⚠${RESET} $*"; }
error()   { echo -e "${RED}  ✗${RESET} $*" >&2; }
die()     { error "$*"; exit 1; }
dim()     { echo -e "    ${CYAN}$*${RESET}"; }

# ── Repo root ─────────────────────────────────────────────────────────────────
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"

# ── Parse args ────────────────────────────────────────────────────────────────
DRY_RUN=true
PLATFORM_ONLY=false
MAIN_ONLY=false

for arg in "$@"; do
  case "${arg}" in
    --live)          DRY_RUN=false ;;
    --platform-only) PLATFORM_ONLY=true ;;
    --main-only)     MAIN_ONLY=true ;;
    --help|-h)
      echo "Usage: ./scripts/publish.sh [--live] [--platform-only] [--main-only]"
      exit 0
      ;;
    *) die "Unknown argument: ${arg}" ;;
  esac
done

# ── Package definitions ───────────────────────────────────────────────────────

PLATFORM_PACKAGES=(
  repo-intel-linux-x64
  repo-intel-darwin-arm64
  repo-intel-darwin-x64
  repo-intel-win32-x64
)

MAIN_PACKAGE="repo-intel"

# ── Banner ────────────────────────────────────────────────────────────────────

echo ""
echo -e "${BOLD}repo-intel — publish${RESET}"

if ${DRY_RUN}; then
  warn "DRY RUN mode — pass --live to actually publish"
else
  echo -e "${RED}${BOLD}  LIVE MODE — publishing to npm${RESET}"
fi

echo ""

# ── Pre-flight checks ─────────────────────────────────────────────────────────

info "Running pre-flight checks…"

# 1. npm auth
if ! npm whoami &>/dev/null; then
  die "Not logged in to npm. Run: npm login"
fi

NPM_USER=$(npm whoami)
success "npm authenticated as: ${NPM_USER}"

# 2. Verify version consistency across all package.json files
MAIN_VERSION=$(node -p "require('${REPO_ROOT}/packages/${MAIN_PACKAGE}/package.json').version")
info "Main package version: ${MAIN_VERSION}"

VERSION_MISMATCH=false
for pkg in "${PLATFORM_PACKAGES[@]}"; do
  PKG_DIR="${REPO_ROOT}/packages/${pkg}"
  if [[ -f "${PKG_DIR}/package.json" ]]; then
    PKG_VERSION=$(node -p "require('${PKG_DIR}/package.json').version")
    if [[ "${PKG_VERSION}" != "${MAIN_VERSION}" ]]; then
      error "${pkg} version mismatch: ${PKG_VERSION} ≠ ${MAIN_VERSION}"
      VERSION_MISMATCH=true
    fi
  fi
done

if ${VERSION_MISMATCH}; then
  die "Version mismatch detected. Align all package.json versions to ${MAIN_VERSION} first."
fi

success "All package versions consistent: ${MAIN_VERSION}"

# 3. Verify binaries are present in platform packages
if ! ${MAIN_ONLY}; then
  MISSING_BINS=false
  for pkg in "${PLATFORM_PACKAGES[@]}"; do
    BIN_DIR="${REPO_ROOT}/packages/${pkg}/bin"
    # Check for either repo-intel or repo-intel.exe
    if [[ ! -f "${BIN_DIR}/repo-intel" ]] && [[ ! -f "${BIN_DIR}/repo-intel.exe" ]]; then
      error "${pkg}: binary missing in ${BIN_DIR}/"
      error "  Run: ./scripts/copy-binaries.sh"
      MISSING_BINS=true
    else
      success "${pkg}: binary present"
    fi
  done

  if ${MISSING_BINS}; then
    die "Missing binaries. Run copy-binaries.sh first."
  fi
fi

# 4. Verify JS is built
if ! ${PLATFORM_ONLY}; then
  DIST_DIR="${REPO_ROOT}/packages/${MAIN_PACKAGE}/dist"
  if [[ ! -d "${DIST_DIR}" ]] || [[ -z "$(ls -A "${DIST_DIR}" 2>/dev/null)" ]]; then
    warn "dist/ is empty or missing — building JS now…"
    cd "${REPO_ROOT}"
    pnpm --filter repo-intel build
    success "JS build complete"
  else
    success "dist/ present"
  fi
fi

echo ""

# ── Publish function ──────────────────────────────────────────────────────────

publish_package() {
  local pkg_dir="$1"
  local pkg_name="$2"

  info "Publishing ${pkg_name}…"
  dim "Directory: ${pkg_dir}"

  local npm_args=(
    publish
    --access public
    --no-git-checks
  )

  if ${DRY_RUN}; then
    npm_args+=(--dry-run)
  fi

  cd "${pkg_dir}"

  if npm "${npm_args[@]}"; then
    if ${DRY_RUN}; then
      success "${pkg_name}: dry-run OK"
    else
      success "${pkg_name}: published ✓"
    fi
  else
    error "${pkg_name}: publish failed"
    return 1
  fi

  cd "${REPO_ROOT}"
  echo ""
}

# ── Publish platform packages ─────────────────────────────────────────────────

if ! ${MAIN_ONLY}; then
  echo -e "${BOLD}── Platform packages ────────────────────────────────────────────${RESET}"
  echo ""

  for pkg in "${PLATFORM_PACKAGES[@]}"; do
    PKG_DIR="${REPO_ROOT}/packages/${pkg}"

    if [[ ! -d "${PKG_DIR}" ]]; then
      warn "${pkg}: directory not found, skipping"
      continue
    fi

    publish_package "${PKG_DIR}" "${pkg}"
  done
fi

# ── Publish main package ──────────────────────────────────────────────────────

if ! ${PLATFORM_ONLY}; then
  echo -e "${BOLD}── Main package ─────────────────────────────────────────────────${RESET}"
  echo ""

  MAIN_DIR="${REPO_ROOT}/packages/${MAIN_PACKAGE}"
  publish_package "${MAIN_DIR}" "${MAIN_PACKAGE}"
fi

# ── Done ──────────────────────────────────────────────────────────────────────

echo -e "${BOLD}── Complete ─────────────────────────────────────────────────────${RESET}"

if ${DRY_RUN}; then
  echo ""
  warn "This was a DRY RUN. To actually publish, run:"
  echo ""
  echo "    ./scripts/publish.sh --live"
  echo ""
else
  echo ""
  success "All packages published successfully!"
  echo ""
  dim "View on npm: https://www.npmjs.com/package/repo-intel"
  echo ""
fi