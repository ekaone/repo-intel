use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(
    name = "repo-intel",
    version,
    about = "Scan your repository and generate AI agent context files",
    long_about = None
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Scan the repository and print context.json to stdout
    Scan {
        /// Root directory to scan (defaults to current directory)
        #[arg(short, long, default_value = ".")]
        root: PathBuf,

        /// Path to .repo-intel.toml config file
        #[arg(short, long)]
        config: Option<PathBuf>,
    },

    /// Run the full pipeline: scan → detect → build context → write agent docs
    Generate {
        /// Root directory to scan (defaults to current directory)
        #[arg(short, long, default_value = ".")]
        root: PathBuf,

        /// Path to .repo-intel.toml config file
        #[arg(short, long)]
        config: Option<PathBuf>,

        /// Output directory for generated agent docs
        #[arg(short, long, default_value = "agents")]
        output: PathBuf,

        /// Skip AI generation and use static fallback templates
        #[arg(long)]
        no_ai: bool,
    },

    /// Print version information
    Version,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Scan { root, config } => {
            let cfg = repo_intel_core::config::load(config.as_deref())?;
            let scan_result = repo_intel_core::scanner::scan(&root, &cfg)?;
            let stack_result = repo_intel_core::detector::detect(&scan_result);
            let context = repo_intel_core::context::build(&stack_result);
            let json = repo_intel_core::context::serializer::to_json(&context)?;
            println!("{json}");
        }

        Commands::Generate {
            root,
            config,
            output: _,
            no_ai: _,
        } => {
            let cfg = repo_intel_core::config::load(config.as_deref())?;
            let scan_result = repo_intel_core::scanner::scan(&root, &cfg)?;
            let stack_result = repo_intel_core::detector::detect(&scan_result);
            let context = repo_intel_core::context::build(&stack_result);
            let json = repo_intel_core::context::serializer::to_json(&context)?;
            // The JS layer consumes this JSON from stdout to drive AI generation
            println!("{json}");
        }

        Commands::Version => {
            println!("repo-intel {}", env!("CARGO_PKG_VERSION"));
        }
    }

    Ok(())
}
