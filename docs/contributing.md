# Contributing

## Prerequisites

- [Rust](https://rustup.rs/) ≥ 1.75 with `stable` toolchain
- [Node.js](https://nodejs.org/) ≥ 18
- [pnpm](https://pnpm.io/) ≥ 9

## Development Setup

```bash
# Clone the repo
git clone https://github.com/ekaone/repo-intel
cd repo-intel

# Install JS dependencies
pnpm install

# Check Rust compiles
cargo check --all
```

## Running Tests

### Rust tests

```bash
cargo test --all
```

### Rust linting

```bash
cargo clippy --all-targets --all-features -- -D warnings
cargo fmt --all -- --check
```

### TypeScript lint + typecheck

```bash
pnpm lint
pnpm typecheck
```

## Running Benchmarks

```bash
cargo bench -p repo-intel-core
```

## Building Locally

```bash
# Build Rust (native target only)
cargo build --release -p repo-intel-core

# Build TypeScript
pnpm build
```

## Project Structure

See [architecture.md](architecture.md) for a full module breakdown.

## Commit Style

Use [Conventional Commits](https://www.conventionalcommits.org/):

- `feat:` — new feature
- `fix:` — bug fix
- `refactor:` — code refactor
- `docs:` — documentation only
- `test:` — adding or fixing tests
- `ci:` — CI/CD changes
- `chore:` — tooling / dependency updates
