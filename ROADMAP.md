# repo-intel Roadmap

## v0.1.0 — CLI MVP
> Status: In progress

- Rust core: metadata-scanner, stack-detector, context-builder
- TypeScript: CLI, AI layer (Anthropic/OpenAI/Ollama), fallback static gen
- stdout bridge between Rust and JS
- Platform binaries: Linux x64, macOS ARM64/x64, Windows x64
- Published to npm as `repo-intel`

## v0.2.0 — Speed + Incremental Intelligence
> Status: Planned

- Incremental cache system (< 5ms on cache hit)
- Rayon parallel file analysis (4–8x faster on large repos)
- Git diff detection (skip unaffected agent roles)
- Watch mode (`repo-intel watch`)
- YAML output format (targeting agentskills.io open standard)
- `repo-intel stale` command — detect outdated agent docs
- YAML / JSON output formats
- Custom prompt templates

## v0.2.x — SDK
> Status: Planned

- New package: `repo-intel-sdk`
- Programmatic API — composable individual pipeline steps
- Same platform binaries as CLI (no new Rust code needed)
- Low effort — re-exports existing pipeline with clean public API

## v0.3.0 — Native Bridge
> Status: Future

- New package: `repo-intel-native`
- napi-rs bridge replaces stdout (direct in-process Rust calls)
- Same API as `repo-intel-sdk` — drop-in replacement
- Targets: watch mode in IDE extensions, maximum performance use cases
- Distribution: `.node` files per Node.js ABI × platform

## Package Evolution

| Version | Packages |
|---|---|
| v0.1.0 | `repo-intel` (CLI) |
| v0.2.x | + `repo-intel-sdk` |
| v0.3.0 | + `repo-intel-native` |