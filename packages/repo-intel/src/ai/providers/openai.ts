/**
 * OpenAI provider — calls gpt-4o by default.
 * Requires OPENAI_API_KEY.
 */
export async function callOpenAI(
  prompt: string,
  model?: string,
): Promise<string> {
  const apiKey = process.env.OPENAI_API_KEY;
  if (!apiKey) {
    throw new Error(
      "OPENAI_API_KEY environment variable is not set. " +
        "See docs/providers.md for setup instructions.",
    );
  }

  const { default: OpenAI } = await import("openai");
  const client = new OpenAI({ apiKey });

  const response = await client.chat.completions.create({
    model: model ?? "gpt-4o",
    messages: [{ role: "user", content: prompt }],
    max_tokens: 1024,
  });

  const text = response.choices[0]?.message?.content;
  if (!text) {
    throw new Error("OpenAI response contained no content");
  }

  return text;
}
