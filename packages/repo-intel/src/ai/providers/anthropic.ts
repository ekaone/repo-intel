import Anthropic from "@anthropic-ai/sdk";
import type { AIConfig, AIProvider } from "../../types";
import { DEFAULT_MODELS } from "../../types";

/** Timeout in ms for a single Anthropic API call. */
const TIMEOUT_MS = 60_000;

export const anthropicProvider: AIProvider = {
  async call(prompt: string, config: AIConfig): Promise<string> {
    if (!config.apiKey) {
      throw new Error(
        "repo-intel: Anthropic API key is not set.\n" +
          "Set the ANTHROPIC_API_KEY environment variable and try again.",
      );
    }

    const client = new Anthropic({
      apiKey: config.apiKey,
      timeout: TIMEOUT_MS,
    });

    const model = config.model ?? DEFAULT_MODELS.anthropic;

    let message: Anthropic.Message;

    try {
      message = await client.messages.create({
        model,
        max_tokens: config.maxTokens ?? 2048,
        messages: [{ role: "user", content: prompt }],
      });
    } catch (err) {
      // Re-throw with provider context so caller.ts can log it clearly
      throw new Error(`Anthropic API error: ${errorMessage(err)}`);
    }

    // Extract all text blocks and join — handles multi-block responses
    const text = message.content
      .filter((b): b is Anthropic.TextBlock => b.type === "text")
      .map((b) => b.text)
      .join("");

    if (!text) {
      throw new Error(
        `Anthropic returned an empty response for model '${model}'`,
      );
    }

    return text;
  },
};

function errorMessage(err: unknown): string {
  if (err instanceof Anthropic.APIError) {
    return `${err.status} ${err.name}: ${err.message}`;
  }
  return err instanceof Error ? err.message : String(err);
}
