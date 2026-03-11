use std::path::PathBuf;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum RepoIntelError {
    // ── I/O ──────────────────────────────────────────────────────────────────
    #[error("Failed to read file '{path}': {source}")]
    FileRead {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("Failed to write file '{path}': {source}")]
    FileWrite {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("Failed to walk directory '{path}': {source}")]
    DirWalk {
        path: PathBuf,
        #[source]
        source: walkdir::Error,
    },

    // ── Parsing ───────────────────────────────────────────────────────────────
    #[error("Failed to parse package.json at '{path}': {reason}")]
    PackageJsonParse { path: PathBuf, reason: String },

    #[error("Failed to parse Cargo.toml at '{path}': {reason}")]
    CargoTomlParse { path: PathBuf, reason: String },

    #[error("Failed to parse .repo-intel.toml: {reason}")]
    ConfigParse { reason: String },

    #[error("Failed to serialize context to JSON: {source}")]
    JsonSerialize {
        #[source]
        source: serde_json::Error,
    },

    // ── Scanner ───────────────────────────────────────────────────────────────
    #[error("Scan root does not exist: '{path}'")]
    RootNotFound { path: PathBuf },

    #[error("Scan root is not a directory: '{path}'")]
    RootNotDirectory { path: PathBuf },

    // ── Config ────────────────────────────────────────────────────────────────
    #[error("Invalid AI provider '{value}'. Expected: anthropic | openai | ollama")]
    InvalidProvider { value: String },

    #[error("API key env var '{var}' is not set")]
    MissingApiKey { var: String },

    // ── Pipeline ──────────────────────────────────────────────────────────────
    #[error("No agent roles could be inferred from the scanned repository")]
    NoRolesDetected,

    #[error("Context build failed: {reason}")]
    ContextBuild { reason: String },
}

/// Convenience alias — most functions return this.
pub type Result<T> = std::result::Result<T, RepoIntelError>;