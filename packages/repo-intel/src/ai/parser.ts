import type { AgentDoc, AgentRole } from "../types.js";

/**
 * Parse raw LLM response text into an AgentDoc.
 * Handles both fenced and plain markdown responses.
 */
export function parseResponse(rawText: string, role: AgentRole): AgentDoc {
  // Strip any outer markdown code fence if the LLM wrapped the entire response
  let content = rawText.trim();

  if (content.startsWith("```markdown")) {
    content = content.slice("```markdown".length);
  } else if (content.startsWith("```")) {
    content = content.slice(3);
  }

  if (content.endsWith("```")) {
    content = content.slice(0, -3).trim();
  }

  // Ensure the document starts with the role title as an H1
  if (!content.startsWith("# ")) {
    content = `# ${role.title}\n\n${content}`;
  }

  return {
    role_id: role.id,
    title: role.title,
    content,
  };
}
