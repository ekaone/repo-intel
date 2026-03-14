# repo-intel

> **Understand any codebase instantly.** Scan your repository and generate AI agent context files — powered by a blazing-fast Rust core.

## Installation

```bash
npm install @ekaone/repo-intel
# or
pnpm add @ekaone/repo-intel
```

## Programmatic API

```typescript
import { analyze, scan, generate } from "@ekaone/repo-intel";

// Full pipeline: scan + AI generate
const result = await analyze({ root: process.cwd() });

// Just scan (returns context.json as object)
const context = await scan({ root: process.cwd() });

// Generate agent docs from an existing context
await generate({ context, outputDir: "agents" });
```

## CLI

```bash
# Generate agent docs
npx @ekaone/repo-intel generate

# Just scan and print context.json
npx @ekaone/repo-intel scan

# Use a specific AI provider
npx @ekaone/repo-intel generate --provider anthropic
```

## Configuration

Create `.repo-intel.toml` in your project root:

```toml
[ai]
provider = "anthropic"
model = "claude-sonnet-4-20250514"

[output]
dir = "agents"
```

See [docs/providers.md](../../docs/providers.md) for provider setup.
