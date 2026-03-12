import { describe, it, expect } from "vitest";
import {
  parseAgentResponse,
  parseAgentResponses,
  roleToFilename,
} from "../src/ai/parser";
import type { RepoContext } from "../src/types";

// ── Shared fixture ────────────────────────────────────────────────────────────

/** Minimal RepoContext sufficient for parser tests. */
function makeContext(overrides: Partial<RepoContext> = {}): RepoContext {
  return {
    version: "0.1.0",
    scanned_at: "2026-03-12T00:00:00Z",
    root: "/projects/my-app",
    project: {
      name: "my-app",
      description: "A dashboard for monitoring IoT devices",
      readme_excerpt: "my-app is a real-time IoT dashboard built with Next.js.",
    },
    stack: {
      language: "TypeScript",
      framework: "Next.js",
      styling: "Tailwind CSS",
      state_management: "Zustand",
      testing: "Vitest",
      database: "Prisma",
      runtime: "Node.js",
      architecture_style: "layer_based",
      skills: [
        { name: "React", confidence: 0.99, source: { type: "package_json" } },
        { name: "Next.js", confidence: 0.99, source: { type: "package_json" } },
        { name: "Prisma", confidence: 0.97, source: { type: "package_json" } },
        {
          name: "Testing",
          confidence: 0.8,
          source: { type: "file_pattern", value: "*.test.ts" },
        },
      ],
    },
    architecture: {
      style: "layer_based",
      folders: ["components", "hooks", "services", "pages"],
      has_monorepo: false,
      has_docker: true,
      has_ci: true,
      has_git: true,
    },
    agent_roles: ["Fullstack Engineer", "QA & Testing Engineer"],
    ...overrides,
  };
}

/** A valid markdown response that meets the minimum length requirement. */
const VALID_MARKDOWN = `# Fullstack Engineer

A full-stack engineer who owns the complete request lifecycle from React component to database query.

## Identity

This agent is responsible for building end-to-end features in the my-app project using TypeScript and Next.js.

## Core Responsibilities

- Build and maintain API routes in the Next.js pages/api directory
- Write React components that consume backend data cleanly
- Manage Prisma schema changes and migrations
- Write Vitest tests for both API routes and React components
`.trim();

// ── roleToFilename() ──────────────────────────────────────────────────────────

describe("roleToFilename()", () => {
  it("converts simple role names to kebab-case", () => {
    expect(roleToFilename("Fullstack Engineer")).toBe("fullstack-engineer.md");
    expect(roleToFilename("Frontend Engineer")).toBe("frontend-engineer.md");
    expect(roleToFilename("Backend API Engineer")).toBe(
      "backend-api-engineer.md",
    );
  });

  it("handles ampersand in role names", () => {
    expect(roleToFilename("QA & Testing Engineer")).toBe(
      "qa-testing-engineer.md",
    );
  });

  it("handles slash in role names", () => {
    expect(roleToFilename("Platform / Monorepo Engineer")).toBe(
      "platform-monorepo-engineer.md",
    );
  });

  it("collapses multiple consecutive dashes", () => {
    // & + space would naively produce double dashes without the collapse rule
    const result = roleToFilename("QA & Testing Engineer");
    expect(result).not.toContain("--");
  });

  it("always ends with .md", () => {
    for (const role of [
      "Fullstack Engineer",
      "DevOps Engineer",
      "GraphQL Engineer",
    ]) {
      expect(roleToFilename(role)).toMatch(/\.md$/);
    }
  });

  it("is fully lowercase", () => {
    const result = roleToFilename("Fullstack Engineer");
    expect(result).toBe(result.toLowerCase());
  });
});

// ── parseAgentResponse() — happy path ────────────────────────────────────────

describe("parseAgentResponse() — happy path", () => {
  it("returns an AgentDoc with correct role and filename", () => {
    const doc = parseAgentResponse(
      VALID_MARKDOWN,
      "Fullstack Engineer",
      makeContext(),
    );
    expect(doc.role).toBe("Fullstack Engineer");
    expect(doc.filename).toBe("fullstack-engineer.md");
  });

  it("content is trimmed and matches the input", () => {
    const doc = parseAgentResponse(
      VALID_MARKDOWN,
      "Fullstack Engineer",
      makeContext(),
    );
    expect(doc.content).toBe(VALID_MARKDOWN);
  });

  it("generatedAt is a valid ISO 8601 timestamp", () => {
    const doc = parseAgentResponse(
      VALID_MARKDOWN,
      "Fullstack Engineer",
      makeContext(),
    );
    expect(() => new Date(doc.generatedAt)).not.toThrow();
    expect(new Date(doc.generatedAt).getFullYear()).toBeGreaterThanOrEqual(
      2026,
    );
  });

  it("generatedBy uses framework when available", () => {
    const doc = parseAgentResponse(
      VALID_MARKDOWN,
      "Fullstack Engineer",
      makeContext(),
    );
    expect(doc.generatedBy).toBe("Next.js");
  });

  it("generatedBy falls back to language when no framework", () => {
    const ctx = makeContext();
    ctx.stack.framework = null;
    const doc = parseAgentResponse(VALID_MARKDOWN, "Fullstack Engineer", ctx);
    expect(doc.generatedBy).toBe("TypeScript");
  });

  it("confidence is between 0 and 1", () => {
    const doc = parseAgentResponse(
      VALID_MARKDOWN,
      "Fullstack Engineer",
      makeContext(),
    );
    expect(doc.confidence).toBeGreaterThanOrEqual(0);
    expect(doc.confidence).toBeLessThanOrEqual(1);
  });

  it("confidence is 0 when context has no skills", () => {
    const ctx = makeContext();
    ctx.stack.skills = [];
    const doc = parseAgentResponse(VALID_MARKDOWN, "Fullstack Engineer", ctx);
    expect(doc.confidence).toBe(0);
  });

  it("confidence is the average of skill confidences, rounded to 2dp", () => {
    const ctx = makeContext();
    ctx.stack.skills = [
      { name: "React", confidence: 0.99, source: { type: "package_json" } },
      { name: "Prisma", confidence: 0.97, source: { type: "package_json" } },
    ];
    const doc = parseAgentResponse(VALID_MARKDOWN, "Fullstack Engineer", ctx);
    expect(doc.confidence).toBe(0.98);
  });
});

