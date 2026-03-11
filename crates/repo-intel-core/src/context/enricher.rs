use crate::types::{RepoContext, ScanResult};
use std::path::Path;

/// Enrich a `RepoContext` in-place using the original `ScanResult` for file access.
pub fn enrich(ctx: &mut RepoContext, scan: &ScanResult) {
    // Set repo name from root directory name
    ctx.name = scan
        .root
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| "unknown".into());

    // README excerpt
    ctx.readme_excerpt = read_readme_excerpt(&scan.root);

    // Git presence
    ctx.has_git = scan.root.join(".git").exists();

    // Docker presence
    ctx.has_docker = scan.signal_files.contains_key("Dockerfile")
        || scan.signal_files.contains_key("docker-compose.yml")
        || scan.signal_files.contains_key("docker-compose.yaml");

    // CI presence
    ctx.has_ci = detect_ci(&scan.root, &scan.folder_map);
}

fn read_readme_excerpt(root: &Path) -> Option<String> {
    for name in &["README.md", "README.MD", "readme.md", "Readme.md"] {
        let path = root.join(name);
        if let Ok(content) = std::fs::read_to_string(path) {
            let excerpt: String = content.chars().take(500).collect();
            return Some(excerpt);
        }
    }
    None
}

fn detect_ci(root: &Path, folder_map: &std::collections::HashMap<String, Vec<String>>) -> bool {
    // GitHub Actions
    if root.join(".github").join("workflows").exists() {
        return true;
    }
    // GitLab CI
    if root.join(".gitlab-ci.yml").exists() {
        return true;
    }
    // CircleCI
    if root.join(".circleci").exists() {
        return true;
    }
    // Check folder map for any CI indicators
    folder_map.keys().any(|k| {
        k.contains(".github/workflows") || k.contains(".circleci") || k.contains(".gitlab")
    })
}
