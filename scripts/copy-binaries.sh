#!/usr/bin/env bash
# ─────────────────────────────────────────────────────────────────────────────
# scripts/copy-binaries.sh
#
# Copy compiled Rust binaries from target/<triple>/release/
# into the correct platform npm package's bin/ directory.
#
# Must be run AFTER build-all.sh has completed successfully.
#
# Usage:
#   ./scripts/copy-binaries.sh              # copy all 4 platforms
#   ./scripts/copy-binaries.sh linux-x64    # copy a single platform
#
# After this script, each platform package will contain:
#   packages/repo-intel-linux-x64/bin/repo-intel
#   packages/repo-intel-darwin-arm64/bin/repo-intel
#   packages/repo-intel-darwin-x64/bin/repo-intel
#   packages/repo-intel-win32-x64/bin/repo-intel.exe
# ─────────────────────────────────────────────────────────────────────────────

set -euo pipefail

# ── Colour helpers ────────────────────────────────────────────────────────────
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BOLD='\033[1m'
RESET='\033[0m'

info()    { echo -e "${BOLD}  →${RESET} $*"; }
success() { echo -e "${GREEN}  ✓${RESET} $*"; }
warn()    { echo -e "${YELLOW}  ⚠${RESET} $*"; }
error()   { echo -e "${RED}  ✗${RESET} $*" >&2; }
die()     { error "$*"; exit 1; }

# ── Platform definitions ──────────────────────────────────────────────────────
# Format: "rust-triple|npm-package-name|binary-filename"

declare -A PLATFORMS=(
  [linux-x64]="x86_64-unknown-linux-gnu|repo-intel-linux-x64|repo-intel"
  [darwin-arm64]="aarch64-apple-darwin|repo-intel-darwin-arm64|repo-intel"
  [darwin-x64]="x86_64-apple-darwin|repo-intel-darwin-x64|repo-intel"
  [win32-x64]="x86_64-pc-windows-msvc|repo-intel-win32-x64|repo-intel.exe"
)

PLATFORM_ORDER=(linux-x64 darwin-arm64 darwin-x64 win32-x64)

# ── Repo root ─────────────────────────────────────────────────────────────────
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"

# ── Parse args ────────────────────────────────────────────────────────────────
SELECTED=()

if [[ $# -eq 0 ]]; then
  SELECTED=("${PLATFORM_ORDER[@]}")
else
  for arg in "$@"; do
    if [[ -v PLATFORMS[$arg] ]]; then
      SELECTED+=("$arg")
    else
      die "Unknown platform '$arg'. Valid: ${PLATFORM_ORDER[*]}"
    fi
  done
fi

# ── Main ──────────────────────────────────────────────────────────────────────

echo ""
echo -e "${BOLD}repo-intel — copy-binaries${RESET}"
echo "  Copying ${#SELECTED[@]} platform(s): ${SELECTED[*]}"
echo ""

COPIED=()
SKIPPED=()
FAILED=()

for name in "${SELECTED[@]}"; do
  IFS='|' read -r triple npm_pkg binary <<< "${PLATFORMS[$name]}"

  SRC="${REPO_ROOT}/target/${triple}/release/${binary}"
  DEST_DIR="${REPO_ROOT}/packages/${npm_pkg}/bin"
  DEST="${DEST_DIR}/${binary}"

  info "${name}: ${SRC}"
  info "      → ${DEST}"

  # ── Check source binary exists ──────────────────────────────────────────
  if [[ ! -f "${SRC}" ]]; then
    warn "${name}: binary not found at ${SRC}"
    warn "  Run: ./scripts/build-all.sh ${name}"
    SKIPPED+=("${name}")
    echo ""
    continue
  fi

  # ── Check npm package directory exists ──────────────────────────────────
  if [[ ! -d "${REPO_ROOT}/packages/${npm_pkg}" ]]; then
    error "${name}: npm package directory not found: packages/${npm_pkg}"
    FAILED+=("${name}")
    echo ""
    continue
  fi

  # ── Create bin/ directory ────────────────────────────────────────────────
  mkdir -p "${DEST_DIR}"

  # ── Copy binary ──────────────────────────────────────────────────────────
  cp "${SRC}" "${DEST}"

  # ── Make executable (not needed on Windows but harmless) ─────────────────
  chmod +x "${DEST}"

  # ── Verify ───────────────────────────────────────────────────────────────
  if [[ -f "${DEST}" ]]; then
    SIZE=$(du -sh "${DEST}" | cut -f1)
    success "${name}: copied (${SIZE})"
    COPIED+=("${name}")
  else
    error "${name}: copy failed"
    FAILED+=("${name}")
  fi

  echo ""
done

# ── Summary ───────────────────────────────────────────────────────────────────

echo -e "${BOLD}── Summary ──────────────────────────────────────────────────────${RESET}"

if [[ ${#COPIED[@]} -gt 0 ]]; then
  success "Copied   (${#COPIED[@]}): ${COPIED[*]}"
fi

if [[ ${#SKIPPED[@]} -gt 0 ]]; then
  warn "Skipped  (${#SKIPPED[@]}): ${SKIPPED[*]}  ← binary not built yet"
fi

if [[ ${#FAILED[@]} -gt 0 ]]; then
  error "Failed   (${#FAILED[@]}): ${FAILED[*]}"
  exit 1
fi

if [[ ${#COPIED[@]} -gt 0 ]]; then
  echo ""
  echo "  Next step: ./scripts/publish.sh"
  echo ""
fi