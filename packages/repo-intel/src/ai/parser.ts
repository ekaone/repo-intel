import type { AgentDoc, RepoContext, Skill } from "../types";

/** Minimum character count for a response to be considered valid. */
const MIN_CONTENT_LENGTH = 100;

/**
 * Parse a raw LLM response string into a typed `AgentDoc`.
 *
 * Responsibilities:
 * 1. Validate the response is non-empty and meets minimum length.
 * 2. Strip code fences — LLMs sometimes wrap output in ```markdown``` despite
 *    being instructed not to.
 * 3. Strip leading/trailing whitespace.
 * 4. Attach metadata: role, filename, timestamps, confidence.
 */
export function parseAgentResponse(
  rawMarkdown: string,
  role: string,
  context: RepoContext,
): AgentDoc {
  if (!rawMarkdown || rawMarkdown.trim().length === 0) {
    throw new Error(
      `repo-intel: LLM returned an empty response for role '${role}'`,
    );
  }

  const cleaned = cleanMarkdown(rawMarkdown);

  if (cleaned.length < MIN_CONTENT_LENGTH) {
    throw new Error(
      `repo-intel: LLM returned insufficient content for role '${role}' ` +
        `(${cleaned.length} chars, minimum is ${MIN_CONTENT_LENGTH})`,
    );
  }

  return {
    role,
    filename: roleToFilename(role),
    content: cleaned,
    generatedAt: new Date().toISOString(),
    generatedBy: context.stack.framework ?? context.stack.language,
    confidence: averageConfidence(context.stack.skills),
  };
}

// ── Filename generation ───────────────────────────────────────────────────────

/**
 * Convert a role name to a safe kebab-case filename.
 *
 * Examples:
 *   "Fullstack Engineer"          → "fullstack-engineer.md"
 *   "QA & Testing Engineer"       → "qa-testing-engineer.md"
 *   "Platform / Monorepo Engineer"→ "platform-monorepo-engineer.md"
 */
export function roleToFilename(role: string): string {
  return (
    role
      .toLowerCase()
      .replace(/[&/\\]+/g, "-") // & / \ → -
      .replace(/\s+/g, "-") // spaces → -
      .replace(/-{2,}/g, "-") // collapse multiple dashes
      .replace(/^-|-$/g, "") + // strip leading/trailing dashes
    ".md"
  );
}

// ── Markdown cleanup ──────────────────────────────────────────────────────────

/**
 * Strip code fences that LLMs sometimes add despite instructions.
 *
 * Handles:
 *   ```markdown ... ```
 *   ```md ... ```
 *   ``` ... ```      (bare fences)
 */
function cleanMarkdown(raw: string): string {
  let text = raw.trim();

  // Strip opening code fence: ```markdown, ```md, or ```
  text = text.replace(/^```(?:markdown|md)?\n?/i, "");

  // Strip closing code fence
  text = text.replace(/\n?```\s*$/i, "");

  return text.trim();
}

// ── Confidence scoring ────────────────────────────────────────────────────────

/**
 * Compute the average confidence across all detected skills.
 * Returns 0 if there are no skills.
 */
function averageConfidence(skills: Skill[]): number {
  if (skills.length === 0) return 0;

  const sum = skills.reduce((acc, s) => acc + s.confidence, 0);
  return Math.round((sum / skills.length) * 100) / 100; // round to 2dp
}

// ── Batch parsing ─────────────────────────────────────────────────────────────

/**
 * Parse multiple LLM responses — one per agent role.
 * Collects errors without short-circuiting so a single bad response
 * doesn't block the rest of the docs from being written.
 */
export function parseAgentResponses(
  responses: Array<{ role: string; raw: string }>,
  context: RepoContext,
): { docs: AgentDoc[]; errors: Array<{ role: string; error: string }> } {
  const docs: AgentDoc[] = [];
  const errors: Array<{ role: string; error: string }> = [];

  for (const { role, raw } of responses) {
    try {
      docs.push(parseAgentResponse(raw, role, context));
    } catch (err) {
      errors.push({
        role,
        error: err instanceof Error ? err.message : String(err),
      });
    }
  }

  return { docs, errors };
}
