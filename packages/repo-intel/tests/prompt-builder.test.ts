import { describe, it, expect } from "vitest";
import { buildPrompt } from "../src/ai/prompt-builder";
import type { RepoContext } from "../src/types";

// ── Shared fixture ────────────────────────────────────────────────────────────

function makeContext(overrides: Partial<RepoContext> = {}): RepoContext {
  return {
    version: "0.1.0",
    scanned_at: "2026-03-12T00:00:00Z",
    root: "/projects/my-app",
    project: {
      name: "my-app",
      description: "A dashboard for monitoring IoT devices",
      readme_excerpt:
        "my-app is a real-time IoT dashboard built with Next.js and Prisma.",
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
          name: "Tailwind CSS",
          confidence: 0.95,
          source: { type: "package_json" },
        },
        {
          name: "Testing",
          confidence: 0.8,
          source: { type: "file_pattern", value: "*.test.ts" },
        },
        {
          name: "Storybook",
          confidence: 0.6,
          source: { type: "file_pattern", value: "*.stories.tsx" },
        },
      ],
    },
    architecture: {
      style: "layer_based",
      folders: ["components", "hooks", "services", "pages", "lib"],
      has_monorepo: false,
      has_docker: true,
      has_ci: true,
      has_git: true,
    },
    agent_roles: [
      "Fullstack Engineer",
      "QA & Testing Engineer",
      "Database Engineer",
    ],
    ...overrides,
  };
}

// ── Return type ───────────────────────────────────────────────────────────────

describe("buildPrompt() — return type", () => {
  it("returns a non-empty string", () => {
    const prompt = buildPrompt(makeContext(), "Fullstack Engineer");
    expect(typeof prompt).toBe("string");
    expect(prompt.length).toBeGreaterThan(0);
  });

  it("returns a prompt of at least 500 characters", () => {
    const prompt = buildPrompt(makeContext(), "Fullstack Engineer");
    expect(prompt.length).toBeGreaterThanOrEqual(500);
  });
});

// ── Project context ───────────────────────────────────────────────────────────

describe("buildPrompt() — project context", () => {
  it("includes the project name", () => {
    const prompt = buildPrompt(makeContext(), "Fullstack Engineer");
    expect(prompt).toContain("my-app");
  });

  it("includes the project description", () => {
    const prompt = buildPrompt(makeContext(), "Fullstack Engineer");
    expect(prompt).toContain("IoT devices");
  });

  it("includes the README excerpt", () => {
    const prompt = buildPrompt(makeContext(), "Fullstack Engineer");
    expect(prompt).toContain("real-time IoT dashboard");
  });

  it("handles missing description gracefully", () => {
    const ctx = makeContext();
    ctx.project.description = null;
    const prompt = buildPrompt(ctx, "Fullstack Engineer");
    expect(prompt).toContain("Not specified");
  });

  it("handles missing README excerpt gracefully", () => {
    const ctx = makeContext();
    ctx.project.readme_excerpt = null;
    const prompt = buildPrompt(ctx, "Fullstack Engineer");
    expect(prompt).toContain("Not available");
  });
});

// ── Stack fields ──────────────────────────────────────────────────────────────

describe("buildPrompt() — stack fields", () => {
  it("includes the detected language", () => {
    const prompt = buildPrompt(makeContext(), "Fullstack Engineer");
    expect(prompt).toContain("TypeScript");
  });

  it("includes the detected framework", () => {
    const prompt = buildPrompt(makeContext(), "Fullstack Engineer");
    expect(prompt).toContain("Next.js");
  });

  it("includes the detected styling", () => {
    const prompt = buildPrompt(makeContext(), "Fullstack Engineer");
    expect(prompt).toContain("Tailwind CSS");
  });

  it("includes the detected database", () => {
    const prompt = buildPrompt(makeContext(), "Fullstack Engineer");
    expect(prompt).toContain("Prisma");
  });

  it("includes the detected testing framework", () => {
    const prompt = buildPrompt(makeContext(), "Fullstack Engineer");
    expect(prompt).toContain("Vitest");
  });

  it("includes the detected runtime", () => {
    const prompt = buildPrompt(makeContext(), "Fullstack Engineer");
    expect(prompt).toContain("Node.js");
  });

  it('shows "None detected" for null stack fields', () => {
    const ctx = makeContext();
    ctx.stack.framework = null;
    ctx.stack.database = null;
    const prompt = buildPrompt(ctx, "Fullstack Engineer");
    expect(prompt).toContain("None detected");
  });
});

// ── Architecture ──────────────────────────────────────────────────────────────

describe("buildPrompt() — architecture", () => {
  it("includes detected folder names", () => {
    const prompt = buildPrompt(makeContext(), "Fullstack Engineer");
    expect(prompt).toContain("components");
    expect(prompt).toContain("hooks");
    expect(prompt).toContain("services");
  });

  it("includes Docker flag when present", () => {
    const prompt = buildPrompt(makeContext(), "Fullstack Engineer");
    expect(prompt).toContain("Docker");
  });

  it("includes CI flag when present", () => {
    const prompt = buildPrompt(makeContext(), "Fullstack Engineer");
    expect(prompt).toContain("CI");
  });

  it("reflects has_docker: false correctly", () => {
    const ctx = makeContext();
    ctx.architecture.has_docker = false;
    ctx.architecture.has_ci = false;
    const prompt = buildPrompt(ctx, "Fullstack Engineer");
    expect(prompt).not.toContain("Docker");
  });
});

