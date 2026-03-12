import { spawnSync } from "child_process";
import { resolve } from "path";
import { getBinaryPath } from "../loader";
import type { RepoContext } from "../types";

/** Max stdout buffer — 10MB is far more than any context.json will need. */
const MAX_BUFFER = 10 * 1024 * 1024;

/**
 * Spawn the Rust binary, run the full scan pipeline, and return the parsed
 * `RepoContext` from its stdout.
 *
 * The binary is called with `scan --root <root> --json` which prints
 * compact JSON to stdout and debug info to stderr only.
 *
 * @param root  Absolute path to the repository root to scan.
 * @param debug If true, forward the binary's stderr to our stderr.
 * @throws      On binary not found, non-zero exit, empty output, or JSON parse failure.
 */
export function runRustPipeline(root: string, debug = false): RepoContext {
  const bin = getBinaryPath();
  const absoluteRoot = resolve(root);

  if (debug) {
    console.error(`[debug] binary: ${bin}`);
    console.error(`[debug] scanning: ${absoluteRoot}`);
  }

  const result = spawnSync(bin, ["scan", "--root", absoluteRoot, "--json"], {
    encoding: "utf8",
    maxBuffer: MAX_BUFFER,
  });

  // ── Spawn-level error (binary not found, permission denied, etc.) ─────────
  if (result.error) {
    throw new Error(
      `repo-intel: failed to spawn binary '${bin}':\n${result.error.message}\n\n` +
        `Run 'repo-intel --version' to verify the binary is installed correctly.`,
    );
  }

  // ── Forward binary stderr in debug mode ───────────────────────────────────
  if (debug && result.stderr) {
    process.stderr.write(result.stderr);
  }

  // ── Non-zero exit — binary ran but failed ─────────────────────────────────
  if (result.status !== 0) {
    const stderr = result.stderr?.trim() ?? "(no stderr output)";
    throw new Error(
      `repo-intel binary exited with code ${result.status}:\n${stderr}`,
    );
  }

  // ── Empty stdout — binary succeeded but printed nothing ──────────────────
  const stdout = result.stdout?.trim() ?? "";
  if (!stdout) {
    throw new Error(
      `repo-intel binary produced no output.\n` +
        `Try running manually: ${bin} scan --root ${absoluteRoot} --json`,
    );
  }

  // ── Parse context.json ────────────────────────────────────────────────────
  let context: RepoContext;

  try {
    context = JSON.parse(stdout) as RepoContext;
  } catch (err) {
    const preview = stdout.slice(0, 200);
    throw new Error(
      `repo-intel binary output is not valid JSON:\n${preview}\n\n` +
        `Parse error: ${err instanceof Error ? err.message : String(err)}`,
    );
  }

  // ── Minimal shape validation ───────────────────────────────────────────────
  // Guard against a future Rust/JS version mismatch producing an unexpected shape.
  validateContext(context);

  if (debug) {
    console.error(
      `[debug] context.json parsed: version=${context.version}, roles=${context.agent_roles.length}`,
    );
  }

  return context;
}

// ── Shape validation ──────────────────────────────────────────────────────────

function validateContext(ctx: unknown): asserts ctx is RepoContext {
  if (typeof ctx !== "object" || ctx === null) {
    throw new Error("repo-intel: context.json is not an object");
  }

  const required = [
    "version",
    "scanned_at",
    "root",
    "project",
    "stack",
    "architecture",
    "agent_roles",
  ];
  const obj = ctx as Record<string, unknown>;

  for (const field of required) {
    if (!(field in obj)) {
      throw new Error(
        `repo-intel: context.json is missing required field '${field}'.\n` +
          `This may be a version mismatch between the Rust binary and the JS package.`,
      );
    }
  }
}