// ── parseAgentResponse() — code fence stripping ───────────────────────────────

describe("parseAgentResponse() — code fence stripping", () => {
  it("strips ```markdown opening and closing fences", () => {
    const fenced = "```markdown\n" + VALID_MARKDOWN + "\n```";
    const doc = parseAgentResponse(fenced, "Fullstack Engineer", makeContext());
    expect(doc.content).not.toContain("```markdown");
    expect(doc.content).not.toMatch(/```\s*$/);
    expect(doc.content).toBe(VALID_MARKDOWN);
  });

  it("strips ```md opening fence", () => {
    const fenced = "```md\n" + VALID_MARKDOWN + "\n```";
    const doc = parseAgentResponse(fenced, "Fullstack Engineer", makeContext());
    expect(doc.content).not.toContain("```md");
    expect(doc.content).toBe(VALID_MARKDOWN);
  });

  it("strips bare ``` opening fence", () => {
    const fenced = "```\n" + VALID_MARKDOWN + "\n```";
    const doc = parseAgentResponse(fenced, "Fullstack Engineer", makeContext());
    expect(doc.content).toBe(VALID_MARKDOWN);
  });

  it("handles content without fences unchanged", () => {
    const doc = parseAgentResponse(
      VALID_MARKDOWN,
      "Fullstack Engineer",
      makeContext(),
    );
    expect(doc.content).toBe(VALID_MARKDOWN);
  });
});

// ── parseAgentResponse() — validation errors ─────────────────────────────────

describe("parseAgentResponse() — validation errors", () => {
  it("throws on empty string", () => {
    expect(() =>
      parseAgentResponse("", "Fullstack Engineer", makeContext()),
    ).toThrow(/empty response/);
  });

  it("throws on whitespace-only string", () => {
    expect(() =>
      parseAgentResponse("   \n\t  ", "Fullstack Engineer", makeContext()),
    ).toThrow(/empty response/);
  });

  it("throws on content shorter than minimum (100 chars)", () => {
    const tooShort = "# Title\n\nToo short.";
    expect(() =>
      parseAgentResponse(tooShort, "Fullstack Engineer", makeContext()),
    ).toThrow(/insufficient content/);
  });

  it("error message includes the role name", () => {
    expect(() =>
      parseAgentResponse("", "QA & Testing Engineer", makeContext()),
    ).toThrow("QA & Testing Engineer");
  });

  it("does not throw on content exactly at the 100 char minimum", () => {
    const atMin = "x".repeat(100);
    // Should not throw (cleaned length = 100 = MIN_CONTENT_LENGTH)
    expect(() =>
      parseAgentResponse(atMin, "Fullstack Engineer", makeContext()),
    ).not.toThrow();
  });
});

// ── parseAgentResponses() — batch helper ─────────────────────────────────────

describe("parseAgentResponses()", () => {
  it("parses all valid responses", () => {
    const responses = [
      { role: "Fullstack Engineer", raw: VALID_MARKDOWN },
      { role: "QA & Testing Engineer", raw: VALID_MARKDOWN },
    ];
    const { docs, errors } = parseAgentResponses(responses, makeContext());
    expect(docs).toHaveLength(2);
    expect(errors).toHaveLength(0);
  });

  it("collects errors without aborting the batch", () => {
    const responses = [
      { role: "Fullstack Engineer", raw: VALID_MARKDOWN },
      { role: "QA & Testing Engineer", raw: "" }, // will fail
      { role: "DevOps Engineer", raw: VALID_MARKDOWN },
    ];
    const { docs, errors } = parseAgentResponses(responses, makeContext());
    expect(docs).toHaveLength(2);
    expect(errors).toHaveLength(1);
    expect(errors[0]!.role).toBe("QA & Testing Engineer");
    expect(errors[0]!.error).toMatch(/empty response/);
  });

  it("returns empty arrays for empty input", () => {
    const { docs, errors } = parseAgentResponses([], makeContext());
    expect(docs).toHaveLength(0);
    expect(errors).toHaveLength(0);
  });

  it("each doc has the correct filename for its role", () => {
    const responses = [{ role: "Fullstack Engineer", raw: VALID_MARKDOWN }];
    const { docs } = parseAgentResponses(responses, makeContext());
    expect(docs[0]!.filename).toBe("fullstack-engineer.md");
  });
});
