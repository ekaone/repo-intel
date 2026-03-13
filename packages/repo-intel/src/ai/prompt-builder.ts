import type { RepoContext, Skill } from "../types";

// ── Confidence thresholds (mirrors Rust constants) ────────────────────────────

const PRIMARY_THRESHOLD = 0.9;
const SECONDARY_THRESHOLD = 0.5;

// ── Role-specific focus hints ─────────────────────────────────────────────────
// These are injected into the prompt to steer the LLM toward role-relevant
// responsibilities. Iterate on these as output quality is reviewed.

const ROLE_HINTS: Record<string, string> = {
  "Fullstack Engineer":
    "Focus on the full request lifecycle: frontend rendering, API routes, " +
    "database queries, and deployment. Reference both the UI layer and server layer.",

  "Frontend Engineer":
    "Focus on component architecture, UI patterns, styling approach, and " +
    "client-side state management. Reference the component folder structure directly.",

  "Backend API Engineer":
    "Focus on API design, request validation, error handling, database access " +
    "patterns, and service layer architecture.",

  "QA & Testing Engineer":
    "Focus on test strategy, test coverage, tooling setup, and quality gates. " +
    "Reference the actual testing framework and existing test file patterns.",

  "Database Engineer":
    "Focus on schema design, query optimization, migration strategy, and ORM " +
    "usage patterns. Reference the specific ORM and database detected.",

  "DevOps Engineer":
    "Focus on CI/CD pipeline, containerization, environment configuration, " +
    "deployment strategy, and infrastructure as code.",

  "GraphQL Engineer":
    "Focus on schema design, resolver patterns, query optimization, and " +
    "client-server contract. Reference the specific GraphQL tooling detected.",

  "Platform / Monorepo Engineer":
    "Focus on workspace tooling, package interdependencies, shared tooling, " +
    "build orchestration, and developer experience across the monorepo.",

  "Developer Tooling Engineer":
    "Focus on CLI design, binary distribution, developer ergonomics, and " +
    "cross-platform compatibility.",

  "WebAssembly Engineer":
    "Focus on Rust→WASM compilation, JS interop, performance characteristics, " +
    "and browser or Node.js runtime integration.",
};

/** Fallback hint for roles not in the map above. */
const DEFAULT_ROLE_HINT =
  "Focus on the responsibilities most relevant to this role given the detected stack.";

// ── Main export ───────────────────────────────────────────────────────────────

/**
 * Build a rich, project-aware LLM prompt for a single agent role.
 *
 * This is the most important function in the entire project.
 * Output quality is 100% determined here.
 *
 * ⚠️  ITERATION NOTE: You will change this function many times.
 * The structure, detail level, section order, and role hints will all
 * be refined based on real output quality review. This is expected and normal.
 */
export function buildPrompt(context: RepoContext, role: string): string {
  const { project, stack, architecture, agent_roles } = context;

  // ── Skill lists ─────────────────────────────────────────────────────────────
  const primarySkills = filterSkills(stack.skills, PRIMARY_THRESHOLD, 1.01);
  const secondarySkills = filterSkills(
    stack.skills,
    SECONDARY_THRESHOLD,
    PRIMARY_THRESHOLD,
  );

  const primarySkillNames =
    primarySkills.map((s) => s.name).join(", ") || "None detected";
  const secondarySkillNames =
    secondarySkills.map((s) => s.name).join(", ") || "None detected";

  // ── Architecture style label ─────────────────────────────────────────────────
  const archStyleLabel = formatArchStyle(architecture.style);

  // ── Other agent roles (for cross-agent context) ───────────────────────────
  const otherRoles = agent_roles.filter((r) => r !== role);
  const otherRolesLine =
    otherRoles.length > 0
      ? otherRoles.join(", ")
      : "None (this is the only agent for this project)";

  // ── Infra flags ──────────────────────────────────────────────────────────────
  const infraFlags = buildInfraFlags(architecture);

  // ── Role-specific focus hint ─────────────────────────────────────────────────
  const roleHint = ROLE_HINTS[role] ?? DEFAULT_ROLE_HINT;

  // ── Assemble prompt ───────────────────────────────────────────────────────────
  return [
    buildSystemSection(),
    buildProjectSection(project),
    buildStackSection(stack),
    buildArchitectureSection(archStyleLabel, architecture.folders, infraFlags),
    buildSkillsSection(primarySkillNames, secondarySkillNames),
    buildTeamSection(role, otherRolesLine),
    buildInstructionsSection(role, roleHint),
  ].join("\n\n");
}

// ── Prompt section builders ───────────────────────────────────────────────────
// Each section is a pure function so they can be unit-tested and swapped
// independently during prompt iteration.

function buildSystemSection(): string {
  return `You are an expert technical writer specialising in AI agent persona definitions for software development teams. Your agent docs are used by AI coding assistants like Claude, Cursor, and GitHub Copilot to understand their role, expertise, and workflow within a specific project.`;
}

