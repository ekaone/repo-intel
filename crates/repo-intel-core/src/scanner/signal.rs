use std::path::Path;
use walkdir::WalkDir;

use crate::error::{RepoIntelError, Result};
use crate::types::{SignalFile, SignalKind};

/// Scan `root` for all known signal files and return them with their contents.
///
/// Signal files are read sequentially with `std::fs::read_to_string` — no async.
/// There are at most ~15 of them so the overhead is negligible.
pub fn collect_signals(root: &Path) -> Result<(Vec<SignalFile>, Option<String>)> {
    let mut signals: Vec<SignalFile> = Vec::new();
    let mut readme_excerpt: Option<String> = None;

    // ── Root-level signal files ───────────────────────────────────────────────
    let root_signals: &[(&str, SignalKind)] = &[
        ("package.json",          SignalKind::PackageJson),
        ("tsconfig.json",         SignalKind::TsConfig),
        ("Cargo.toml",            SignalKind::CargoToml),
        ("pyproject.toml",        SignalKind::PyProject),
        ("requirements.txt",      SignalKind::RequirementsTxt),
        ("go.mod",                SignalKind::GoMod),
        ("Dockerfile",            SignalKind::Dockerfile),
        ("docker-compose.yml",    SignalKind::DockerCompose),
        ("docker-compose.yaml",   SignalKind::DockerCompose),
        (".repo-intel.toml",      SignalKind::RepoIntelConfig),
    ];

    for (filename, kind) in root_signals {
        let path = root.join(filename);
        if path.exists() {
            if let Some(signal) = read_signal(&path, kind.clone())? {
                signals.push(signal);
            }
        }
    }

    // ── README (first 500 chars only) ─────────────────────────────────────────
    for name in &["README.md", "README.MD", "readme.md", "Readme.md"] {
        let path = root.join(name);
        if path.exists() {
            let content = read_file_content(&path)?;
            readme_excerpt = Some(excerpt(&content, 500));

            signals.push(SignalFile {
                kind: SignalKind::ReadmeMd,
                path: path.clone(),
                content: excerpt(&content, 500),
            });
            break; // only the first README found
        }
    }

    // ── .github/workflows/*.yml ───────────────────────────────────────────────
    // We only need to know CI is present, not parse the full YAML.
    // Collect just the first workflow file found — one is enough.
    let workflows_dir = root.join(".github").join("workflows");
    if workflows_dir.is_dir() {
        for entry in WalkDir::new(&workflows_dir).max_depth(1) {
            let entry = match entry {
                Ok(e) => e,
                Err(_) => continue,
            };

            let path = entry.path();
            let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");

            if (ext == "yml" || ext == "yaml") && path.is_file() {
                if let Some(signal) = read_signal(path, SignalKind::GithubWorkflow)? {
                    signals.push(signal);
                    break; // one is enough to confirm CI presence
                }
            }
        }
    }

    // ── Workspace-level package.json files (monorepo detection) ───────────────
    // Check one level into `packages/` and `apps/` for child package.json files.
    for workspace_dir in &["packages", "apps"] {
        let dir = root.join(workspace_dir);
        if !dir.is_dir() {
            continue;
        }

        for entry in WalkDir::new(&dir).min_depth(1).max_depth(2) {
            let entry = match entry {
                Ok(e) => e,
                Err(_) => continue,
            };

            let path = entry.path();
            if path.file_name().and_then(|n| n.to_str()) == Some("package.json")
                && path != root.join("package.json")
            {
                if let Some(signal) = read_signal(path, SignalKind::PackageJson)? {
                    signals.push(signal);
                }
            }
        }
    }

    Ok((signals, readme_excerpt))
}

/// Read a single signal file. Returns `None` if the file is empty or unreadable.
fn read_signal(path: &Path, kind: SignalKind) -> Result<Option<SignalFile>> {
    let content = match std::fs::read_to_string(path) {
        Ok(c) if c.trim().is_empty() => return Ok(None),
        Ok(c) => c,
        Err(e) => {
            // Non-fatal: log to stderr and continue
            eprintln!("warn: could not read {}: {e}", path.display());
            return Ok(None);
        }
    };

    Ok(Some(SignalFile {
        kind,
        path: path.to_path_buf(),
        content,
    }))
}

/// Read a file's full content, returning a `FileRead` error on failure.
fn read_file_content(path: &Path) -> Result<String> {
    std::fs::read_to_string(path).map_err(|e| RepoIntelError::FileRead {
        path: path.to_path_buf(),
        source: e,
    })
}

/// Return the first `max_chars` characters of `s`, trimmed.
/// Cuts at a char boundary to avoid panics on multibyte text.
fn excerpt(s: &str, max_chars: usize) -> String {
    let trimmed = s.trim();
    if trimmed.chars().count() <= max_chars {
        return trimmed.to_string();
    }

    // Find the byte index of the max_chars-th char boundary
    let cut = trimmed
        .char_indices()
        .nth(max_chars)
        .map(|(i, _)| i)
        .unwrap_or(trimmed.len());

    trimmed[..cut].to_string()
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn setup(files: &[(&str, &str)]) -> TempDir {
        let dir = TempDir::new().unwrap();
        for (path, content) in files {
            let full = dir.path().join(path);
            if let Some(parent) = full.parent() {
                fs::create_dir_all(parent).unwrap();
            }
            fs::write(full, content).unwrap();
        }
        dir
    }

    #[test]
    fn detects_package_json() {
        let dir = setup(&[("package.json", r#"{"name":"my-app","dependencies":{"react":"^18"}}"#)]);
        let (signals, _) = collect_signals(dir.path()).unwrap();
        assert!(signals.iter().any(|s| s.kind == SignalKind::PackageJson));
    }

    #[test]
    fn detects_cargo_toml() {
        let dir = setup(&[("Cargo.toml", "[package]\nname = \"my-crate\"\nversion = \"0.1.0\"")]);
        let (signals, _) = collect_signals(dir.path()).unwrap();
        assert!(signals.iter().any(|s| s.kind == SignalKind::CargoToml));
    }

    #[test]
    fn readme_excerpt_is_truncated() {
        let long_readme = "A".repeat(1000);
        let dir = setup(&[("README.md", &long_readme)]);
        let (_, excerpt) = collect_signals(dir.path()).unwrap();
        let excerpt = excerpt.unwrap();
        assert!(excerpt.len() <= 500);
    }

    #[test]
    fn detects_github_workflow() {
        let dir = setup(&[(".github/workflows/ci.yml", "on: [push]")]);
        let (signals, _) = collect_signals(dir.path()).unwrap();
        assert!(signals.iter().any(|s| s.kind == SignalKind::GithubWorkflow));
    }

    #[test]
    fn detects_dockerfile() {
        let dir = setup(&[("Dockerfile", "FROM node:20")]);
        let (signals, _) = collect_signals(dir.path()).unwrap();
        assert!(signals.iter().any(|s| s.kind == SignalKind::Dockerfile));
    }

    #[test]
    fn no_signals_on_empty_repo() {
        let dir = TempDir::new().unwrap();
        let (signals, readme) = collect_signals(dir.path()).unwrap();
        assert!(signals.is_empty());
        assert!(readme.is_none());
    }

    #[test]
    fn excerpt_handles_multibyte_chars() {
        let s = "こんにちは世界".repeat(100); // Japanese chars are 3 bytes each
        let result = excerpt(&s, 5);
        assert_eq!(result.chars().count(), 5);
    }
}