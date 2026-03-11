import type { AgentDoc, RepoContext } from "../types.js";
import { SKILL_DESCRIPTIONS } from "./static-gen.js";
import Handlebars from "handlebars";

const TEMPLATE = Handlebars.compile(`# {{title}}

## Persona

You are the {{title}} for the **{{repoName}}** repository. {{description}}

## Core Responsibilities

{{#each responsibilities}}
- {{this}}
{{/each}}

## Key Patterns & Conventions

{{#each patterns}}
- {{this}}
{{/each}}

## Common Tasks

{{#each tasks}}
- {{this}}
{{/each}}

## Cautions / Anti-patterns

- Always check the existing codebase before introducing new patterns
- Follow the established conventions in the repository
- Keep changes focused and minimal
`);

/**
 * Generate static AgentDoc[] without calling any LLM.
 */
export function generateStatic(context: RepoContext): AgentDoc[] {
  return context.agent_roles.map((role) => {
    const skills = [
      ...context.stack.languages,
      ...context.stack.frameworks,
      ...context.stack.tooling,
    ];

    const patterns = skills
      .filter((s) => s.confidence >= 0.8)
      .map((s) => SKILL_DESCRIPTIONS[s.name] ?? `Work with ${s.name}`)
      .slice(0, 5);

    const content = TEMPLATE({
      title: role.title,
      repoName: context.name,
      description: role.description,
      responsibilities: [
        "Design and implement features within your domain",
        "Review code changes related to your area",
        "Maintain documentation and tests",
        "Ensure quality and correctness of your module",
      ],
      patterns: patterns.length > 0
        ? patterns
        : ["Follow established conventions in the codebase"],
      tasks: [
        `Add new features to ${context.name}`,
        "Fix bugs and regressions",
        "Refactor and improve code quality",
        "Write and update tests",
      ],
    });

    return {
      role_id: role.id,
      title: role.title,
      content,
    };
  });
}
