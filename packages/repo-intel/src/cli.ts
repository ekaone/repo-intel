import { parseArgs } from "node:util";
import { run } from "./pipeline/index.js";

const { values, positionals } = parseArgs({
  args: process.argv.slice(2),
  options: {
    root: { type: "string", short: "r", default: "." },
    config: { type: "string", short: "c" },
    output: { type: "string", short: "o", default: "agents" },
    provider: { type: "string", short: "p" },
    model: { type: "string", short: "m" },
    "no-ai": { type: "boolean", default: false },
    help: { type: "boolean", short: "h", default: false },
    version: { type: "boolean", short: "v", default: false },
  },
  allowPositionals: true,
});

if (values.version) {
  const pkg = await import("../package.json", { with: { type: "json" } });
  console.log(`repo-intel v${pkg.default.version}`);
  process.exit(0);
}

if (values.help || positionals.length === 0) {
  console.log(`
repo-intel — scan your repo and generate AI agent context files

USAGE:
  repo-intel <command> [options]

COMMANDS:
  scan      Scan repo and print context.json to stdout
  generate  Run full pipeline: scan → AI → write agent docs

OPTIONS:
  -r, --root <path>       Root directory to scan (default: .)
  -c, --config <path>     Path to .repo-intel.toml config file
  -o, --output <dir>      Output directory for agent docs (default: agents)
  -p, --provider <name>   AI provider: anthropic | openai | ollama
  -m, --model <name>      Model name override
      --no-ai             Skip AI and use static fallback templates
  -h, --help              Show this help message
  -v, --version           Show version
`);
  process.exit(0);
}

const command = positionals[0];

await run({
  command: command as "scan" | "generate",
  root: values.root!,
  config: values.config,
  output: values.output!,
  provider: values.provider,
  model: values.model,
  noAi: values["no-ai"]!,
});
