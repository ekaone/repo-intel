import { createRequire } from "node:module";
import { arch, platform } from "node:os";
import { join } from "node:path";

type Platform = "linux" | "darwin" | "win32";
type Arch = "x64" | "arm64";

const PLATFORM_MAP: Record<`${Platform}-${Arch}`, string> = {
  "linux-x64": "repo-intel-linux-x64",
  "darwin-arm64": "repo-intel-darwin-arm64",
  "darwin-x64": "repo-intel-darwin-x64",
  "win32-x64": "repo-intel-win32-x64",
};

/**
 * Resolve the absolute path to the platform-specific binary.
 * Throws if the current platform is unsupported.
 */
export function resolveBinaryPath(): string {
  const os = platform() as Platform;
  const cpu = arch() as Arch;
  const key = `${os}-${cpu}` as keyof typeof PLATFORM_MAP;

  const pkgName = PLATFORM_MAP[key];
  if (!pkgName) {
    throw new Error(
      `Unsupported platform: ${os}-${cpu}. ` +
        `Supported platforms: ${Object.keys(PLATFORM_MAP).join(", ")}`,
    );
  }

  const require = createRequire(import.meta.url);
  try {
    const pkgDir = require.resolve(`${pkgName}/package.json`);
    const binName = os === "win32" ? "repo-intel.exe" : "repo-intel";
    return join(pkgDir, "..", "bin", binName);
  } catch {
    throw new Error(
      `Could not locate binary package "${pkgName}". ` +
        `Make sure it is installed as an optional dependency.`,
    );
  }
}
