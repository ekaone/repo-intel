pub mod signal;
pub mod walker;

use std::path::Path;

use crate::config::Config;
use crate::error::{RepoIntelError, Result};
use crate::types::ScanResult;
use walker::DEPTH_CAP;

/// Scan `root` and return a `ScanResult` ready for the detector.
///
/// Steps:
/// 1. Validate `root` exists and is a directory.
/// 2. Walk the directory tree → `folder_map` + `file_patterns`.
/// 3. Collect signal files → `signal_files` + `readme_excerpt`.
pub fn scan(root: &Path, cfg: &Config) -> Result<ScanResult> {
    // ── Validate root ─────────────────────────────────────────────────────────
    if !root.exists() {
        return Err(RepoIntelError::RootNotFound {
            path: root.to_path_buf(),
        });
    }

    if !root.is_dir() {
        return Err(RepoIntelError::RootNotDirectory {
            path: root.to_path_buf(),
        });
    }

    // ── Walk directory tree ───────────────────────────────────────────────────
    let (mut folder_map, file_patterns) = walker::walk(root, DEPTH_CAP);

    // Remove folders that are in the user's exclude list
    for excluded in &cfg.project.exclude {
        // Normalise: strip trailing slash if present
        let key = excluded.trim_end_matches('/');
        folder_map.remove(key);
    }

    // ── Collect signal files ──────────────────────────────────────────────────
    let (signal_files, readme_excerpt) = signal::collect_signals(root)?;

    Ok(ScanResult {
        root: root.to_path_buf(),
        signal_files,
        folder_map,
        file_patterns,
        readme_excerpt,
    })
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn default_cfg() -> Config {
        Config::default()
    }

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
    fn scan_returns_error_on_missing_root() {
        let result = scan(Path::new("/this/does/not/exist"), &default_cfg());
        assert!(matches!(result, Err(RepoIntelError::RootNotFound { .. })));
    }

    #[test]
    fn scan_nextjs_fixture() {
        let dir = setup(&[
            ("package.json", r#"{"name":"my-app","dependencies":{"next":"14","react":"18","tailwindcss":"3"}}"#),
            ("tsconfig.json", r#"{"compilerOptions":{"target":"ES2020"}}"#),
            ("src/components/Button.tsx", ""),
            ("src/hooks/useAuth.ts", ""),
            ("src/services/api.ts", ""),
            ("src/app/page.tsx", ""),
            ("src/auth.test.ts", ""),
            ("tailwind.config.ts", ""),
            ("README.md", "# My App\nA dashboard application."),
        ]);

        let result = scan(dir.path(), &default_cfg()).unwrap();

        // Signal files detected
        assert!(result.signal_files.iter().any(|s| s.kind == crate::types::SignalKind::PackageJson));
        assert!(result.signal_files.iter().any(|s| s.kind == crate::types::SignalKind::TsConfig));

        // Folders detected
        assert!(result.folder_map.contains_key("components"));
        assert!(result.folder_map.contains_key("hooks"));
        assert!(result.folder_map.contains_key("services"));

        // Patterns detected
        assert!(result.file_patterns.contains(&".tsx".to_string()));
        assert!(result.file_patterns.contains(&"*.test.ts".to_string()));

        // README excerpt present
        assert!(result.readme_excerpt.is_some());
    }

    #[test]
    fn scan_respects_exclude_config() {
        let dir = setup(&[
            ("src/index.ts", ""),
            ("legacy/old.ts", ""),
        ]);

        let mut cfg = default_cfg();
        cfg.project.exclude = vec!["legacy".into()];

        let result = scan(dir.path(), &cfg).unwrap();
        assert!(!result.folder_map.contains_key("legacy"));
        assert!(result.folder_map.contains_key("src"));
    }

    #[test]
    fn scan_empty_repo_does_not_panic() {
        let dir = TempDir::new().unwrap();
        let result = scan(dir.path(), &default_cfg());
        assert!(result.is_ok());

        let scan = result.unwrap();
        assert!(scan.signal_files.is_empty());
        assert!(scan.readme_excerpt.is_none());
    }
}