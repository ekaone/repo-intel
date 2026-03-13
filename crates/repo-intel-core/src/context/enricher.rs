use std::path::Path;

use crate::types::{ArchMeta, ArchStyle, FolderMap, ProjectMeta, SignalFile, SignalKind, Skill};

/// Build `ArchMeta` and `ProjectMeta` by examining the scan outputs and the
/// file system directly.
///
/// This enrichment step adds signals that neither the deps nor folders layers
/// can provide cleanly — git presence, CI, docker, monorepo detection, and the
/// README-derived project description.
pub fn enrich(
    root: &Path,
    signal_files: &[SignalFile],
    folder_map: &FolderMap,
    readme_excerpt: Option<String>,
    skills: &[Skill],
    arch_style: Option<ArchStyle>,
) -> (ProjectMeta, ArchMeta) {
    let project = build_project_meta(signal_files, readme_excerpt);
    let arch = build_arch_meta(root, signal_files, folder_map, skills, arch_style);
    (project, arch)
}

// ── ProjectMeta ───────────────────────────────────────────────────────────────

fn build_project_meta(signal_files: &[SignalFile], readme_excerpt: Option<String>) -> ProjectMeta {
    let mut name = "unknown".to_string();
    let mut description: Option<String> = None;

    // Sort signal files by path depth (shallowest first) so the root-level
    // package.json / Cargo.toml wins over nested workspace members.
    // Without this, packages/repo-intel-win32-x64/package.json would overwrite
    // the root package.json because it appears later in the walk order.
    let mut sorted: Vec<&SignalFile> = signal_files.iter().collect();
    sorted.sort_by_key(|s| s.path.components().count());

    for signal in sorted {
        match signal.kind {
            SignalKind::PackageJson => {
                if let Ok(json) = serde_json::from_str::<serde_json::Value>(&signal.content) {
                    // Only take name if we don't have one yet (shallowest wins)
                    if name == "unknown" {
                        if let Some(n) = json.get("name").and_then(|v| v.as_str()) {
                            if !n.is_empty() {
                                name = n.to_string();
                            }
                        }
                    }
                    if description.is_none() {
                        if let Some(d) = json.get("description").and_then(|v| v.as_str()) {
                            if !d.is_empty() {
                                description = Some(d.to_string());
                            }
                        }
                    }
                }
            }
            SignalKind::CargoToml => {
                if let Ok(table) = signal.content.parse::<toml::Value>() {
                    if let Some(pkg) = table.get("package") {
                        if name == "unknown" {
                            if let Some(n) = pkg.get("name").and_then(|v| v.as_str()) {
                                if !n.is_empty() {
                                    name = n.to_string();
                                }
                            }
                        }
                        if description.is_none() {
                            if let Some(d) = pkg.get("description").and_then(|v| v.as_str()) {
                                if !d.is_empty() {
                                    description = Some(d.to_string());
                                }
                            }
                        }
                    }
                }
            }
            _ => {}
        }
    }

    ProjectMeta {
        name,
        description,
        readme_excerpt,
    }
}

// ── ArchMeta ──────────────────────────────────────────────────────────────────

fn build_arch_meta(
    root: &Path,
    signal_files: &[SignalFile],
    folder_map: &FolderMap,
    skills: &[Skill],
    arch_style: Option<ArchStyle>,
) -> ArchMeta {
    let has_git = detect_git(root);
    let has_docker = detect_docker(root, signal_files, skills);
    let has_ci = detect_ci(root, signal_files, skills);
    let has_monorepo = detect_monorepo(root, signal_files, folder_map, skills);

    // Collect meaningful top-level folder names for the prompt
    let folders = collect_top_folders(folder_map);

    ArchMeta {
        style: arch_style,
        folders,
        has_monorepo,
        has_docker,
        has_ci,
        has_git,
    }
}

// ── Detection helpers ─────────────────────────────────────────────────────────

/// Check for `.git/` directory — presence means the repo is git-tracked.
fn detect_git(root: &Path) -> bool {
    root.join(".git").is_dir()
}

/// Docker is present if:
/// - `Dockerfile` or `docker-compose.yml` exists at root, OR
/// - a `__has_docker` internal marker was emitted by the patterns layer, OR
/// - a DockerFile signal file was collected
fn detect_docker(root: &Path, signal_files: &[SignalFile], skills: &[Skill]) -> bool {
    if root.join("Dockerfile").exists()
        || root.join("docker-compose.yml").exists()
        || root.join("docker-compose.yaml").exists()
    {
        return true;
    }

    if signal_files
        .iter()
        .any(|s| matches!(s.kind, SignalKind::Dockerfile | SignalKind::DockerCompose))
    {
        return true;
    }

    // Fallback: internal marker from patterns layer
    skills.iter().any(|s| s.name == "__has_docker")
}

/// CI is present if:
/// - `.github/workflows/` contains any `.yml` file, OR
/// - `.circleci/config.yml` exists, OR
/// - `Jenkinsfile` exists, OR
/// - a `GithubWorkflow` signal was collected
fn detect_ci(root: &Path, signal_files: &[SignalFile], skills: &[Skill]) -> bool {
    if signal_files
        .iter()
        .any(|s| matches!(s.kind, SignalKind::GithubWorkflow))
    {
        return true;
    }

    if root.join(".github").join("workflows").is_dir() {
        return true;
    }

    if root.join(".circleci").join("config.yml").exists()
        || root.join(".circleci").join("config.yaml").exists()
    {
        return true;
    }

    if root.join("Jenkinsfile").exists() {
        return true;
    }

    // Fallback: internal marker from patterns layer
    skills.iter().any(|s| s.name == "__has_ci")
}

