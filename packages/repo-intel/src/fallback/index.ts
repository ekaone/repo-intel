import Handlebars from "handlebars";
import type { AgentDoc, RepoContext } from "../types";
import { roleToFilename } from "../ai/parser";
import {
  getRoleContent,
  buildStackSummary,
  buildSkillLines,
  buildInfraLine,
} from "./static-gen";
import { AGENT_DOC_TEMPLATE, getRoleWorkflow } from "./templates";

// ── Compiled template (once, not per-call) ────────────────────────────────────
const compiledTemplate = Handlebars.compile(AGENT_DOC_TEMPLATE, {
  noEscape: true,
});

// ── Main export ───────────────────────────────────────────────────────────────

/**
 * Generate static agent docs for every role in `context.agent_roles`
 * without making any LLM API calls.
 *
 * Output is structurally identical to `generateAgentDocs()` so the
 * pipeline layer can treat both paths interchangeably.
 *
 * Used by: `--no-ai` flag, CI environments, and offline development.
 */
export function generateStatic(
  context: RepoContext,
  options: { debug?: boolean } = {},
): AgentDoc[] {
  const { debug = false } = options;
  const roles = context.agent_roles;

  if (roles.length === 0) {
    if (debug) {
      console.error(
        "[debug] no agent roles detected — nothing to generate (static mode)",
      );
    }
    return [];
  }

  if (debug) {
    console.error(
      `[debug] static mode: generating ${roles.length} role(s): ${roles.join(", ")}`,
    );
  }

  return roles.map((role) => generateStaticDoc(role, context));
}

// ── Single doc generation ─────────────────────────────────────────────────────

function generateStaticDoc(role: string, context: RepoContext): AgentDoc {
  const { project, stack, architecture, agent_roles } = context;

  // ── Role content (identity, personality, memory, experience) ─────────────
  const roleContent = getRoleContent(role);
  const roleWorkflow = getRoleWorkflow(role);

  // ── Pre-rendered helpers ──────────────────────────────────────────────────
  const stackSummary = buildStackSummary(context);
  const skillLines = buildSkillLines(context);
  const infraFlags = buildInfraLine(context);
  const folders =
    architecture.folders.length > 0
      ? architecture.folders.map((f) => `  ${f}/`).join("\n")
      : "  (no significant folders detected)";

  // ── Other roles (for team context section) ────────────────────────────────
  const otherRoles = agent_roles
    .filter((r) => r !== role)
    .map((r) => `- [${r}](./${roleToFilename(r)})`)
    .join("\n");

  // ── Template variables ────────────────────────────────────────────────────
  const vars: Record<string, unknown> = {
    // Role
    role,

    // Project
    projectName: project.name,
    projectDesc: project.description ?? "",

    // Stack
    stackSummary,
    language: stack.language,
    framework: stack.framework ?? "None detected",
    styling: stack.styling ?? "None detected",
    stateManagement: stack.state_management ?? "None detected",
    testing: stack.testing ?? "None detected",
    database: stack.database ?? "None detected",
    runtime: stack.runtime ?? "None detected",

    // Architecture
    folders,
    infraFlags,
    skillLines,

    // Role content
    identity: roleContent.identity,
    personality: roleContent.personality,
    memory: roleContent.memory,
    experience: roleContent.experience,

    // Workflow
    responsibilities: roleWorkflow.responsibilities,
    workflowSteps: roleWorkflow.workflowSteps,
    rules: roleWorkflow.rules,
    metrics: roleWorkflow.metrics,

    // Team context
    otherRoles: otherRoles || null,

    // Metadata
    generatedAt: new Date().toUTCString(),
  };

  const content = compiledTemplate(vars);

  return {
    role,
    filename: roleToFilename(role),
    content,
    generatedAt: new Date().toISOString(),
    generatedBy: stack.framework ?? stack.language,
    confidence: averageConfidence(stack.skills),
  };
}

// ── Helpers ───────────────────────────────────────────────────────────────────

function averageConfidence(skills: RepoContext["stack"]["skills"]): number {
  if (skills.length === 0) return 0;
  const sum = skills.reduce((acc, s) => acc + s.confidence, 0);
  return Math.round((sum / skills.length) * 100) / 100;
}
