#!/usr/bin/env node
/**
 * repo-intel CLI entry point.
 *
 * Usage:
 *   repo-intel generate [options]
 *   repo-intel scan     [options]   ← thin wrapper around the Rust binary
 *
 * This file is the value of `"bin": { "repo-intel": "./dist/cli.js" }` in
 * package.json. tsup compiles it to CJS so the shebang works on all platforms.
 */

import { parseArgs } from "node:util";
import { resolve } from "node:path";
import { runPipeline } from "./pipeline/index";
import { runRustPipeline } from "./pipeline/runner";
import {
  getBinaryPath,
  getSupportedPlatforms,
  isPlatformSupported,
} from "./loader";
import type { GenerateOptions, AIProviderName } from "./types";

// ── Version ───────────────────────────────────────────────────────────────────
// Inlined at build time by tsup via `define` — fallback for dev.
declare const __VERSION__: string;
const VERSION = typeof __VERSION__ !== "undefined" ? __VERSION__ : "0.0.0-dev";

// ── Entrypoint ────────────────────────────────────────────────────────────────

async function main(): Promise<void> {
  const argv = process.argv.slice(2);

  // Handle top-level flags before subcommand parsing
  if (argv.length === 0 || argv.includes("--help") || argv.includes("-h")) {
    printHelp();
    process.exit(0);
  }

  if (argv.includes("--version") || argv.includes("-v")) {
    console.log(`repo-intel/${VERSION}`);
    process.exit(0);
  }

  const subcommand = argv[0];

  switch (subcommand) {
    case "generate":
      await runGenerate(argv.slice(1));
      break;

    case "scan":
      await runScan(argv.slice(1));
      break;

    default:
      console.error(`repo-intel: unknown command '${subcommand}'`);
      console.error(`Run 'repo-intel --help' for usage.`);
      process.exit(1);
  }
}

// ── generate command ──────────────────────────────────────────────────────────

async function runGenerate(argv: string[]): Promise<void> {
  let parsed: ReturnType<typeof parseArgs>;

  try {
    parsed = parseArgs({
      args: argv,
      options: {
        root: { type: "string", short: "r", default: process.cwd() },
        output: { type: "string", short: "o" },
        provider: { type: "string", short: "p" },
        model: { type: "string", short: "m" },
        "no-ai": { type: "boolean", default: false },
        "dry-run": { type: "boolean", default: false },
        debug: { type: "boolean", default: false },
        help: { type: "boolean", short: "h", default: false },
      },
      allowPositionals: false,
    });
  } catch (err) {
    console.error(
      `repo-intel generate: ${err instanceof Error ? err.message : String(err)}`,
    );
    console.error(`Run 'repo-intel generate --help' for usage.`);
    process.exit(1);
  }

  const values = parsed.values;

  if (values.help) {
    printGenerateHelp();
    process.exit(0);
  }

  // ── Validate provider ────────────────────────────────────────────────────
  const VALID_PROVIDERS: AIProviderName[] = ["anthropic", "openai", "ollama"];
  const provider = values.provider as AIProviderName | undefined;

  if (provider && !VALID_PROVIDERS.includes(provider)) {
    console.error(`repo-intel generate: unknown provider '${provider}'`);
    console.error(`Valid providers: ${VALID_PROVIDERS.join(", ")}`);
    process.exit(1);
  }

  const options: GenerateOptions = {
    root: resolve(values.root as string),
    noAi: values["no-ai"] as boolean,
    dryRun: values["dry-run"] as boolean,
    debug: values.debug as boolean,
    provider,
    outputDir: values.output as string | undefined,
  };

  try {
    await runPipeline(options);
  } catch (err) {
    const message = err instanceof Error ? err.message : String(err);
    console.error(`\nrepo-intel: error during generation:\n${message}`);

    if (options.debug) {
      console.error("\nStack trace:");
      console.error(err instanceof Error ? err.stack : String(err));
    } else {
      console.error("\nRun with --debug for more detail.");
    }

    process.exit(1);
  }
}

