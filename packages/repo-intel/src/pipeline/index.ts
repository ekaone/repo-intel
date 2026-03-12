import { resolve, join } from "path";
import type {
  GenerateOptions,
  PipelineResult,
  AIConfig,
  RepoIntelConfig,
} from "../types";
import { runRustPipeline } from "./runner";
import { writeAgentDocs, printSummary } from "./writer";
import { generateAgentDocs } from "../ai/index";
import { generateStatic } from "../fallback/index";

// ── Default config ────────────────────────────────────────────────────────────

const DEFAULT_OUTPUT_DIR = "agents";
const DEFAULT_PROVIDER = "anthropic" as const;

// ── Main export ───────────────────────────────────────────────────────────────

/**
 * Run the full repo-intel pipeline end-to-end:
 *
 *   1. Spawn Rust binary → context.json
 *   2. Generate agent docs via AI or static fallback
 *   3. Write docs to disk (or print in dry-run mode)
 *
 * This is the single entry point used by both the CLI (`cli.ts`) and the
 * public SDK API (`index.ts`).
 */
export async function runPipeline(
  options: GenerateOptions,
  repoConfig?: Partial<RepoIntelConfig>,
): Promise<PipelineResult> {
  const startMs = Date.now();
  const root = resolve(options.root);

  if (options.debug) {
    console.error(`[debug] pipeline start`);
    console.error(`[debug] root:    ${root}`);
    console.error(`[debug] no-ai:   ${options.noAi}`);
    console.error(`[debug] dry-run: ${options.dryRun}`);
    console.error(
      `[debug] provider: ${options.provider ?? repoConfig?.ai?.provider ?? DEFAULT_PROVIDER}`,
    );
  }

  // ── Step 1: Spawn Rust binary → RepoContext ───────────────────────────────
  console.log(`  Scanning ${root}…`);

  const context = runRustPipeline(root, options.debug);

  console.log(
    `  Detected: ${context.stack.language}${context.stack.framework ? " + " + context.stack.framework : ""}`,
  );
  console.log(
    `  Roles:    ${context.agent_roles.length > 0 ? context.agent_roles.join(", ") : "(none detected)"}`,
  );

  if (context.agent_roles.length === 0) {
    console.warn("\n  warn: no agent roles were detected for this repository.");
    console.warn(
      "  The repo may be too minimal or use an unsupported stack.\n",
    );
  }

  // ── Step 2: Generate agent docs ───────────────────────────────────────────
  let docs: Awaited<ReturnType<typeof generateAgentDocs>>["docs"];
  let errors: Awaited<ReturnType<typeof generateAgentDocs>>["errors"];
  let usedAi: boolean;

  if (options.noAi) {
    // ── Static fallback path ──────────────────────────────────────────────
    console.log(`  Generating docs (static mode)…`);

    docs = generateStatic(context, { debug: options.debug });
    errors = [];
    usedAi = false;
  } else {
    // ── AI path ───────────────────────────────────────────────────────────
    const aiConfig = buildAIConfig(options, repoConfig);

    console.log(`  Generating docs via ${aiConfig.provider}…`);

    const result = await generateAgentDocs(context, aiConfig, {
      debug: options.debug,
      onProgress: (role, i, total) => {
        console.log(`  [${i}/${total}] ${role}`);
      },
    });

    docs = result.docs;
    errors = result.errors;
    usedAi = true;
  }

  // ── Step 3: Write files (or dry-run print) ────────────────────────────────
  const outputDir = resolve(
    root,
    options.outputDir ?? repoConfig?.output?.dir ?? DEFAULT_OUTPUT_DIR,
  );

  writeAgentDocs(docs, outputDir, options.dryRun);

  const durationMs = Date.now() - startMs;

  // ── Summary ───────────────────────────────────────────────────────────────
  if (!options.dryRun) {
    printSummary(docs, errors, context, durationMs, usedAi);
  }

  if (options.debug) {
    console.error(`[debug] pipeline complete in ${durationMs}ms`);
  }

  return {
    context,
    docs,
    usedAi,
    durationMs,
  };
}

// ── AI config builder ─────────────────────────────────────────────────────────

/**
 * Build the `AIConfig` for this run by merging in priority order:
 *   CLI flags  >  .repo-intel.toml  >  defaults
 *
 * API key is NOT resolved here — `ai/index.ts` resolves it from env.
 */
function buildAIConfig(
  options: GenerateOptions,
  repoConfig?: Partial<RepoIntelConfig>,
): AIConfig {
  const provider =
    options.provider ?? repoConfig?.ai?.provider ?? DEFAULT_PROVIDER;

  const model = repoConfig?.ai?.model;

  // baseUrl only relevant for Ollama
  const baseUrl =
    provider === "ollama" ? (repoConfig?.ai?.base_url ?? undefined) : undefined;

  return {
    provider,
    model,
    baseUrl,
    // apiKey intentionally omitted — resolved from env in ai/index.ts
  };
}
