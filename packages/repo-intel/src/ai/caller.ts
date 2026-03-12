import type { AIConfig, AIProvider } from "../types";
import { anthropicProvider } from "./providers/anthropic";
import { openaiProvider } from "./providers/openai";
import { ollamaProvider } from "./providers/ollama";

// ── Provider registry ─────────────────────────────────────────────────────────

const PROVIDERS: Record<string, AIProvider> = {
  anthropic: anthropicProvider,
  openai: openaiProvider,
  ollama: ollamaProvider,
};

/**
 * Resolve the correct `AIProvider` instance from `config.provider`.
 * Throws a clear error if the provider name is unrecognised.
 */
export function resolveProvider(config: AIConfig): AIProvider {
  const provider = PROVIDERS[config.provider];

  if (!provider) {
    throw new Error(
      `repo-intel: unknown AI provider '${config.provider}'.\n` +
        `Supported providers: ${Object.keys(PROVIDERS).join(", ")}`,
    );
  }

  return provider;
}

// ── Retry wrapper ─────────────────────────────────────────────────────────────

/**
 * Call `provider` with exponential backoff retries.
 *
 * Retry schedule (from PLAN.md):
 *   attempt 1 → immediate
 *   attempt 2 → wait 1000ms
 *   attempt 3 → wait 2000ms
 *   attempt 4 → wait 3000ms  (if maxRetries > 3)
 *
 * Non-retryable errors (auth failures, bad requests) are re-thrown immediately
 * on the first attempt to avoid wasting time and API credits.
 */
export async function callWithRetry(
  provider: AIProvider,
  prompt: string,
  config: AIConfig,
  maxRetries = 3,
  debug = false,
): Promise<string> {
  let lastError: unknown;

  for (let attempt = 1; attempt <= maxRetries; attempt++) {
    try {
      if (debug && attempt > 1) {
        console.error(`[debug] retry attempt ${attempt}/${maxRetries}…`);
      }

      const result = await provider.call(prompt, config);
      return result;
    } catch (err) {
      lastError = err;

      // Don't retry on non-retryable errors
      if (isNonRetryable(err)) {
        if (debug) {
          console.error(
            `[debug] non-retryable error, aborting retries: ${errorMessage(err)}`,
          );
        }
        throw err;
      }

      if (attempt === maxRetries) break;

      const waitMs = 1000 * attempt; // 1s, 2s, 3s…
      if (debug) {
        console.error(
          `[debug] attempt ${attempt} failed: ${errorMessage(err)}`,
        );
        console.error(`[debug] waiting ${waitMs}ms before retry…`);
      }

      await sleep(waitMs);
    }
  }

  // All attempts exhausted — throw the last error with context
  throw new Error(
    `repo-intel: AI call failed after ${maxRetries} attempts.\n` +
      `Last error: ${errorMessage(lastError)}`,
  );
}

// ── Convenience: resolve + call in one step ───────────────────────────────────

/**
 * Resolve provider from config and call with retry.
 * This is the main entry point used by `ai/index.ts`.
 */
export async function callAI(
  prompt: string,
  config: AIConfig,
  options: { maxRetries?: number; debug?: boolean } = {},
): Promise<string> {
  const provider = resolveProvider(config);
  return callWithRetry(
    provider,
    prompt,
    config,
    options.maxRetries ?? 3,
    options.debug ?? false,
  );
}

// ── Helpers ───────────────────────────────────────────────────────────────────

function sleep(ms: number): Promise<void> {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

/**
 * Returns true for errors that should NOT be retried:
 * - Authentication failures (401)
 * - Bad request / invalid model (400)
 * - Rate limit exceeded (429) — retrying immediately won't help, but we still
 *   retry with backoff since the wait might clear the limit window.
 *
 * Connection errors, timeouts, and 5xx server errors ARE retried.
 */
function isNonRetryable(err: unknown): boolean {
  if (!(err instanceof Error)) return false;

  const msg = err.message.toLowerCase();

  // Auth errors — retrying won't fix a bad API key
  if (
    msg.includes("401") ||
    msg.includes("authentication") ||
    msg.includes("api key")
  ) {
    return true;
  }

  // Bad request — retrying won't fix a malformed payload
  if (msg.includes("400") || msg.includes("bad request")) {
    return true;
  }

  return false;
}

function errorMessage(err: unknown): string {
  return err instanceof Error ? err.message : String(err);
}
