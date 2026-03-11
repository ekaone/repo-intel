import type { AgentRole, RepoContext } from "../types.js";

/**
 * Build the LLM prompt for a specific agent role.
 * This is the most-iterated file — keep the prompt structured but concise.
 */
export function buildPrompt(context: RepoContext, role: AgentRole): string {
  const stack = context.stack;

  const languages = stack.languages.map((l) => `${l.name} (conf: ${l.confidence})`).join(", ");
  const frameworks = stack.frameworks.map((f) => f.name).join(", ");
  const tooling = stack.tooling.map((t) => t.name).join(", ");
  const architecture = stack.architecture.join(", ");

  const readme = context.readme_excerpt
    ? `\n## README Excerpt\n${context.readme_excerpt}\n`
    : "";

  return `You are generating technical documentation for an AI coding agent.

## Repository: ${context.name}

### Detected Stack
- Languages: ${languages || "unknown"}
- Frameworks: ${frameworks || "none detected"}
- Tooling: ${tooling || "none detected"}
- Architecture patterns: ${architecture || "none detected"}
- Has Git: ${context.has_git}
- Has Docker: ${context.has_docker}
- Has CI: ${context.has_ci}
${readme}
### Agent Role: ${role.title}
${role.description}

## Task
Write a comprehensive, LLM-optimised agent documentation file for the "${role.title}" role in this repository.

The document should:
1. Start with a one-paragraph persona description
2. Include a "Core Responsibilities" section (bullet list)
3. Include a "Key Patterns & Conventions" section specific to the detected stack
4. Include a "Common Tasks" section with example prompts the agent should handle well
5. Include a "Cautions / Anti-patterns" section
6. Be written in second-person ("You are responsible for...")

Format: plain markdown, no code fences wrapping the entire output.
Aim for 300–500 words. Be specific to the detected stack, not generic.`;
}
