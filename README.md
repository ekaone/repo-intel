# repo-intel

> **🚧 Under Active Development** - Not yet released. Work in progress.

> Scan your repository. Generate AI agent documentation. Instantly.

**repo-intel** is a blazing-fast CLI tool powered by a Rust core that scans your codebase,
detects your tech stack, and generates rich AI agent persona docs — so tools like
Claude Code, Cursor, and GitHub Copilot actually understand your project.

> **Current Status**: Development phase. Core Rust scanner is functional, TypeScript layer in progress.
> **Planned Release**: v0.1.0 targeting early 2026.
> **Contributions**: Welcome! See [contributing guide](docs/contributing.md).

```bash
npx repo-intel generate
```

```
✓ Scanned 6,241 files in 147ms
✓ Detected: Next.js · TypeScript · Tailwind CSS · Prisma · Vitest
✓ Generated 4 agent docs → ./agents/
  agents/frontend-engineer.md
  agents/fullstack-engineer.md
  agents/database-engineer.md
  agents/testing-engineer.md
✓ Written: AGENTS.md
```

> **Note**: The above output represents the target functionality. Currently implementing the core scanning and detection logic.

---

## Why repo-intel?

AI coding agents are only as good as the context they have. Without project-specific
knowledge, they give generic answers — correct for the framework, wrong for your codebase.

**Without repo-intel**, your agent knows:
- How React works in general
- How Prisma works in general

**With repo-intel**, your agent knows:
- That *your* project uses a feature-based architecture under `src/modules/`
- That *your* hooks live in `src/hooks/` and follow a specific pattern
- That *your* team targets LCP < 2.5s as a hard performance constraint
- That *your* project is an IoT monitoring dashboard (from README context)

---

## How it works

```
Your repo
   ↓
Rust core (fast)
  ├── metadata-scanner   walks directory tree, finds signal files
  ├── stack-detector     3-layer inference: deps → folders → file patterns
  └── context-builder    produces structured context.json

   ↓ stdout (context.json)

TypeScript layer
  ├── prompt-builder     context → rich LLM prompt
  ├── AI provider        Anthropic · OpenAI · Ollama
  └── file writer        agents/*.md + AGENTS.md
```

The Rust binary handles all filesystem work — scanning 10,000 files in under 200ms.
The TypeScript layer handles the LLM call and file output.

---

## Requirements

