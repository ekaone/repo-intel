# Architecture

## Overview

repo-intel is a **hybrid Rust+TypeScript tool** that scans any repository, detects its tech stack, and generates AI-ready agent documentation.

## Pipeline Diagram

```
repo root
   │
   ▼
┌─────────────┐    context.json    ┌──────────────────┐
│  Rust Core  │ ─────────────────▶ │  JS/TS Pipeline  │
│  (scanner   │   stdout/pipe      │  (AI generate    │
│   detector  │                    │   + file writer) │
│   context)  │                    └──────────────────┘
└─────────────┘                            │
                                           ▼
                                    agents/*.md
                                    AGENTS.md
```

## Module Responsibilities

### Rust Crate (`crates/repo-intel-core`)

| Module | Responsibility |
|--------|---------------|
| `scanner` | Walk the directory tree, build `folder_map`, detect signal files |
| `detector` | Three-layer analysis: deps → folders → filename patterns |
| `context` | Aggregate to `RepoContext`, enrich with git/docker/CI, serialise |

#### Detection Layers

1. **Layer 1 — Dependencies** (`detector/deps.rs`): Parses `package.json`, `Cargo.toml`, etc. to identify languages and frameworks with confidence scores.
2. **Layer 2 — Folders** (`detector/folders.rs`): Infers architecture patterns from directory names (e.g., `components/`, `controllers/`, `packages/`).
3. **Layer 3 — Patterns** (`detector/patterns.rs`): Detects tool config files (Dockerfile, `biome.json`, `vitest.config.*`, etc.).

### TypeScript Package (`packages/repo-intel`)

| Module | Responsibility |
|--------|---------------|
| `pipeline/runner.ts` | Spawn Rust binary, capture `context.json` from stdout |
| `ai/prompt-builder.ts` | Convert `RepoContext` → LLM prompt |
| `ai/caller.ts` | Provider-agnostic call with retries + timeout |
| `ai/providers/` | Anthropic, OpenAI, Ollama implementations |
| `pipeline/writer.ts` | Write `AgentDoc[]` → `agents/*.md` + `AGENTS.md` |
| `fallback/` | Static generation without LLM |
| `loader.ts` | Resolve correct platform binary at runtime |

## Context JSON Schema (v1)

```json
{
  "name": "string",
  "stack": {
    "languages": [{ "name": "string", "category": "language", "confidence": 0.0, "signals": [] }],
    "frameworks": [...],
    "tooling": [...],
    "architecture": ["string"]
  },
  "agent_roles": [{ "id": "string", "title": "string", "description": "string" }],
  "readme_excerpt": "string | null",
  "has_git": true,
  "has_docker": false,
  "has_ci": false,
  "schema_version": "1"
}
```
