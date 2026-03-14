import { createRequire } from "module";
import { existsSync } from "fs";
import { resolve, join } from "path";
import { fileURLToPath } from "url";

// ── Platform → npm package mapping ───────────────────────────────────────────

const PLATFORM_PACKAGES: Record<string, string> = {
  "linux-x64": "@ekaone/repo-intel-linux-x64",
  "darwin-arm64": "@ekaone/repo-intel-darwin-arm64",
  "darwin-x64": "@ekaone/repo-intel-darwin-x64",
  "win32-x64": "@ekaone/repo-intel-win32-x64",
};

/** Binary filename — `.exe` on Windows, bare name everywhere else. */
const BINARY_NAME =
  process.platform === "win32" ? "repo-intel.exe" : "repo-intel";

// ── Resolver ──────────────────────────────────────────────────────────────────

/**
 * Resolve the absolute path to the platform-specific Rust binary.
 *
 * Resolution order:
 * 1. `REPO_INTEL_BINARY` env var override (useful for local development)
 * 2. Platform npm package (`repo-intel-<os>-<arch>/bin/repo-intel`)
 * 3. Fallback: look for a locally built binary in `../../target/release/`
 *    (lets contributors run from a `cargo build --release` without npm install)
 *
 * Throws a descriptive error if no binary can be found.
 */
export function getBinaryPath(): string {
  // ── 1. Env var override ───────────────────────────────────────────────────
  const envOverride = process.env["REPO_INTEL_BINARY"];
  if (envOverride) {
    if (!existsSync(envOverride)) {
      throw new Error(
        `repo-intel: REPO_INTEL_BINARY is set to '${envOverride}' but the file does not exist`,
      );
    }
    return envOverride;
  }

  // ── 2. Platform npm package ───────────────────────────────────────────────
  const platformKey = `${process.platform}-${process.arch}`;
  const pkgName = PLATFORM_PACKAGES[platformKey];

  if (!pkgName) {
    throw new Error(
      `repo-intel: unsupported platform '${platformKey}'\n` +
        `Supported: ${Object.keys(PLATFORM_PACKAGES).join(", ")}\n` +
        `Alternatively, install Rust and run: cargo install repo-intel`,
    );
  }

  // Use createRequire for ESM compatibility — require.resolve works in CJS but
  // not in native ESM modules. createRequire bridges the gap.
  const require = createRequire(import.meta.url);

  try {
    return require.resolve(`${pkgName}/bin/${BINARY_NAME}`);
  } catch {
    // ── 3. Local dev fallback ───────────────────────────────────────────────
    const devBinary = resolveDevBinary();
    if (devBinary) return devBinary;

    throw new Error(
      `repo-intel: platform binary not found for '${platformKey}'\n` +
        `Install the platform package: npm install ${pkgName}\n` +
        `Or build from source: cargo build --release`,
    );
  }
}

/**
 * Try to find a locally compiled binary for contributors running from source.
 * Looks for `../../target/release/repo-intel` relative to this file.
 */
function resolveDevBinary(): string | null {
  try {
    // __dirname equivalent in ESM
    const thisFile = fileURLToPath(import.meta.url);
    // packages/repo-intel/src/loader.ts → ../../.. → repo root
    const repoRoot = resolve(thisFile, "..", "..", "..", "..", "..");
    const devPath = join(repoRoot, "target", "release", BINARY_NAME);

    if (existsSync(devPath)) {
      return devPath;
    }

    // Also check target/debug for `cargo build` (non-release)
    const debugPath = join(repoRoot, "target", "debug", BINARY_NAME);
    if (existsSync(debugPath)) {
      return debugPath;
    }
  } catch {
    // Ignore resolution errors in the dev fallback
  }
  return null;
}

// ── Platform info helpers ─────────────────────────────────────────────────────

/** Returns the platform package name for the current OS/arch, or null if unsupported. */
export function getPlatformPackageName(): string | null {
  const key = `${process.platform}-${process.arch}`;
  return PLATFORM_PACKAGES[key] ?? null;
}

/** Returns true if the current platform is supported. */
export function isPlatformSupported(): boolean {
  return getPlatformPackageName() !== null;
}

/** Returns all supported platform keys. */
export function getSupportedPlatforms(): string[] {
  return Object.keys(PLATFORM_PACKAGES);
}
