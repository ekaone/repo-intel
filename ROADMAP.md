# repo-intel Roadmap

## v0.1.x — CLI MVP
> Status: Shipped ✅

- Rust core: metadata-scanner, stack-detector, context-builder
- TypeScript: CLI, AI layer (Anthropic/OpenAI/Ollama), fallback static gen
- stdout bridge between Rust and JS (TypeScript spawns Rust binary via `spawnSync`)
- Platform binaries: Linux x64, macOS ARM64/x64, Windows x64
- Cross-platform CI/CD via GitHub Actions (4 parallel platform builds)
- Single package install — platform binary resolved automatically at runtime
- `npx` support — no manual binary install required
- `optionalDependencies` rewritten to exact versions before publish (fixes `workspace:*` issue)
- Published to npm as `@ekaone/repo-intel`

**Known limitations (deferred to v0.2.x):**
- Scanner is signal-based only — does not read actual source files (`.ts`, `.rs`, etc.)
- Agent docs overwrite silently on re-generation (no `--force` guard yet)
- Sequential LLM calls (~35s per agent role) — no parallel option yet

---

## v0.2.0 — Speed + Incremental Intelligence
> Status: Planned

**Known issues from v0.1.x to fix first:**
- Language detection incorrect on monorepos — file pattern signals (e.g. `.rs` files)
  override `package.json` language. Example: `next.js` repo detected as `Rust` instead
  of `TypeScript`. Fix: weight `package.json` language detection higher than file pattern inference.
- Project name returns `"unknown"` when root directory has no `package.json` (e.g. monorepos).
  Fix: search one level deep for a `package.json` as fallback.
- Hidden directory detection broken — `.github/`, `.git/`, `.packages/` are skipped during
  walk, causing `has_ci`, `has_git`, and `has_monorepo` to return false incorrectly.
  Fix: explicitly check named hidden dirs for signal detection separately from the walk filter.
- Folder list polluted by test/docs/fixture directories — scanner walks into `test/`,
  `examples/`, `docs/`, `bench/`, `turbopack/` etc. collecting thousands of irrelevant nested
  folder names (numbered fixtures, Next.js route interception patterns like `(.)page`, `01-components`).
  Fix: expand `SKIP_DIRS` to exclude non-application directories.
- URL-encoded folder names — folder names with underscores are stored as `%5F` instead of `_`.
  Fix: store raw folder names in `walker.rs`.
- Rust unit tests deleted — 65 tests across scanner, detector, and context builder
  must be restored before v0.2.0 development begins.

**New features:**
- Incremental cache system (< 5ms on cache hit)
- Rayon parallel file analysis (4–8x faster on large repos)
- Git diff detection — skip re-generating unchanged agent roles
- Watch mode (`repo-intel watch`) — auto-regenerate on file changes
- YAML output format (targeting agentskills.io open standard)
- `repo-intel stale` command — detect outdated agent docs
- Custom prompt templates via `.repo-intel.toml`
- `--force` flag — explicit overwrite guard on re-generation
- `--parallel` flag — parallel LLM calls for users who want speed over rate-limit safety
- Deep source file scanning — sample actual `.ts`/`.rs`/`.py` files for richer context
  - `tsconfig.json` path aliases
  - `package.json` script patterns
  - Export signatures from entry files
  - Git history signals (recently active files)
- Agent Skills output mode — add `--format skills` flag to generate spec-compliant `SKILL.md`
  files (YAML frontmatter + instructions) compatible with Claude Code, Cursor, GitHub Copilot,
  and other agentskills.io-compatible tools
- Dual output modes — `--format persona` (current default, rich identity/personality docs) and
  `--format skills` (agentskills.io standard, task-focused capability packages)
- Prompt Builder customization — allow users to override section templates via `.repo-intel.toml`
  for custom agent personas, tone, and output structure

---

## v0.2.x — SDK
> Status: Planned

- New package: `repo-intel-sdk`
- Programmatic API — composable individual pipeline steps
- Same platform binaries as CLI (no new Rust code needed)
- Low effort — re-exports existing pipeline with clean public API
- `repo-intel-sdk` targets IDE extensions, CI pipelines, and custom tooling

---

## v0.3.0 — Native Bridge
> Status: Future

- New package: `repo-intel-native`
- napi-rs bridge replaces stdout (direct in-process Rust calls, zero spawn overhead)
- Same API as `repo-intel-sdk` — drop-in replacement
- Single package install with bundled binaries (no `optionalDependencies` needed)
- Targets: watch mode in IDE extensions, maximum performance use cases
- Distribution: `.node` files per Node.js ABI × platform
- Resolves `npx` and global install complexity permanently

---

## Package Evolution

| Version | Packages | Install |
|---|---|---|
| v0.1.x | `@ekaone/repo-intel` | `pnpm add -g @ekaone/repo-intel` |
| v0.2.x | + `@ekaone/repo-intel-sdk` | `pnpm add @ekaone/repo-intel-sdk` |
| v0.3.0 | + `@ekaone/repo-intel-native` | `pnpm add @ekaone/repo-intel-native` |

---

## Architecture Notes

**Current stdout bridge (v0.1.x):**
```
CLI (TypeScript)
  └── spawnSync(rust-binary, ["scan", "--root", path, "--json"])
        └── stdout → JSON.parse → RepoContext
              └── prompt-builder → AI provider → agent docs
```

**Future napi-rs bridge (v0.3.0):**
```
CLI (TypeScript)
  └── require('@ekaone/repo-intel-native')
        └── napi::call → Rust fn → RepoContext (in-process, zero IPC)
              └── prompt-builder → AI provider → agent docs
```

The stdout bridge was chosen for v0.1.x due to lower complexity — LLM call time
(~35s per role) dominates total pipeline time, making spawn overhead negligible.
napi-rs becomes worthwhile in v0.3.0 for watch mode and IDE extension use cases
where sub-millisecond scan latency matters.