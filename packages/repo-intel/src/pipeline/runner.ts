import { spawn } from "node:child_process";
import type { ScanOptions, RepoContext } from "../types.js";
import { resolveBinaryPath } from "../loader.js";

/**
 * Spawn the Rust binary in `scan` mode and parse the context.json from stdout.
 */
export async function spawnRust(opts: ScanOptions): Promise<RepoContext> {
  const binaryPath = resolveBinaryPath();
  const args: string[] = ["scan", "--root", opts.root ?? "."];

  if (opts.config) {
    args.push("--config", opts.config);
  }

  return new Promise((resolve, reject) => {
    const child = spawn(binaryPath, args, { stdio: ["ignore", "pipe", "pipe"] });

    let stdout = "";
    let stderr = "";

    child.stdout.on("data", (chunk: Buffer) => {
      stdout += chunk.toString();
    });

    child.stderr.on("data", (chunk: Buffer) => {
      stderr += chunk.toString();
    });

    child.on("close", (code) => {
      if (code !== 0) {
        reject(
          new Error(
            `repo-intel binary exited with code ${code}.\nstderr: ${stderr}`,
          ),
        );
        return;
      }

      try {
        const context: RepoContext = JSON.parse(stdout);
        resolve(context);
      } catch (err) {
        reject(
          new Error(`Failed to parse context.json from binary output: ${err}`),
        );
      }
    });

    child.on("error", reject);
  });
}

// Re-export as the public `scan` API
export const scan = spawnRust;
