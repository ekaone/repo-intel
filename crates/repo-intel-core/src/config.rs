use crate::error::{RepoIntelError, Result};
use serde::{Deserialize, Serialize};
use std::path::Path;

// ── Top-level config ──────────────────────────────────────────────────────────

/// Parsed representation of `.repo-intel.toml`.
/// All fields are optional — missing keys fall back to defaults.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Config {
    #[serde(default)]
    pub ai: AiConfig,

    #[serde(default)]
    pub output: OutputConfig,

    #[serde(default)]
    pub project: ProjectConfig,

    #[serde(default)]
    pub stack: StackConfig,
}

// ── [ai] ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiConfig {
    /// Which LLM provider to use.
    pub provider: AiProvider,
    /// Model name override (provider-specific).
    pub model: Option<String>,
    /// Name of the env var that holds the API key. Never the key itself.
    pub api_key_env: String,
    /// Base URL override (mainly for Ollama / self-hosted endpoints).
    pub base_url: Option<String>,
}

impl Default for AiConfig {
    fn default() -> Self {
        Self {
            provider: AiProvider::Anthropic,
            model: None,
            api_key_env: "ANTHROPIC_API_KEY".into(),
            base_url: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum AiProvider {
    #[default]
    Anthropic,
    OpenAi,
    Ollama,
}

impl AiProvider {
    pub fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "anthropic" => Ok(Self::Anthropic),
            "openai" => Ok(Self::OpenAi),
            "ollama" => Ok(Self::Ollama),
            other => Err(RepoIntelError::InvalidProvider {
                value: other.to_string(),
            }),
        }
    }

    pub fn default_model(&self) -> &'static str {
        match self {
            Self::Anthropic => "claude-sonnet-4-20250514",
            Self::OpenAi => "gpt-4o",
            Self::Ollama => "llama3.2",
        }
    }

    pub fn default_api_key_env(&self) -> &'static str {
        match self {
            Self::Anthropic => "ANTHROPIC_API_KEY",
            Self::OpenAi => "OPENAI_API_KEY",
            Self::Ollama => "", // Ollama is local, no key required
        }
    }
}

// ── [output] ──────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputConfig {
    /// Directory where agent `.md` files are written.
    pub dir: String,
    /// Output format (only "markdown" for MVP).
    pub format: OutputFormat,
}

impl Default for OutputConfig {
    fn default() -> Self {
        Self {
            dir: "./agents".into(),
            format: OutputFormat::Markdown,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum OutputFormat {
    #[default]
    Markdown,
    // JSON and YAML are v0.2.0 additions
}

// ── [project] ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProjectConfig {
    /// Paths to exclude from scanning (relative to repo root).
    #[serde(default)]
    pub exclude: Vec<String>,
}

// ── [stack] ───────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct StackConfig {
    /// Force-include skill names that the scanner might miss.
    #[serde(default)]
    pub override_skills: Vec<String>,
}

// ── Loader ────────────────────────────────────────────────────────────────────

impl Config {
    /// Load `.repo-intel.toml` from `root`, falling back to defaults if not found.
    pub fn load(root: &Path) -> Result<Self> {
        let config_path = root.join(".repo-intel.toml");

        if !config_path.exists() {
            return Ok(Self::default());
        }

        let raw = std::fs::read_to_string(&config_path).map_err(|e| RepoIntelError::FileRead {
            path: config_path.clone(),
            source: e,
        })?;

        let config: Config = toml::from_str(&raw).map_err(|e| RepoIntelError::ConfigParse {
            reason: e.to_string(),
        })?;

        Ok(config)
    }

    /// Resolve the effective model name — config override or provider default.
    pub fn effective_model(&self) -> &str {
        self.ai
            .model
            .as_deref()
            .unwrap_or_else(|| self.ai.provider.default_model())
    }

    /// Read the API key from the configured env var.
    /// Returns `None` for Ollama (no key required).
    pub fn resolve_api_key(&self) -> Result<Option<String>> {
        if self.ai.provider == AiProvider::Ollama {
            return Ok(None);
        }

        let var = &self.ai.api_key_env;
        if var.is_empty() {
            return Ok(None);
        }

        std::env::var(var)
            .map(Some)
            .map_err(|_| RepoIntelError::MissingApiKey { var: var.clone() })
    }
}