/// Monorepo is present if:
/// - `packages/` or `apps/` exists AND contains at least one child `package.json`, OR
/// - a `pnpm-workspace.yaml` / `lerna.json` / `nx.json` exists at root, OR
/// - internal `__has_monorepo` marker was emitted by the folders layer
fn detect_monorepo(
    root: &Path,
    signal_files: &[SignalFile],
    folder_map: &FolderMap,
    skills: &[Skill],
) -> bool {
    // Explicit workspace config files
    let workspace_files = [
        "pnpm-workspace.yaml",
        "pnpm-workspace.yml",
        "lerna.json",
        "nx.json",
        "turbo.json",
    ];
    if workspace_files.iter().any(|f| root.join(f).exists()) {
        return true;
    }

    // packages/ or apps/ with child package.json signals
    let workspace_dirs = ["packages", "apps"];
    let has_workspace_dir = workspace_dirs.iter().any(|d| folder_map.contains_key(*d));
    let has_child_pkg = signal_files
        .iter()
        .filter(|s| s.kind == SignalKind::PackageJson)
        .count()
        > 1;

    if has_workspace_dir && has_child_pkg {
        return true;
    }

    // Internal marker from folders layer
    skills.iter().any(|s| s.name == "__has_monorepo")
}

/// Return a sorted, deduplicated list of meaningful top-level folder names.
/// Filters out noisy or generated folder names to keep the prompt clean.
fn collect_top_folders(folder_map: &FolderMap) -> Vec<String> {
    const SKIP: &[&str] = &[
        "node_modules",
        ".git",
        "dist",
        "build",
        "target",
        ".next",
        ".nuxt",
        "coverage",
        ".turbo",
        ".cache",
        "__pycache__",
        ".venv",
        "vendor",
        ".svelte-kit",
        ".github",
        ".cargo",
        ".angular",
    ];

    let mut folders: Vec<String> = folder_map
        .keys()
        .filter(|name| !SKIP.contains(&name.as_str()) && !name.starts_with('.'))
        .cloned()
        .collect();

    folders.sort();
    folders
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::fs;
    use tempfile::TempDir;

    fn no_skills() -> Vec<Skill> {
        vec![]
    }
    fn no_signals() -> Vec<SignalFile> {
        vec![]
    }
    fn empty_folders() -> FolderMap {
        HashMap::new()
    }

    fn setup(files: &[&str]) -> TempDir {
        let dir = TempDir::new().unwrap();
        for f in files {
            let path = dir.path().join(f);
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent).unwrap();
            }
            fs::write(path, "").unwrap();
        }
        dir
    }

    #[test]
    fn detects_git_presence() {
        let dir = setup(&[".git/HEAD"]);
        assert!(detect_git(dir.path()));
    }

    #[test]
    fn detects_no_git_on_empty_dir() {
        let dir = TempDir::new().unwrap();
        assert!(!detect_git(dir.path()));
    }

    #[test]
    fn detects_docker_from_dockerfile() {
        let dir = setup(&["Dockerfile"]);
        assert!(detect_docker(dir.path(), &no_signals(), &no_skills()));
    }

    #[test]
    fn detects_docker_from_compose() {
        let dir = setup(&["docker-compose.yml"]);
        assert!(detect_docker(dir.path(), &no_signals(), &no_skills()));
    }

    #[test]
    fn detects_ci_from_github_workflows_dir() {
        let dir = setup(&[".github/workflows/ci.yml"]);
        assert!(detect_ci(dir.path(), &no_signals(), &no_skills()));
    }

    #[test]
    fn detects_ci_from_circleci() {
        let dir = setup(&[".circleci/config.yml"]);
        assert!(detect_ci(dir.path(), &no_signals(), &no_skills()));
    }

    #[test]
    fn detects_monorepo_from_pnpm_workspace() {
        let dir = setup(&["pnpm-workspace.yaml"]);
        assert!(detect_monorepo(
            dir.path(),
            &no_signals(),
            &empty_folders(),
            &no_skills()
        ));
    }

    #[test]
    fn top_folders_excludes_noise() {
        let fm: FolderMap = ["src", "components", "node_modules", ".git", "dist"]
            .iter()
            .map(|&n| (n.to_string(), vec![]))
            .collect();

        let folders = collect_top_folders(&fm);
        assert!(folders.contains(&"src".to_string()));
        assert!(folders.contains(&"components".to_string()));
        assert!(!folders.contains(&"node_modules".to_string()));
        assert!(!folders.contains(&"dist".to_string()));
        assert!(!folders.contains(&".git".to_string()));
    }

    #[test]
    fn project_meta_extracts_name_from_package_json() {
        use crate::types::SignalKind;
        use std::path::PathBuf;

        let signal = SignalFile {
            kind: SignalKind::PackageJson,
            path: PathBuf::from("package.json"),
            content: r#"{"name":"my-dashboard","description":"IoT monitoring tool"}"#.to_string(),
        };

        let meta = build_project_meta(&[signal], None);
        assert_eq!(meta.name, "my-dashboard");
        assert_eq!(meta.description.as_deref(), Some("IoT monitoring tool"));
    }

    #[test]
    fn project_meta_falls_back_to_cargo_toml() {
        use crate::types::SignalKind;
        use std::path::PathBuf;

        let signal = SignalFile {
            kind: SignalKind::CargoToml,
            path: PathBuf::from("Cargo.toml"),
            content: "[package]\nname = \"repo-intel\"\ndescription = \"A CLI tool\"\nversion = \"0.1.0\"".to_string(),
        };

        let meta = build_project_meta(&[signal], None);
        assert_eq!(meta.name, "repo-intel");
        assert_eq!(meta.description.as_deref(), Some("A CLI tool"));
    }
}
