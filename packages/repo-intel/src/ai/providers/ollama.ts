/**
 * Ollama REST API provider — local LLM, free, offline.
 * Defaults to http://localhost:11434 and model "llama3".
 */
export async function callOllama(
  prompt: string,
  model?: string,
): Promise<string> {
  const baseUrl = process.env.OLLAMA_BASE_URL ?? "http://localhost:11434";
  const resolvedModel = model ?? process.env.OLLAMA_MODEL ?? "llama3";

  const response = await fetch(`${baseUrl}/api/generate`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({
      model: resolvedModel,
      prompt,
      stream: false,
    }),
  });

  if (!response.ok) {
    throw new Error(
      `Ollama request failed: ${response.status} ${response.statusText}`,
    );
  }

  const json = (await response.json()) as { response?: string; error?: string };

  if (json.error) {
    throw new Error(`Ollama error: ${json.error}`);
  }

  if (!json.response) {
    throw new Error("Ollama returned an empty response");
  }

  return json.response;
}
