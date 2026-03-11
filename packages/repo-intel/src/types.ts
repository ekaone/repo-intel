// ── Rust core types (mirroring context.json schema) ─────────────────────────

export interface Skill {
  name: string;
  category: string;
  confidence: number;
  signals: string[];
}

export interface StackResult {
  languages: Skill[];
  frameworks: Skill[];
  tooling: Skill[];
  architecture: string[];
}

export interface AgentRole {
  id: string;
  title: string;
  description: string;
}

export interface RepoContext {
  name: string;
  stack: StackResult;
  agent_roles: AgentRole[];
  readme_excerpt: string | null;
  has_git: boolean;
  has_docker: boolean;
  has_ci: boolean;
  schema_version: string;
}

// ── AI output types ──────────────────────────────────────────────────────────

export interface AgentDoc {
  role_id: string;
  title: string;
  content: string;
}

// ── Option types for the public API ─────────────────────────────────────────

export interface ScanOptions {
  root?: string;
  config?: string;
}

export interface GenerateOptions {
  context: RepoContext;
  outputDir?: string;
  provider?: "anthropic" | "openai" | "ollama";
  model?: string;
  noAi?: boolean;
}

export interface AnalyzeOptions extends ScanOptions {
  outputDir?: string;
  provider?: "anthropic" | "openai" | "ollama";
  model?: string;
  noAi?: boolean;
}

export type AiProvider = "anthropic" | "openai" | "ollama";
