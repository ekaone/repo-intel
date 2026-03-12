import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import type { RepoContext } from "../src/types";

// ── Mock spawnSync + getBinaryPath before importing runner ────────────────────
// runner.ts calls getBinaryPath() at call time (not module load time) and
// spawnSync is called inside runRustPipeline. We mock both so no real binary
// or filesystem access is needed.

vi.mock("child_process", () => ({
  spawnSync: vi.fn(),
}));

vi.mock("../src/loader", () => ({
  getBinaryPath: vi.fn(() => "/mock/bin/repo-intel"),
}));

// Imports must come AFTER vi.mock() declarations
import { spawnSync } from "child_process";
import { getBinaryPath } from "../src/loader";
import { runRustPipeline } from "../src/pipeline/runner";

// ── Fixtures ──────────────────────────────────────────────────────────────────

const MOCK_CONTEXT: RepoContext = {
  version: "0.1.0",
  scanned_at: "2026-03-12T00:00:00Z",
  root: "/projects/my-app",
  project: {
    name: "my-app",
    description: "A dashboard for monitoring IoT devices",
    readme_excerpt: "my-app is a real-time IoT dashboard.",
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
    ],
  },
  architecture: {
    style: "layer_based",
    folders: ["components", "hooks", "services"],
    has_monorepo: false,
    has_docker: true,
    has_ci: true,
    has_git: true,
  },
  agent_roles: ["Fullstack Engineer", "QA & Testing Engineer"],
};

/** Build a mock spawnSync result that simulates a successful binary run. */
function mockSuccess(context: RepoContext = MOCK_CONTEXT) {
  vi.mocked(spawnSync).mockReturnValue({
    stdout: JSON.stringify(context),
    stderr: "",
    status: 0,
    error: undefined,
    pid: 1234,
    output: [],
    signal: null,
  } as any);
}

/** Build a mock spawnSync result that simulates a non-zero exit. */
function mockFailure(stderr = "error: root not found", status = 1) {
  vi.mocked(spawnSync).mockReturnValue({
    stdout: "",
    stderr,
    status,
    error: undefined,
    pid: 1234,
    output: [],
    signal: null,
  } as any);
}

/** Build a mock that simulates a spawn-level error (binary not found). */
function mockSpawnError(message = "ENOENT: no such file or directory") {
  vi.mocked(spawnSync).mockReturnValue({
    stdout: "",
    stderr: "",
    status: null,
    error: new Error(message),
    pid: 0,
    output: [],
    signal: null,
  } as any);
}

// ── Tests ─────────────────────────────────────────────────────────────────────

describe("runRustPipeline() — happy path", () => {
  beforeEach(() => mockSuccess());

  it("returns a parsed RepoContext", () => {
    const ctx = runRustPipeline("/projects/my-app");
    expect(ctx).toMatchObject({
      version: "0.1.0",
      project: { name: "my-app" },
      stack: { language: "TypeScript", framework: "Next.js" },
    });
  });

  it("calls spawnSync with the correct arguments", () => {
    runRustPipeline("/projects/my-app");
    expect(spawnSync).toHaveBeenCalledWith(
      "/mock/bin/repo-intel",
      ["scan", "--root", expect.stringContaining("my-app"), "--json"],
      expect.objectContaining({ encoding: "utf8" }),
    );
  });

  it("resolves root to an absolute path before passing to binary", () => {
    runRustPipeline("relative/path");
    const [, args] = vi.mocked(spawnSync).mock.calls[0]!;
    const rootArg = (args as string[])[2]!;
    expect(rootArg.startsWith("/")).toBe(true);
  });

  it("returns agent_roles array", () => {
    const ctx = runRustPipeline("/projects/my-app");
    expect(ctx.agent_roles).toContain("Fullstack Engineer");
    expect(ctx.agent_roles).toContain("QA & Testing Engineer");
  });

  it("returns architecture flags correctly", () => {
    const ctx = runRustPipeline("/projects/my-app");
    expect(ctx.architecture.has_docker).toBe(true);
    expect(ctx.architecture.has_ci).toBe(true);
    expect(ctx.architecture.has_git).toBe(true);
    expect(ctx.architecture.has_monorepo).toBe(false);
  });

  it("calls getBinaryPath() to resolve the binary", () => {
    runRustPipeline("/projects/my-app");
    expect(getBinaryPath).toHaveBeenCalled();
  });
});

