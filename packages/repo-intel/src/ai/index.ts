import type { AgentDoc, AIConfig, RepoContext } from "../types";
import { buildPrompt } from "./prompt-builder";
import { callAI } from "./caller";
import { parseAgentResponse } from "./parser";
import { API_KEY_ENV_VARS } from "../types";

// ── Options ───────────────────────────────────────────────────────────────────

export interface GenerateAgentDocsOptions {
  /** Max retries per role on transient failures. Default: 3 */
  maxRetries?: number;
  /** Emit debug info to stderr. Default: false */
  debug?: boolean;
  /**
   * Called after each role is successfully generated.
   * Useful for CLI progress indicators.
   */
  onProgress?: (role: string, index: number, total: number) => void;
}

// ── Main export ───────────────────────────────────────────────────────────────

/**
 * Generate agent documentation for every role in `context.agent_roles`.
 *
 * Flow per role:
 *   1. buildPrompt(context, role)  → prompt string
 *   2. callAI(prompt, config)      → raw LLM markdown
 *   3. parseAgentResponse(raw)     → AgentDoc
 *
 * Roles that fail after all retries are collected in `errors` rather than
 * aborting the entire run — a partial result is better than nothing.
 *
 * @throws if `config.apiKey` is missing for providers that require one.
 */
export async function generateAgentDocs(
  context: RepoContext,
  config: AIConfig,
  options: GenerateAgentDocsOptions = {},
): Promise<{ docs: AgentDoc[]; errors: GenerationError[] }> {
  const { maxRetries = 3, debug = false, onProgress } = options;

  // ── Pre-flight: resolve API key from env if not already set ────────────────
  const resolvedConfig = resolveApiKey(config, debug);

  const roles = context.agent_roles;

  if (roles.length === 0) {
    if (debug) {
      console.error("[debug] no agent roles detected — nothing to generate");
    }
    return { docs: [], errors: [] };
  }

  if (debug) {
    console.error(
      `[debug] generating docs for ${roles.length} role(s): ${roles.join(", ")}`,
    );
    console.error(`[debug] provider: ${resolvedConfig.provider}`);
    console.error(
      `[debug] model:    ${resolvedConfig.model ?? "(provider default)"}`,
    );
  }

  const docs: AgentDoc[] = [];
  const errors: GenerationError[] = [];

  // ── Generate one doc per role (sequential — avoids rate limit bursts) ──────
  for (let i = 0; i < roles.length; i++) {
    const role = roles[i]!;

    if (debug) {
      console.error(`[debug] [${i + 1}/${roles.length}] generating: ${role}`);
    }

    try {
      const doc = await generateOneRole(role, context, resolvedConfig, {
        maxRetries,
        debug,
      });

      docs.push(doc);
      onProgress?.(role, i + 1, roles.length);

      if (debug) {
        console.error(`[debug] ✓ ${role} — ${doc.content.length} chars`);
      }
    } catch (err) {
      const message = err instanceof Error ? err.message : String(err);

      errors.push({ role, error: message });

      // Always log generation errors — they're non-fatal but important
      console.error(`warn: failed to generate doc for '${role}': ${message}`);
    }
  }

  if (debug) {
    console.error(
      `[debug] generation complete: ${docs.length} succeeded, ${errors.length} failed`,
    );
  }

  return { docs, errors };
}

// ── Single role generation ────────────────────────────────────────────────────

async function generateOneRole(
  role: string,
  context: RepoContext,
  config: AIConfig,
  options: { maxRetries: number; debug: boolean },
): Promise<AgentDoc> {
  // Step 1 — build the prompt
  const prompt = buildPrompt(context, role);

  if (options.debug) {
    console.error(
      `[debug] prompt length for '${role}': ${prompt.length} chars`,
    );
  }

  // Step 2 — call the LLM with retries
  const rawMarkdown = await callAI(prompt, config, {
    maxRetries: options.maxRetries,
    debug: options.debug,
  });

  // Step 3 — parse and validate the response
  const doc = parseAgentResponse(rawMarkdown, role, context);

  return doc;
}

// ── API key resolution ────────────────────────────────────────────────────────

/**
 * Resolve the API key from the environment if not already present in config.
 * Ollama is local and requires no key.
 *
 * API keys are NEVER read from disk or hardcoded — env vars only.
 */
function resolveApiKey(config: AIConfig, debug: boolean): AIConfig {
  if (config.provider === "ollama") {
    return config; // no key needed
  }

  if (config.apiKey) {
    return config; // already resolved upstream
  }

  const envVar = API_KEY_ENV_VARS[config.provider];

  if (!envVar) {
    return config;
  }

  const key = process.env[envVar];

  if (!key) {
    throw new Error(
      `repo-intel: API key not found for provider '${config.provider}'.\n` +
        `Set the ${envVar} environment variable and try again.\n` +
        `Example: export ${envVar}=your-key-here`,
    );
  }

  if (debug) {
    console.error(`[debug] resolved API key from env var: ${envVar}`);
  }

  return { ...config, apiKey: key };
}

// ── Error type ────────────────────────────────────────────────────────────────

export interface GenerationError {
  role: string;
  error: string;
}