// ── Role injection ────────────────────────────────────────────────────────────

describe("buildPrompt() — role injection", () => {
  it("includes the target role name in the prompt", () => {
    const prompt = buildPrompt(makeContext(), "Fullstack Engineer");
    expect(prompt).toContain("Fullstack Engineer");
  });

  it("uses the correct role name for different roles", () => {
    const prompt = buildPrompt(makeContext(), "QA & Testing Engineer");
    expect(prompt).toContain("QA & Testing Engineer");
  });

  it("injects the Fullstack Engineer role hint", () => {
    const prompt = buildPrompt(makeContext(), "Fullstack Engineer");
    expect(prompt.toLowerCase()).toContain("request lifecycle");
  });

  it("injects the DevOps Engineer role hint", () => {
    const prompt = buildPrompt(makeContext(), "DevOps Engineer");
    expect(prompt.toLowerCase()).toContain("ci/cd");
  });

  it("injects the Database Engineer role hint", () => {
    const prompt = buildPrompt(makeContext(), "Database Engineer");
    expect(prompt.toLowerCase()).toContain("schema");
  });
});

// ── Skills sections ───────────────────────────────────────────────────────────

describe("buildPrompt() — skills", () => {
  it("includes high-confidence skills (≥ 0.90) in the primary section", () => {
    const prompt = buildPrompt(makeContext(), "Fullstack Engineer");
    // React (0.99), Next.js (0.99), Prisma (0.97), Tailwind CSS (0.95) are primary
    expect(prompt).toContain("React");
    expect(prompt).toContain("Prisma");
    expect(prompt).toContain("Tailwind CSS");
  });

  it("includes medium-confidence skills (0.50–0.89) in the secondary section", () => {
    const prompt = buildPrompt(makeContext(), "Fullstack Engineer");
    // Testing (0.80) and Storybook (0.60) are secondary
    expect(prompt).toContain("Testing");
    expect(prompt).toContain("Storybook");
  });

  it("does not include internal __ markers in the prompt", () => {
    const ctx = makeContext();
    ctx.stack.skills.push({
      name: "__has_docker",
      confidence: 0.99,
      source: { type: "file_pattern", value: "Dockerfile" },
    });
    const prompt = buildPrompt(ctx, "Fullstack Engineer");
    expect(prompt).not.toContain("__has_docker");
  });

  it('shows "None detected" when no primary skills exist', () => {
    const ctx = makeContext();
    ctx.stack.skills = ctx.stack.skills.map((s) => ({ ...s, confidence: 0.6 }));
    const prompt = buildPrompt(ctx, "Fullstack Engineer");
    expect(prompt).toContain("None detected");
  });
});

// ── Team context ──────────────────────────────────────────────────────────────

describe("buildPrompt() — team context", () => {
  it("lists other agent roles in the team context section", () => {
    const prompt = buildPrompt(makeContext(), "Fullstack Engineer");
    // Other roles: QA & Testing Engineer, Database Engineer
    expect(prompt).toContain("QA & Testing Engineer");
    expect(prompt).toContain("Database Engineer");
  });

  it("does not list the current role as another agent", () => {
    const prompt = buildPrompt(makeContext(), "Fullstack Engineer");
    // Extract only the other-roles line — the role name legitimately appears
    // again later in ## Your Task, so we must not check the entire tail.
    const afterLabel = prompt.split("Other agents in this project:**")[1] ?? "";
    const otherRolesLine = afterLabel.split("\n")[0];
    expect(otherRolesLine).not.toContain("Fullstack Engineer");
    expect(otherRolesLine).toContain("QA & Testing Engineer");
    expect(otherRolesLine).toContain("Database Engineer");
  });

  it("notes no other agents when roles array has one entry", () => {
    const ctx = makeContext();
    ctx.agent_roles = ["Fullstack Engineer"];
    const prompt = buildPrompt(ctx, "Fullstack Engineer");
    expect(prompt).toContain("None");
  });
});

// ── Output instructions ───────────────────────────────────────────────────────

describe("buildPrompt() — output instructions", () => {
  it("instructs the LLM to output only Markdown", () => {
    const prompt = buildPrompt(makeContext(), "Fullstack Engineer");
    expect(prompt.toLowerCase()).toContain("markdown");
    expect(prompt.toLowerCase()).toContain("no preamble");
  });

  it("specifies the 10 required sections", () => {
    const prompt = buildPrompt(makeContext(), "Fullstack Engineer");
    // Check a sample of the required sections
    expect(prompt).toContain("Identity");
    expect(prompt).toContain("Personality");
    expect(prompt).toContain("Workflow");
    expect(prompt).toContain("Rules");
    expect(prompt).toContain("Metrics");
  });
});
