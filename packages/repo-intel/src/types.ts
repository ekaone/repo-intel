// ── context.json contract (mirrors Rust types exactly) ───────────────────────
// Field names are snake_case to match Rust serde output.
// BOTH sides must agree on this shape — do not rename fields without updating
// the Rust serializer too.

export type SkillSource =
  | { type: "package_json" }
  | { type: "cargo_toml" }
  | { type: "folder_name"; value: string }
  | { type: "file_pattern"; value: string }
  | { type: "readme_signal" };

export interface Skill {
  name: string;
  confidence: number; // 0.0 – 1.0
  source: SkillSource;
}

export type ArchStyle = "feature_based" | "layer_based" | "flat";

export interface StackResult {
  language: string;
  framework: string | null;
  styling: string | null;
  state_management: string | null;
  testing: string | null;
  database: string | null;
  runtime: string | null;
  skills: Skill[];
  architecture_style: ArchStyle | null;
}

export interface ProjectMeta {
  name: string;
  description: string | null;
  readme_excerpt: string | null;
}

export interface ArchMeta {
  style: ArchStyle | null;
  folders: string[];
  has_monorepo: boolean;
  has_docker: boolean;
  has_ci: boolean;
  has_git: boolean;
}

/** The full context.json contract between Rust and JS. */
export interface RepoContext {
  version: string;
  scanned_at: string; // ISO 8601
  root: string;
  project: ProjectMeta;
  stack: StackResult;
  architecture: ArchMeta;
  agent_roles: string[];
}

// ── AI layer types ────────────────────────────────────────────────────────────

export type AIProviderName = "anthropic" | "openai" | "ollama";

/** Runtime configuration passed to every AI provider call. */
export interface AIConfig {
  provider: AIProviderName;
  /** Model name — falls back to provider default if omitted. */
  model?: string;
  /** API key resolved from env var. Never hardcoded. */
  apiKey?: string;
  /** Base URL override — primarily for Ollama / self-hosted endpoints. */
  baseUrl?: string;
  /** Max tokens for LLM response. Default: 2048. */
  maxTokens?: number;
}

/**
 * Contract every LLM provider must implement.
 * Providers are pure functions: prompt + config → raw markdown string.
 */
export interface AIProvider {
  call(prompt: string, config: AIConfig): Promise<string>;
}

/** Default models per provider. */
export const DEFAULT_MODELS: Record<AIProviderName, string> = {
  anthropic: "claude-sonnet-4-20250514",
  openai: "gpt-4o",
  ollama: "llama3.2",
};

/** Default API key env var names per provider. */
export const API_KEY_ENV_VARS: Record<AIProviderName, string> = {
  anthropic: "ANTHROPIC_API_KEY",
  openai: "OPENAI_API_KEY",
  ollama: "", // Ollama is local — no key required
};

// ── Agent doc types ───────────────────────────────────────────────────────────

/** A fully generated agent documentation file. */
export interface AgentDoc {
  /** Human-readable role name, e.g. "Fullstack Engineer" */
  role: string;
  /** Filename to write, e.g. "fullstack-engineer.md" */
  filename: string;
  /** Full Markdown content of the agent doc */
  content: string;
  /** ISO 8601 timestamp of when this doc was generated */
  generatedAt: string;
  /** Framework or language used to identify the project context */
  generatedBy: string;
  /** Average skill confidence from the detected stack (0.0–1.0) */
  confidence: number;
}

// ── CLI option types ──────────────────────────────────────────────────────────

/** Parsed CLI options passed through the pipeline. */
export interface GenerateOptions {
  /** Absolute path to the repo root to scan. Default: cwd */
  root: string;
  /** Skip AI layer — use static fallback templates. */
  noAi: boolean;
  /** Print generated docs to stdout without writing files. */
  dryRun: boolean;
  /** Emit verbose debug info to stderr. */
  debug: boolean;
  /** Override the AI provider from config. */
  provider?: AIProviderName;
  /** Override the output directory from config. */
  outputDir?: string;
}

// ── Pipeline result types ─────────────────────────────────────────────────────

export interface PipelineResult {
  context: RepoContext;
  docs: AgentDoc[];
  /** True if AI was used, false if static fallback was used. */
  usedAi: boolean;
  /** Wall-clock ms for the full pipeline run. */
  durationMs: number;
}

// ── Config file types (mirrors .repo-intel.toml) ──────────────────────────────

export interface RepoIntelConfig {
  ai: {
    provider: AIProviderName;
    model?: string;
    api_key_env: string;
    base_url?: string;
  };
  output: {
    dir: string;
    format: "markdown";
  };
  project: {
    exclude: string[];
  };
  stack: {
    override_skills: string[];
  };
}