function buildProjectSection(project: RepoContext["project"]): string {
  return `## Project Context
- **Name:** ${project.name}
- **Description:** ${project.description ?? "Not specified"}
- **README excerpt:** ${project.readme_excerpt ? `\n\n> ${project.readme_excerpt.split("\n").slice(0, 5).join("\n> ")}` : "Not available"}`;
}

function buildStackSection(stack: RepoContext["stack"]): string {
  return `## Detected Tech Stack
- **Language:** ${stack.language}
- **Framework:** ${stack.framework ?? "None detected"}
- **Runtime:** ${stack.runtime ?? "None detected"}
- **Styling:** ${stack.styling ?? "None detected"}
- **State management:** ${stack.state_management ?? "None detected"}
- **Testing:** ${stack.testing ?? "None detected"}
- **Database:** ${stack.database ?? "None detected"}`;
}

function buildArchitectureSection(
  style: string,
  folders: string[],
  infraFlags: string,
): string {
  const folderList =
    folders.length > 0
      ? folders.map((f) => `\`${f}/\``).join(", ")
      : "No significant folders detected";

  return `## Repository Architecture
- **Style:** ${style}
- **Key folders:** ${folderList}
- **Infrastructure:** ${infraFlags}`;
}

function buildSkillsSection(primary: string, secondary: string): string {
  return `## Detected Skills
- **Primary (confidence ≥ 0.90):** ${primary}
- **Secondary (confidence 0.50–0.89):** ${secondary}`;
}

function buildTeamSection(role: string, otherRoles: string): string {
  return `## Agent Team Context
- **This agent's role:** ${role}
- **Other agents in this project:** ${otherRoles}

When writing the Workflow section, reference how this agent collaborates with or hands off to the other agents listed above.`;
}

function buildInstructionsSection(role: string, roleHint: string): string {
  return `## Your Task

Generate a complete agent documentation file in Markdown for the role: **${role}**

**Role focus:** ${roleHint}

The document must contain exactly these sections, in this order:

1. **Role title and one-line description** — A bold \`# Title\` heading and a single sharp sentence describing what this agent does.
2. **Identity** — Who this agent is: their expertise, specialisation, and what makes them uniquely valuable to this project.
3. **Personality** — Working style, values, communication approach, and how they handle trade-offs and disagreements.
4. **Memory** — What patterns, decisions, and institutional knowledge this agent retains across sessions. Make these specific to the detected stack.
5. **Experience** — The kinds of problems this agent has solved in projects like this one. Reference the actual tech stack detected.
6. **Core Responsibilities** — 5–8 bullet points specific to this project's stack and folder structure. Not generic job descriptions.
7. **Workflow** — A numbered step-by-step process this agent follows, referencing actual folder names, tools, and file patterns detected above.
8. **Deliverables** — Concrete outputs with project-specific details, e.g. actual framework versions, folder paths, file naming conventions.
9. **Rules** — 5–8 hard constraints this agent always follows. Include at least one rule about code quality and one about collaboration with other agents.
10. **Metrics** — 3–5 measurable success criteria relevant to this specific role and stack.

**Voice: always second person ("You are...", "You own...", "You retain...").**
Never use first person ("I am", "I own", "I retain"). The document is read by an AI
assistant that must adopt this persona — second person makes that adoption direct and unambiguous.

**Quality requirements:**
- **Be specific** — reference the actual stack, folder names, and domain context. Never use placeholders like "[framework]" or "[tool]".
- **Be opinionated** — the agent should have a clear, strong point of view shaped by the detected stack.
- **Be practical** — workflow steps must be immediately actionable, not aspirational.
- **Be rich** — aim for 500–700 words of substantive content across all sections.

Output ONLY the Markdown content. No preamble, no explanation, no code fences around the output.`;
}

// ── Helpers ───────────────────────────────────────────────────────────────────

function filterSkills(
  skills: Skill[],
  minConf: number,
  maxConf: number,
): Skill[] {
  return skills.filter(
    (s) =>
      s.confidence >= minConf &&
      s.confidence < maxConf &&
      // Exclude internal __ markers that leaked through
      !s.name.startsWith("__"),
  );
}

function formatArchStyle(style: RepoContext["architecture"]["style"]): string {
  switch (style) {
    case "feature_based":
      return "Feature-based (modules/features at top level)";
    case "layer_based":
      return "Layer-based (components/services/hooks separated)";
    case "flat":
      return "Flat (minimal structure)";
    default:
      return "Standard";
  }
}

function buildInfraFlags(arch: RepoContext["architecture"]): string {
  const flags: string[] = [];
  if (arch.has_docker) flags.push("Docker");
  if (arch.has_ci) flags.push("CI/CD");
  if (arch.has_monorepo) flags.push("Monorepo");
  if (arch.has_git) flags.push("Git");
  return flags.length > 0 ? flags.join(", ") : "None detected";
}
