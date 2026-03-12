#!/usr/bin/env bash
# ─────────────────────────────────────────────────────────────────────────────
# scripts/build-all.sh
#
# Cross-compile the repo-intel Rust binary for all 4 supported platforms.
#
# Uses `cross` (https://github.com/cross-rs/cross) which runs each
# cross-compilation target inside a Docker container — no Rust target
# toolchains need to be installed manually.
#
# Usage:
#   ./scripts/build-all.sh              # build all 4 targets
#   ./scripts/build-all.sh linux-x64    # build a single target by name
#
# Requirements:
#   - Rust + Cargo installed
#   - Docker running (required by cross)
#   - cross installed (auto-installed if missing)
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

# ── Target definitions ────────────────────────────────────────────────────────
# Format: "friendly-name|rust-triple|binary-name"

declare -A TARGETS=(
  [linux-x64]="x86_64-unknown-linux-gnu|repo-intel"
  [darwin-arm64]="aarch64-apple-darwin|repo-intel"
  [darwin-x64]="x86_64-apple-darwin|repo-intel"
  [win32-x64]="x86_64-pc-windows-msvc|repo-intel.exe"
)

# Ordered list for consistent output
TARGET_ORDER=(linux-x64 darwin-arm64 darwin-x64 win32-x64)

# ── Repo root ─────────────────────────────────────────────────────────────────
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"

# ── Parse args ────────────────────────────────────────────────────────────────
SELECTED_TARGETS=()

if [[ $# -eq 0 ]]; then
  SELECTED_TARGETS=("${TARGET_ORDER[@]}")
else
  for arg in "$@"; do
    if [[ -v TARGETS[$arg] ]]; then
      SELECTED_TARGETS+=("$arg")
    else
      die "Unknown target '$arg'. Valid targets: ${TARGET_ORDER[*]}"
    fi
  done
fi

# ── Prerequisites ─────────────────────────────────────────────────────────────

echo ""
echo -e "${BOLD}repo-intel — build-all${RESET}"
echo "  Building ${#SELECTED_TARGETS[@]} target(s): ${SELECTED_TARGETS[*]}"
echo ""

# Check Rust
if ! command -v cargo &>/dev/null; then
  die "cargo not found. Install Rust from https://rustup.rs"
fi

# Check Docker (required by cross)
if ! command -v docker &>/dev/null; then
  die "docker not found. Docker is required by 'cross' for cross-compilation."
fi

if ! docker info &>/dev/null; then
  die "Docker is not running. Start Docker and try again."
fi

# Install cross if missing
if ! command -v cross &>/dev/null; then
  warn "'cross' not found — installing via cargo…"
  cargo install cross --git https://github.com/cross-rs/cross
  success "cross installed"
fi

# ── Build each target ─────────────────────────────────────────────────────────

FAILED=()
BUILT=()

cd "${REPO_ROOT}"

for name in "${SELECTED_TARGETS[@]}"; do
  IFS='|' read -r triple binary <<< "${TARGETS[$name]}"

  echo -e "${BOLD}── Building: ${name} (${triple}) ──────────────────────────────${RESET}"

  # Native build for the host platform (faster, no Docker needed)
  HOST_TRIPLE="$(rustc -vV | grep host | awk '{print $2}')"

  if [[ "${triple}" == "${HOST_TRIPLE}" ]]; then
    info "Native build (host matches target — skipping cross)"
    BUILD_CMD="cargo"
  else
    info "Cross build via Docker"
    BUILD_CMD="cross"
  fi

  if ${BUILD_CMD} build \
    --release \
    --target "${triple}" \
    --package repo-intel-core; then

    BINARY_PATH="${REPO_ROOT}/target/${triple}/release/${binary}"

    if [[ -f "${BINARY_PATH}" ]]; then
      SIZE=$(du -sh "${BINARY_PATH}" | cut -f1)
      success "${name}: ${BINARY_PATH} (${SIZE})"
      BUILT+=("${name}")
    else
      error "${name}: build succeeded but binary not found at ${BINARY_PATH}"
      FAILED+=("${name}")
    fi
  else
    error "${name}: build failed"
    FAILED+=("${name}")
  fi

  echo ""
done

# ── Summary ───────────────────────────────────────────────────────────────────

echo -e "${BOLD}── Summary ──────────────────────────────────────────────────────${RESET}"

if [[ ${#BUILT[@]} -gt 0 ]]; then
  success "Built (${#BUILT[@]}): ${BUILT[*]}"
fi

if [[ ${#FAILED[@]} -gt 0 ]]; then
  error "Failed (${#FAILED[@]}): ${FAILED[*]}"
  echo ""
  echo "  Run with a single target to see the full error:"
  echo "  ./scripts/build-all.sh ${FAILED[0]}"
  exit 1
fi

echo ""
echo "  Next step: ./scripts/copy-binaries.sh"
echo ""