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
> Status: In Progress 🔧

**Bug fixes shipped in v0.1.10 ✅:**
- Language detection incorrect on monorepos — fixed: `package_json` source now wins over
  `cargo_toml`; monorepos correctly detected as `TypeScript + Rust` with `Node.js + Rust (native)` runtime.
- Project name returns `"unknown"` — fixed: falls back to root directory name when no
  `package.json` / `Cargo.toml` is present.
- Hidden directory detection broken — fixed: `probe_hidden_signals()` performs shallow
  `is_dir()` checks on `.git/`, `.github/workflows/` outside the main walk.
- Folder list polluted by test/docs/fixture directories — fixed: `SKIP_DIRS` expanded
  (docs, examples, bench, tests, workflows etc.); `is_noisy_dir_name()` filters numbered
  dirs (`01-components`), Next.js route interception (`(.)page`), and single-char dirs.
- URL-encoded folder names (`%5F`) — cannot reproduce on current path; monitoring.
- Rust unit tests restored — 83 tests passing (was 65).

**Remaining folder pollution (v0.2.0):**
- Next.js dynamic route segments — `[slug]/`, `[id]/`, `[...params]/` still appear in
  folder list. Fix: add `starts_with('[')` guard to `is_noisy_dir_name()` in `walker.rs`.
- Deep monorepo folder pollution — extremely large monorepos (e.g. next.js itself) expose
  deeply nested package internals. Fix: depth cap on `collect_top_folders` in `enricher.rs`.

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

| Version | Packages | Install | Status |
|---|---|---|---|
| v0.1.x → v0.1.10 | `@ekaone/repo-intel` | `pnpm add -g @ekaone/repo-intel` | ✅ Shipped |
| v0.2.x | + `@ekaone/repo-intel-sdk` | `pnpm add @ekaone/repo-intel-sdk` | 🔧 Planned |
| v0.3.0 | + `@ekaone/repo-intel-native` | `pnpm add @ekaone/repo-intel-native` | 🔮 Future |

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