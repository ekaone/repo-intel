import type { AgentDoc, AiProvider, RepoContext } from "../types.js";
import { callProvider } from "./caller.js";
import { buildPrompt } from "./prompt-builder.js";
import { parseResponse } from "./parser.js";

export interface AiOptions {
  provider?: AiProvider | string;
  model?: string;
}

/**
 * Generate AgentDoc[] for every role in the context using the configured LLM.
 */
export async function generateAgentDocs(
  context: RepoContext,
  opts: AiOptions,
): Promise<AgentDoc[]> {
  const provider = (opts.provider ?? process.env.REPO_INTEL_PROVIDER ?? "anthropic") as AiProvider;

  const docs = await Promise.all(
    context.agent_roles.map(async (role) => {
      const prompt = buildPrompt(context, role);
      const responseText = await callProvider(prompt, { provider, model: opts.model });
      const doc = parseResponse(responseText, role);
      return doc;
    }),
  );

  return docs;
}
