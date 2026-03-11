use std::path::PathBuf;
use std::process;

use clap::{Parser, Subcommand};
use repo_intel_core::{config::Config, context::serializer, run_pipeline};

// ── CLI definition ────────────────────────────────────────────────────────────

#[derive(Debug, Parser)]
#[command(
    name    = "repo-intel",
    version = env!("CARGO_PKG_VERSION"),
    about   = "Scan a repository and generate AI agent documentation",
    long_about = None,
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Scan a repository and output context.json (consumed by the JS AI layer)
    Scan {
        /// Repository root to scan (defaults to current directory)
        #[arg(short, long, default_value = ".")]
        root: PathBuf,

        /// Output compact JSON to stdout (default — used by JS wrapper)
        #[arg(long)]
        json: bool,

        /// Pretty-print JSON for human inspection
        #[arg(long)]
        pretty: bool,

        /// Write output to a file instead of stdout
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// Run the full pipeline: scan → detect → build context → print JSON
    /// The JS wrapper spawns this command and reads stdout as context.json.
    Generate {
        /// Repository root to scan (defaults to current directory)
        #[arg(short, long, default_value = ".")]
        root: PathBuf,

        /// Skip the AI layer — JS wrapper will use static fallback generation
        #[arg(long)]
        no_ai: bool,

        /// Print what would be generated without writing any files
        #[arg(long)]
        dry_run: bool,

        /// Emit verbose debug info to stderr
        #[arg(long)]
        debug: bool,

        /// AI provider override: anthropic | openai | ollama
        #[arg(long)]
        provider: Option<String>,
    },
}

// ── Entry point ───────────────────────────────────────────────────────────────

fn main() {
    let cli = Cli::parse();

    if let Err(e) = run(cli) {
        eprintln!("error: {e}");
        process::exit(1);
    }
}

fn run(cli: Cli) -> repo_intel_core::Result<()> {
    match cli.command {
        // ── scan ──────────────────────────────────────────────────────────────
        Commands::Scan { root, pretty, output, .. } => {
            let cfg = Config::load(&root)?;
            let ctx = run_pipeline(&root, &cfg)?;

            let json = if pretty {
                serde_json::to_string_pretty(&ctx)
            } else {
                serde_json::to_string(&ctx)
            }
            .map_err(|e| repo_intel_core::RepoIntelError::JsonSerialize { source: e })?;

            match output {
                Some(path) => {
                    serializer::write_to_file(&json, &path)?;
                    eprintln!("✓ context written to {}", path.display());
                }
                None => {
                    // Compact JSON to stdout — JS wrapper reads this
                    println!("{json}");
                }
            }
        }

        // ── generate ──────────────────────────────────────────────────────────
        Commands::Generate { root, no_ai, dry_run, debug, provider } => {
            let mut cfg = Config::load(&root)?;

            // Apply CLI provider override
            if let Some(p) = provider {
                cfg.ai.provider = repo_intel_core::config::AiProvider::from_str(&p)?;
            }

            if debug {
                eprintln!("[debug] root       = {}", root.display());
                eprintln!("[debug] provider   = {:?}", cfg.ai.provider);
                eprintln!("[debug] model      = {}", cfg.effective_model());
                eprintln!("[debug] no_ai      = {no_ai}");
                eprintln!("[debug] dry_run    = {dry_run}");
            }

            let ctx = run_pipeline(&root, &cfg)?;

            if debug {
                eprintln!("[debug] roles detected: {:?}", ctx.agent_roles);
                eprintln!("[debug] skills: {}", ctx.stack.skills.len());
            }

            // Serialize context.json to stdout — JS wrapper takes it from here
            let json = serde_json::to_string(&ctx)
                .map_err(|e| repo_intel_core::RepoIntelError::JsonSerialize { source: e })?;

            println!("{json}");

            if dry_run {
                eprintln!("(dry-run) JS layer would now call the AI provider");
            }
        }
    }

    Ok(())
}