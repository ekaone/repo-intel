import type { AIConfig, AIProvider } from "../../types";
import { DEFAULT_MODELS } from "../../types";

/** Default Ollama base URL — local server. */
const DEFAULT_BASE_URL = "http://localhost:11434";

/** Timeout in ms — longer than cloud providers since local models are slower. */
const TIMEOUT_MS = 120_000;

// No SDK needed — Ollama exposes a simple REST API
export const ollamaProvider: AIProvider = {
  async call(prompt: string, config: AIConfig): Promise<string> {
    const baseUrl = (config.baseUrl ?? DEFAULT_BASE_URL).replace(/\/$/, "");
    const model = config.model ?? DEFAULT_MODELS.ollama;

    let response: Response;

    try {
      const controller = new AbortController();
      const timer = setTimeout(() => controller.abort(), TIMEOUT_MS);

      response = await fetch(`${baseUrl}/api/generate`, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({
          model,
          prompt,
          stream: false, // We want the full response, not a stream
          options: {
            num_predict: config.maxTokens ?? 2048,
          },
        }),
        signal: controller.signal,
      });

      clearTimeout(timer);
    } catch (err) {
      if (err instanceof Error && err.name === "AbortError") {
        throw new Error(
          `Ollama request timed out after ${TIMEOUT_MS / 1000}s.\n` +
            `Is the Ollama server running at ${baseUrl}?`,
        );
      }
      throw new Error(
        `Ollama connection failed: ${errorMessage(err)}\n` +
          `Is the Ollama server running at ${baseUrl}?\n` +
          `Start it with: ollama serve`,
      );
    }

    if (!response.ok) {
      const body = await response.text().catch(() => "");
      throw new Error(
        `Ollama API error ${response.status}: ${body || response.statusText}\n` +
          `Model requested: ${model}\n` +
          `Tip: run 'ollama pull ${model}' to download the model first.`,
      );
    }

    let data: OllamaGenerateResponse;

    try {
      data = (await response.json()) as OllamaGenerateResponse;
    } catch (err) {
      throw new Error(`Ollama returned invalid JSON: ${errorMessage(err)}`);
    }

    const text = data.response ?? "";

    if (!text) {
      throw new Error(
        `Ollama returned an empty response for model '${model}'.\n` +
          `Try a different model with --model <name>.`,
      );
    }

    return text;
  },
};

// ── Ollama API response shape ──────────────────────────────────────────────────
// Only the fields we care about — the actual response has more.

interface OllamaGenerateResponse {
  model: string;
  response: string;
  done: boolean;
  done_reason?: string;
  total_duration?: number;
  eval_count?: number;
}

function errorMessage(err: unknown): string {
  return err instanceof Error ? err.message : String(err);
}
