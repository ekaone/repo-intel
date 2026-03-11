use crate::error::Error;
use serde::{Deserialize, Serialize};
use std::path::Path;

/// Configuration loaded from `.repo-intel.toml`.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Config {
    #[serde(default)]
    pub scan: ScanConfig,

    #[serde(default)]
    pub ai: AiConfig,

    #[serde(default)]
    pub output: OutputConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanConfig {
    /// Directory names to skip during traversal
    #[serde(default = "default_skip_dirs")]
    pub skip_dirs: Vec<String>,
    /// Maximum directory depth (0 = unlimited)
    #[serde(default)]
    pub max_depth: usize,
}

impl Default for ScanConfig {
    fn default() -> Self {
        Self {
            skip_dirs: default_skip_dirs(),
            max_depth: 0,
        }
    }
}

fn default_skip_dirs() -> Vec<String> {
    vec![
        "node_modules".into(),
        "target".into(),
        ".git".into(),
        "dist".into(),
        "build".into(),
        ".next".into(),
        ".turbo".into(),
        "coverage".into(),
    ]
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AiConfig {
    /// AI provider: "anthropic" | "openai" | "ollama"
    #[serde(default)]
    pub provider: Option<String>,
    /// Model name override
    #[serde(default)]
    pub model: Option<String>,
    /// API key (if not using environment variable)
    #[serde(default)]
    pub api_key: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputConfig {
    /// Directory to write generated agent docs into
    #[serde(default = "default_output_dir")]
    pub dir: String,
}

impl Default for OutputConfig {
    fn default() -> Self {
        Self {
            dir: default_output_dir(),
        }
    }
}

fn default_output_dir() -> String {
    "agents".into()
}

/// Load config from an optional explicit path, or from the default `.repo-intel.toml`.
pub fn load(path: Option<&Path>) -> Result<Config, Error> {
    let config_path = match path {
        Some(p) => p.to_path_buf(),
        None => {
            let default = Path::new(".repo-intel.toml");
            if !default.exists() {
                return Ok(Config::default());
            }
            default.to_path_buf()
        }
    };

    let contents = std::fs::read_to_string(&config_path)
        .map_err(|e| Error::ConfigRead(config_path.display().to_string(), e))?;

    toml::from_str::<Config>(&contents)
        .map_err(|e| Error::ConfigParse(config_path.display().to_string(), e.to_string()))
}