// ── Error: spawn failure ──────────────────────────────────────────────────────

describe("runRustPipeline() — spawn error", () => {
  it("throws when the binary cannot be spawned (ENOENT)", () => {
    mockSpawnError("ENOENT: no such file or directory");
    expect(() => runRustPipeline("/projects/my-app")).toThrow(
      /failed to spawn/,
    );
  });

  it("error message includes the binary path", () => {
    mockSpawnError("ENOENT");
    expect(() => runRustPipeline("/projects/my-app")).toThrow(
      "/mock/bin/repo-intel",
    );
  });
});

// ── Error: non-zero exit ──────────────────────────────────────────────────────

describe("runRustPipeline() — non-zero exit", () => {
  it("throws when binary exits with non-zero status", () => {
    mockFailure("error: root directory not found", 1);
    expect(() => runRustPipeline("/projects/my-app")).toThrow(
      /exited with code 1/,
    );
  });

  it("includes stderr output in the error message", () => {
    mockFailure("error: root directory not found");
    expect(() => runRustPipeline("/projects/my-app")).toThrow(
      "root directory not found",
    );
  });
});

// ── Error: empty stdout ───────────────────────────────────────────────────────

describe("runRustPipeline() — empty stdout", () => {
  it("throws when stdout is empty despite status 0", () => {
    vi.mocked(spawnSync).mockReturnValue({
      stdout: "",
      stderr: "",
      status: 0,
      error: undefined,
      pid: 1234,
      output: [],
      signal: null,
    } as any);

    expect(() => runRustPipeline("/projects/my-app")).toThrow(/no output/);
  });

  it("throws when stdout is only whitespace", () => {
    vi.mocked(spawnSync).mockReturnValue({
      stdout: "   \n  ",
      stderr: "",
      status: 0,
      error: undefined,
      pid: 1234,
      output: [],
      signal: null,
    } as any);

    expect(() => runRustPipeline("/projects/my-app")).toThrow(/no output/);
  });
});

// ── Error: invalid JSON ───────────────────────────────────────────────────────

describe("runRustPipeline() — invalid JSON", () => {
  it("throws when stdout is not valid JSON", () => {
    vi.mocked(spawnSync).mockReturnValue({
      stdout: "this is not json at all",
      stderr: "",
      status: 0,
      error: undefined,
      pid: 1234,
      output: [],
      signal: null,
    } as any);

    expect(() => runRustPipeline("/projects/my-app")).toThrow(/not valid JSON/);
  });

  it("includes a preview of the bad output in the error", () => {
    vi.mocked(spawnSync).mockReturnValue({
      stdout: "not-json-content",
      stderr: "",
      status: 0,
      error: undefined,
      pid: 1234,
      output: [],
      signal: null,
    } as any);

    expect(() => runRustPipeline("/projects/my-app")).toThrow(
      "not-json-content",
    );
  });
});

// ── Error: shape validation ───────────────────────────────────────────────────

describe("runRustPipeline() — context shape validation", () => {
  it("throws when a required field is missing from context.json", () => {
    const broken = { version: "0.1.0", scanned_at: "2026-01-01T00:00:00Z" };
    // Missing: root, project, stack, architecture, agent_roles

    vi.mocked(spawnSync).mockReturnValue({
      stdout: JSON.stringify(broken),
      stderr: "",
      status: 0,
      error: undefined,
      pid: 1234,
      output: [],
      signal: null,
    } as any);

    expect(() => runRustPipeline("/projects/my-app")).toThrow(
      /missing required field/,
    );
  });

  it("error message names the missing field", () => {
    const missing_stack = { ...MOCK_CONTEXT };
    delete (missing_stack as any).stack;

    vi.mocked(spawnSync).mockReturnValue({
      stdout: JSON.stringify(missing_stack),
      stderr: "",
      status: 0,
      error: undefined,
      pid: 1234,
      output: [],
      signal: null,
    } as any);

    expect(() => runRustPipeline("/projects/my-app")).toThrow("'stack'");
  });

  it("mentions version mismatch in validation errors", () => {
    const broken = { version: "0.1.0" };
    vi.mocked(spawnSync).mockReturnValue({
      stdout: JSON.stringify(broken),
      stderr: "",
      status: 0,
      error: undefined,
      pid: 1234,
      output: [],
      signal: null,
    } as any);

    expect(() => runRustPipeline("/projects/my-app")).toThrow(
      /version mismatch/,
    );
  });
});
