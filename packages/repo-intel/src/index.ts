/**
 * repo-intel public SDK API.
 *
 * Install:  npm install repo-intel
 * Usage:
 *   import { analyze, scan, generate } from 'repo-intel'
 *
 * The three main functions map to the three use cases:
 *   scan()     → context.json only  (Rust pipeline, no AI)
 *   generate() → agent docs only    (requires context, calls AI)
 *   analyze()  → full pipeline      (scan + generate in one call)
 */

export type {
  // Core context types
  RepoContext,
  ProjectMeta,
  StackResult,
  ArchMeta,
  ArchStyle,
  Skill,
  SkillSource,

  // AI types
  AIConfig,
  AIProvider,
  AIProviderName,

  // Output types
  AgentDoc,
  PipelineResult,
  GenerateOptions,
  RepoIntelConfig,
} from "./types";

export { DEFAULT_MODELS, API_KEY_ENV_VARS } from "./types";

// ── Core functions ────────────────────────────────────────────────────────────

import { resolve } from "node:path";
import { runRustPipeline } from "./pipeline/runner";
import { runPipeline } from "./pipeline/index";
import { generateAgentDocs } from "./ai/index";
import { generateStatic } from "./fallback/index";
import type {
  RepoContext,
  AIConfig,
  AgentDoc,
  GenerateOptions,
  PipelineResult,
} from "./types";

/**
 * Scan a repository and return the structured `RepoContext`.
 *
 * Spawns the Rust binary and returns the parsed `context.json`.
 * No AI calls are made. Useful for inspecting what was detected
 * before committing to a generate run.
 *
 * @param root  Path to the repository root. Defaults to `process.cwd()`.
 */
export async function scan(root?: string): Promise<RepoContext> {
  const resolvedRoot = resolve(root ?? process.cwd());
  return runRustPipeline(resolvedRoot);
}

/**
 * Generate agent docs from an existing `RepoContext`.
 *
 * Use this when you already have a context (e.g. from `scan()`) and want
 * to regenerate docs without re-scanning. Supports both AI and static modes.
 *
 * @param context  The repo context from `scan()` or `analyze()`.
 * @param config   AI provider config. Required unless `noAi` is true.
 * @param noAi     If true, use static templates instead of an LLM.
 */
export async function generate(
  context: RepoContext,
  config?: AIConfig,
  noAi = false,
): Promise<{
  docs: AgentDoc[];
  errors: Array<{ role: string; error: string }>;
}> {
  if (noAi || !config) {
    const docs = generateStatic(context);
    return { docs, errors: [] };
  }

  return generateAgentDocs(context, config);
}

/**
 * Run the full pipeline: scan → generate → return results.
 *
 * This is the highest-level API — equivalent to running `repo-intel generate`
 * from the CLI, but without writing files to disk.
 *
 * @param root     Path to the repository root. Defaults to `process.cwd()`.
 * @param options  Pipeline options (provider, noAi, dryRun, debug, etc.)
 */
export async function analyze(
  root?: string,
  options: Partial<GenerateOptions> = {},
): Promise<PipelineResult> {
  const fullOptions: GenerateOptions = {
    root: resolve(root ?? process.cwd()),
    noAi: options.noAi ?? false,
    dryRun: options.dryRun ?? true, // SDK default: don't write files
    debug: options.debug ?? false,
    provider: options.provider,
    outputDir: options.outputDir,
  };

  return runPipeline(fullOptions);
}

// ── Loader utilities (useful for SDK consumers building their own pipelines) ──

export {
  getBinaryPath,
  isPlatformSupported,
  getSupportedPlatforms,
} from "./loader";

// ── AI sub-exports (for advanced use) ─────────────────────────────────────────

export { buildPrompt } from "./ai/prompt-builder";
export { callAI } from "./ai/caller";
export { parseAgentResponse, roleToFilename } from "./ai/parser";
