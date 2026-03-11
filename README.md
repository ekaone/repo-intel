# repo-intel

> **Understand any codebase instantly.** `repo-intel` scans your repository, detects your tech stack, and generates ready-to-use AI agent context files — powered by a blazing-fast Rust core.

[![CI](https://github.com/your-org/repo-intel/actions/workflows/ci.yml/badge.svg)](https://github.com/your-org/repo-intel/actions/workflows/ci.yml)
[![npm](https://img.shields.io/npm/v/repo-intel)](https://www.npmjs.com/package/repo-intel)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)

## Features

- 🦀 **Rust core** — scans 10k+ files in milliseconds
- 🔍 **Multi-layer detection** — dependencies, folder structure, and file patterns
- 🤖 **AI-powered** — generates agent docs via Anthropic, OpenAI, or local Ollama
- 📄 **No-AI fallback** — deterministic static generation when no LLM is configured
- 🌍 **Cross-platform** — pre-built binaries for Linux, macOS (arm64/x64), and Windows

## Quick Start

```bash
npx repo-intel generate
```

Or install globally:

```bash
npm install -g repo-intel
repo-intel generate
```

## Commands

| Command | Description |
|---------|-------------|
| `repo-intel scan` | Scan repo and print `context.json` to stdout |
| `repo-intel generate` | Run full pipeline: scan → AI → write agent docs |
| `repo-intel version` | Print version info |

## Configuration

Create a `.repo-intel.toml` in your project root:

```toml
[scan]
skip_dirs = ["node_modules", "target", ".git", "dist"]

[ai]
provider = "anthropic"   # anthropic | openai | ollama
model = "claude-sonnet-4-20250514"

[output]
dir = "agents"
```

## Documentation

- [Architecture](docs/architecture.md)
- [Contributing](docs/contributing.md)
- [AI Providers](docs/providers.md)
- [Fallback Templates](docs/templates.md)

## License

MIT