// ── scan command ──────────────────────────────────────────────────────────────
// Thin passthrough to the Rust binary's scan output.
// Useful for debugging the context.json that will be passed to the AI layer.

async function runScan(argv: string[]): Promise<void> {
  let parsed: ReturnType<typeof parseArgs>;

  try {
    parsed = parseArgs({
      args: argv,
      options: {
        root: { type: "string", short: "r", default: process.cwd() },
        pretty: { type: "boolean", short: "p", default: false },
        debug: { type: "boolean", default: false },
        help: { type: "boolean", short: "h", default: false },
      },
      allowPositionals: false,
    });
  } catch (err) {
    console.error(
      `repo-intel scan: ${err instanceof Error ? err.message : String(err)}`,
    );
    process.exit(1);
  }

  const values = parsed.values;

  if (values.help) {
    printScanHelp();
    process.exit(0);
  }

  try {
    const root = resolve(values.root as string);
    const context = runRustPipeline(root, values.debug as boolean);

    const output = values.pretty
      ? JSON.stringify(context, null, 2)
      : JSON.stringify(context);

    process.stdout.write(output + "\n");
  } catch (err) {
    const message = err instanceof Error ? err.message : String(err);
    console.error(`\nrepo-intel scan: ${message}`);
    process.exit(1);
  }
}

// ── Help text ─────────────────────────────────────────────────────────────────

function printHelp(): void {
  console.log(
    `
repo-intel v${VERSION}
Scan a repository and generate AI agent documentation.

USAGE
  repo-intel <command> [options]

COMMANDS
  generate    Generate agent .md files for the current repository
  scan        Print the detected context.json (useful for debugging)

OPTIONS
  -v, --version    Print version and exit
  -h, --help       Print this help and exit

Run 'repo-intel <command> --help' for command-specific options.

EXAMPLES
  repo-intel generate
  repo-intel generate --no-ai
  repo-intel generate --provider openai --dry-run
  repo-intel scan --pretty
`.trim(),
  );
}

function printGenerateHelp(): void {
  console.log(
    `
repo-intel generate — Generate AI agent documentation for a repository

USAGE
  repo-intel generate [options]

OPTIONS
  -r, --root <path>       Repository root to scan (default: current directory)
  -o, --output <dir>      Output directory for agent docs (default: ./agents)
  -p, --provider <name>   AI provider: anthropic | openai | ollama (default: anthropic)
  -m, --model <name>      Model override (default: provider's default model)
      --no-ai             Skip AI — use static templates instead (free, offline)
      --dry-run           Print docs to stdout without writing files
      --debug             Emit verbose debug info to stderr
  -h, --help              Print this help and exit

ENVIRONMENT VARIABLES
  ANTHROPIC_API_KEY       Required for --provider anthropic (default)
  OPENAI_API_KEY          Required for --provider openai
  REPO_INTEL_BINARY       Override the path to the Rust binary

EXAMPLES
  repo-intel generate
  repo-intel generate --root ./my-project
  repo-intel generate --provider openai
  repo-intel generate --provider ollama --model llama3.2
  repo-intel generate --no-ai --output ./docs/agents
  repo-intel generate --dry-run --debug
`.trim(),
  );
}

function printScanHelp(): void {
  console.log(
    `
repo-intel scan — Print the detected context.json for a repository

USAGE
  repo-intel scan [options]

OPTIONS
  -r, --root <path>    Repository root to scan (default: current directory)
  -p, --pretty         Pretty-print JSON output
      --debug          Emit verbose debug info to stderr
  -h, --help           Print this help and exit

EXAMPLES
  repo-intel scan
  repo-intel scan --pretty
  repo-intel scan --root ./my-project --pretty
`.trim(),
  );
}

// ── Run ───────────────────────────────────────────────────────────────────────

main().catch((err) => {
  console.error(
    `repo-intel: unexpected error: ${err instanceof Error ? err.message : String(err)}`,
  );
  process.exit(1);
});
