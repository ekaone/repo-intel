//! # repo-intel-core
//!
//! Rust core for `repo-intel`. Scans a repository, detects its tech stack,
//! and builds a structured `RepoContext` that the TypeScript AI layer consumes.
//!
//! ## Pipeline
//!
//! ```text
//! scan(root)  →  ScanResult
//!     └─ detect(scan)  →  StackResult
//!             └─ build(stack)  →  RepoContext  →  context.json (stdout)
//! ```
//!
//! ## Usage (library)
//!
//! ```rust,no_run
//! use std::path::Path;
//! use repo_intel_core::{config::Config, run_pipeline};
//!
//! let root = Path::new(".");
//! let cfg  = Config::load(root).unwrap();
//! let ctx  = run_pipeline(root, &cfg).unwrap();
//! println!("{}", serde_json::to_string_pretty(&ctx).unwrap());
//! ```

// ── Module declarations ───────────────────────────────────────────────────────

pub mod config;
pub mod context;
pub mod detector;
pub mod error;
pub mod scanner;
pub mod types;

// ── Public re-exports ─────────────────────────────────────────────────────────

pub use config::Config;
pub use error::{RepoIntelError, Result};
pub use types::{
    AgentDoc, ArchMeta, ArchStyle, FolderMap, ProjectMeta, RepoContext, ScanResult, SignalFile,
    SignalKind, Skill, SkillSource, StackResult,
};

// ── Top-level convenience function ───────────────────────────────────────────

use std::path::Path;

/// Run the full Rust pipeline: scan → detect → build context.
///
/// This is the single entry point used by `main.rs` and integration tests.
pub fn run_pipeline(root: &Path, cfg: &Config) -> Result<RepoContext> {
    // Phase 1 — scan
    let scan = scanner::scan(root, cfg)?;

    // Phase 2 — detect stack
    let stack = detector::detect(&scan)?;

    // Phase 3 — build context
    let ctx = context::build(root, &scan, stack, cfg)?;

    Ok(ctx)
}
