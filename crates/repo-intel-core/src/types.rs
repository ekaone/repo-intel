use std::collections::HashMap;
use std::path::PathBuf;
use serde::{Deserialize, Serialize};

// ── Type aliases ──────────────────────────────────────────────────────────────

/// Maps a folder name to the list of files directly inside it.
pub type FolderMap = HashMap<String, Vec<String>>;

// ── Scanner output ────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanResult {
    pub root: PathBuf,
    pub signal_files: Vec<SignalFile>,
    /// Folder name → list of direct child filenames
    pub folder_map: FolderMap,
    /// Collected file extensions + notable name patterns (e.g. "*.test.ts")
    pub file_patterns: Vec<String>,
    pub readme_excerpt: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignalFile {
    pub kind: SignalKind,
    pub path: PathBuf,
    /// Raw file contents — callers are responsible for parsing
    pub content: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SignalKind {
    PackageJson,
    TsConfig,
    CargoToml,
    PyProject,
    RequirementsTxt,
    GoMod,
    ReadmeMd,
    Dockerfile,
    DockerCompose,
    GithubWorkflow,
    RepoIntelConfig,
}

// ── Detector output ───────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StackResult {
    pub language: String,
    pub framework: Option<String>,
    pub styling: Option<String>,
    pub state_management: Option<String>,
    pub testing: Option<String>,
    pub database: Option<String>,
    pub runtime: Option<String>,
    pub skills: Vec<Skill>,
    pub architecture_style: Option<ArchStyle>,
}

impl StackResult {
    /// Convenience: check if any skill name contains `needle` with confidence ≥ threshold.
    pub fn has_skill(&self, needle: &str, min_confidence: f32) -> bool {
        self.skills
            .iter()
            .any(|s| s.name.to_lowercase().contains(&needle.to_lowercase())
                && s.confidence >= min_confidence)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Skill {
    pub name: String,
    /// 0.0 – 1.0
    pub confidence: f32,
    pub source: SkillSource,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "type", content = "value")]
pub enum SkillSource {
    PackageJson,
    CargoToml,
    FolderName(String),
    FilePattern(String),
    ReadmeSignal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ArchStyle {
    /// `src/modules/feature/…`
    FeatureBased,
    /// `src/components`, `src/services`, `src/hooks`
    LayerBased,
    /// Minimal / flat structure
    Flat,
}

// ── Context output (final Rust output → consumed by JS) ──────────────────────

/// The contract between Rust and JS. Shape must match `context.json`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepoContext {
    pub version: String,
    pub scanned_at: String,         // ISO 8601
    pub root: PathBuf,
    pub project: ProjectMeta,
    pub stack: StackResult,
    pub architecture: ArchMeta,
    pub agent_roles: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectMeta {
    pub name: String,
    pub description: Option<String>,
    pub readme_excerpt: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchMeta {
    pub style: Option<ArchStyle>,
    /// Top-level folder names present in the repo
    pub folders: Vec<String>,
    pub has_monorepo: bool,
    pub has_docker: bool,
    pub has_ci: bool,
    pub has_git: bool,
}

// ── Agent doc (produced by JS, typed here for shared reference) ───────────────

/// Mirrors the TypeScript `AgentDoc` type. Not serialized by Rust directly,
/// but defined here so the shape is the single source of truth.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentDoc {
    pub role: String,
    pub filename: String,
    pub content: String,
    pub generated_at: String,
    pub generated_by: String,
    pub confidence: f32,
}