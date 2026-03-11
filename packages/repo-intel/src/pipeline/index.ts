import type { AnalyzeOptions, RepoContext } from "../types.js";
import { spawnRust } from "./runner.js";
import { writeAgentDocs } from "./writer.js";

export interface RunOptions {
  command: "scan" | "generate";
  root: string;
  config?: string;
  output: string;
  provider?: string;
  model?: string;
  noAi: boolean;
}

/**
 * Full pipeline: spawn Rust binary → AI generate → write files.
 */
export async function run(opts: RunOptions): Promise<void> {
  const context = await spawnRust({
    root: opts.root,
    config: opts.config,
  });

  if (opts.command === "scan") {
    console.log(JSON.stringify(context, null, 2));
    return;
  }

  await writeAgentDocs(context, {
    outputDir: opts.output,
    provider: opts.provider as AnalyzeOptions["provider"],
    model: opts.model,
    noAi: opts.noAi,
  });
}

/**
 * Public API: analyze() — full pipeline returning the context.
 */
export async function analyze(opts: AnalyzeOptions): Promise<RepoContext> {
  const context = await spawnRust({
    root: opts.root ?? ".",
    config: opts.config,
  });
  return context;
}