- **Node.js** 18 or later
- An **API key** for your chosen AI provider:
  - [Anthropic](https://console.anthropic.com/) — `ANTHROPIC_API_KEY`
  - [OpenAI](https://platform.openai.com/) — `OPENAI_API_KEY`
  - [Ollama](https://ollama.com/) — free, runs locally, no key needed

---

## Installation

```bash
# npm
npm install -g repo-intel

# pnpm
pnpm add -g repo-intel

# or run without installing
npx repo-intel generate
```

---

## Quick Start

**1. Set your API key**

```bash
# Anthropic (default)
export ANTHROPIC_API_KEY=your-key-here

# or OpenAI
export OPENAI_API_KEY=your-key-here
```

**2. Run in your project**

```bash
cd your-project
npx repo-intel generate
```

**3. Done** — check `./agents/` and `AGENTS.md`

---

## Commands

### `repo-intel generate`

Full pipeline — scan repo, detect stack, generate agent docs via LLM.

```bash
repo-intel generate
repo-intel generate --root ./my-project        # custom root
repo-intel generate --output ./docs/agents     # custom output dir
repo-intel generate --provider openai          # use OpenAI instead
repo-intel generate --provider ollama          # use local Ollama
repo-intel generate --no-ai                    # skip LLM, static output
repo-intel generate --dry-run                  # preview, no files written
```

### `repo-intel scan`

Scan only — outputs `context.json` to stdout. No LLM call. Useful for
debugging what the tool detected, or piping to your own tooling.

```bash
repo-intel scan
repo-intel scan --pretty                       # human-readable JSON
repo-intel scan --root ./my-project
```

### `repo-intel version`

```bash
repo-intel version
# repo-intel 0.1.0
```

---

## AI Providers

### Anthropic (default)

```bash
export ANTHROPIC_API_KEY=your-key
repo-intel generate
# uses claude-sonnet-4-20250514 by default
```

### OpenAI

```bash
export OPENAI_API_KEY=your-key
repo-intel generate --provider openai
# uses gpt-4o by default
```

### Ollama (free, local, offline)

```bash
# Install Ollama: https://ollama.com
ollama pull llama3.2
repo-intel generate --provider ollama
# no API key needed
```

---

## Configuration

Create `.repo-intel.toml` in your project root to customise behaviour.
The file is optional — repo-intel works with zero configuration.

```toml
[ai]
provider = "anthropic"
model = "claude-sonnet-4-20250514"
api_key_env = "ANTHROPIC_API_KEY"   # env var name, never the key itself

[output]
dir = "./agents"
format = "markdown"

[project]
exclude = ["legacy/", "vendor/", "generated/"]

[stack]
# Force-include skills the scanner might miss
override = []
```

---

## Output

### `agents/` directory

One Markdown file per detected agent role:

```
agents/
  frontend-engineer.md
  fullstack-engineer.md
  database-engineer.md
  testing-engineer.md
  devops-engineer.md
```

Each file contains a rich, project-aware agent persona:

```markdown
# Frontend Engineer Agent

## Identity
Expert React + TypeScript UI engineer, specialising in
Next.js server components and performance optimisation.

## Stack
- Next.js 14 (App Router)
- TypeScript 5
- Tailwind CSS 3
- Zustand (client state)
- TanStack Query (server state)

## Core Responsibilities
- Build reusable components following the design system in src/components/ui/
- Maintain hooks architecture in src/hooks/
- Own Core Web Vitals: LCP < 2.5s, FID < 100ms, CLS < 0.1

## Workflow
1. Check existing components before creating new ones
2. Follow feature-based structure: src/modules/{feature}/components/
3. Write Vitest tests alongside every component
...
```

### `AGENTS.md`

A summary index at your repo root listing all generated agents — the
"README for your AI agents."

---

## Stack Detection

repo-intel uses a three-layer confidence system to detect your stack:

| Layer | Method | Example |
|---|---|---|
| 1 | Dependency fingerprinting | `react` in `package.json` → React (0.99) |
| 2 | Folder architecture | `src/hooks/` exists → React hooks pattern (0.85) |
| 3 | File pattern matching | `*.test.ts` files → Testing framework (0.80) |

Skills with confidence ≥ 0.90 are marked as primary and directly influence
agent responsibilities. Skills below 0.50 are ignored.

**Detected automatically:**

- Frameworks: React, Next.js, Nuxt, Remix, Vue, Svelte, Angular
- Languages: TypeScript, JavaScript, Rust, Python, Go
- Styling: Tailwind CSS, CSS Modules, Styled Components
- State: Zustand, Jotai, Redux Toolkit, TanStack Query
- Testing: Vitest, Jest, Playwright, Cypress
- Database: Prisma, Drizzle, TypeORM, SQLx, Diesel
- Runtime: Node.js, Deno, Bun, Axum, Fastify, Express
- Infra: Docker, GitHub Actions, CircleCI

---

## `--no-ai` mode

If you don't have an API key or want a quick static output:

```bash
repo-intel generate --no-ai
```

Produces a simpler but accurate agent doc based on detected stack only —
no LLM call, instant output, always free.

---

## Programmatic API

```typescript
import { analyze, generate } from 'repo-intel'

// Full pipeline — scan + LLM generate
const docs = await generate({
  root: '.',
  output: './agents',
  provider: 'anthropic',
})

docs.forEach(doc => {
  console.log(doc.role)     // "Frontend Engineer"
  console.log(doc.filename) // "frontend-engineer.md"
  console.log(doc.content)  // full markdown content
})

// Scan only — no LLM
const context = await analyze({ root: '.' })
console.log(context.stack)   // { framework: 'Next.js', ... }
console.log(context.skills)  // [{ name: 'React', confidence: 0.99 }, ...]
```

---

## Performance

| Scenario | Time |
|---|---|
| Scan 1,000 files | ~30ms |
| Scan 6,000 files | ~150ms |
| Scan 20,000 files | ~400ms |
| LLM generation (3 agents) | ~5–15s |
| `--no-ai` mode | ~200ms total |

The Rust core is the fast part. The LLM call dominates total time —
that's expected and unavoidable when generating rich, project-aware content.

---

## Platform Support

repo-intel ships pre-compiled Rust binaries for all major platforms.
No Rust toolchain required.

| Platform | Package |
|---|---|
| macOS (Apple Silicon) | `repo-intel-darwin-arm64` |
| macOS (Intel) | `repo-intel-darwin-x64` |
| Linux x64 | `repo-intel-linux-x64` |
| Windows x64 | `repo-intel-win32-x64` |

Binaries are automatically selected by the npm installer based on your OS and CPU.

---

## Roadmap

| Version | Focus |
|---|---|
| **v0.1.0** | CLI MVP — scan, detect, LLM generate, npm publish |
| **v0.2.0** | Incremental cache, rayon parallel, watch mode, YAML output |
| **v0.2.x** | `repo-intel-sdk` — programmatic SDK package |
| **v0.3.0** | `repo-intel-native` — napi-rs bridge for maximum performance |

See [docs/ROADMAP.md](./docs/ROADMAP.md) for the full plan.

---

## Contributing

See [docs/contributing.md](./docs/contributing.md) for dev setup,
running tests, and the contribution workflow.

**Quick start for contributors:**

```bash
git clone https://github.com/ekaone/repo-intel
cd repo-intel
pnpm install
cargo build
```

**Run tests:**

```bash
cargo test          # Rust tests
pnpm test           # JS tests
```

---

## License

MIT © Eka Prasetia — see [LICENSE](./LICENSE) for details.

## Links

- [NPM Package](https://www.npmjs.com/package/repo-intel)
- [GitHub Repository](https://github.com/ekaone/repo-intel)
- [Issue Tracker](https://github.com/ekaone/repo-intel/issues)

---

<div align="center">
  <sub>Built with Rust 🦀 + TypeScript · Inspired by the agent skills ecosystem</sub>
</div>