import OpenAI from "openai";
import type { AIConfig, AIProvider } from "../../types";
import { DEFAULT_MODELS } from "../../types";

/** Timeout in ms for a single OpenAI API call. */
const TIMEOUT_MS = 60_000;

export const openaiProvider: AIProvider = {
  async call(prompt: string, config: AIConfig): Promise<string> {
    if (!config.apiKey) {
      throw new Error(
        "repo-intel: OpenAI API key is not set.\n" +
          "Set the OPENAI_API_KEY environment variable and try again.",
      );
    }

    const client = new OpenAI({
      apiKey: config.apiKey,
      timeout: TIMEOUT_MS,
    });

    const model = config.model ?? DEFAULT_MODELS.openai;

    let response: OpenAI.Chat.ChatCompletion;

    try {
      response = await client.chat.completions.create({
        model,
        max_tokens: config.maxTokens ?? 2048,
        messages: [{ role: "user", content: prompt }],
        // temperature default (1.0) is fine for creative agent doc generation
      });
    } catch (err) {
      throw new Error(`OpenAI API error: ${errorMessage(err)}`);
    }

    const text = response.choices[0]?.message?.content ?? "";

    if (!text) {
      const reason = response.choices[0]?.finish_reason ?? "unknown";
      throw new Error(
        `OpenAI returned an empty response for model '${model}' ` +
          `(finish_reason: ${reason})`,
      );
    }

    return text;
  },
};

function errorMessage(err: unknown): string {
  if (err instanceof OpenAI.APIError) {
    return `${err.status} ${err.name}: ${err.message}`;
  }
  return err instanceof Error ? err.message : String(err);
}
