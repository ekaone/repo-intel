use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Raw output of the scanner step.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanResult {
    /// Absolute path to the scanned root.
    pub root: PathBuf,
    /// Map of relative dir path → list of filenames found inside.
    pub folder_map: HashMap<String, Vec<String>>,
    /// Signal files detected: relative path → file contents (if readable).
    pub signal_files: HashMap<String, String>,
    /// All unique file extensions found in the repo.
    pub extensions: Vec<String>,
}

/// A detected skill / capability.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Skill {
    /// Short name, e.g. "nextjs", "react", "axum"
    pub name: String,
    /// Human-readable category, e.g. "framework", "language", "tooling"
    pub category: String,
    /// Detection confidence between 0.0 and 1.0
    pub confidence: f32,
    /// Which signals contributed to this detection
    pub signals: Vec<String>,
}

/// Output of the detector step.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StackResult {
    /// Primary language(s) detected
    pub languages: Vec<Skill>,
    /// Frameworks / libraries detected
    pub frameworks: Vec<Skill>,
    /// Tooling detected (bundlers, linters, CI, etc.)
    pub tooling: Vec<Skill>,
    /// Inferred architecture patterns
    pub architecture: Vec<String>,
}

/// Role that an AI agent should be generated for.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentRole {
    pub id: String,
    pub title: String,
    pub description: String,
}

/// Final enriched context — serialised to context.json / stdout.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepoContext {
    /// Repo root name (last path segment)
    pub name: String,
    /// Detected tech stack
    pub stack: StackResult,
    /// Agent roles to generate docs for
    pub agent_roles: Vec<AgentRole>,
    /// README excerpt (first 500 chars), if present
    pub readme_excerpt: Option<String>,
    /// Whether a .git directory was found
    pub has_git: bool,
    /// Whether a Dockerfile / docker-compose was found
    pub has_docker: bool,
    /// Whether a CI config was found (.github/workflows, .gitlab-ci.yml, etc.)
    pub has_ci: bool,
    /// Schema version for forward-compat
    pub schema_version: String,
}
