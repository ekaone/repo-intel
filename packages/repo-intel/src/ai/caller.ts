import type { AiProvider } from "../types.js";
import { callAnthropic } from "./providers/anthropic.js";
import { callOpenAI } from "./providers/openai.js";
import { callOllama } from "./providers/ollama.js";

export interface CallOptions {
  provider: AiProvider;
  model?: string;
  maxRetries?: number;
  timeoutMs?: number;
}

/**
 * Provider-agnostic call with retries and timeout.
 */
export async function callProvider(
  prompt: string,
  opts: CallOptions,
): Promise<string> {
  const maxRetries = opts.maxRetries ?? 3;
  const timeoutMs = opts.timeoutMs ?? 30_000;

  let lastError: unknown;

  for (let attempt = 1; attempt <= maxRetries; attempt++) {
    try {
      const result = await Promise.race([
        dispatch(prompt, opts),
        timeout(timeoutMs),
      ]);
      return result;
    } catch (err) {
      lastError = err;
      if (attempt < maxRetries) {
        await sleep(1000 * attempt); // exponential-ish backoff
      }
    }
  }

  throw new Error(`AI call failed after ${maxRetries} attempts: ${lastError}`);
}

function dispatch(prompt: string, opts: CallOptions): Promise<string> {
  switch (opts.provider) {
    case "anthropic":
      return callAnthropic(prompt, opts.model);
    case "openai":
      return callOpenAI(prompt, opts.model);
    case "ollama":
      return callOllama(prompt, opts.model);
    default:
      throw new Error(`Unknown AI provider: ${opts.provider}`);
  }
}

function timeout(ms: number): Promise<never> {
  return new Promise((_, reject) =>
    setTimeout(() => reject(new Error(`AI call timed out after ${ms}ms`)), ms),
  );
}

function sleep(ms: number): Promise<void> {
  return new Promise((resolve) => setTimeout(resolve, ms));
}
