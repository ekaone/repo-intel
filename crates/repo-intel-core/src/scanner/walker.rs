use std::collections::HashMap;
use std::path::Path;
use walkdir::WalkDir;

use crate::types::FolderMap;

/// Directories that are never scanned.
/// Ordered roughly by how common they are so short-circuit hits early.
const SKIP_DIRS: &[&str] = &[
    // Dependencies
    "node_modules",
    "vendor",
    ".venv",
    "__pycache__",
    // Build outputs
    "dist",
    "build",
    "target",
    "out",
    ".output",
    "storybook-static",
    // Framework caches
    ".next",
    ".nuxt",
    ".svelte-kit",
    ".angular",
    ".turbo",
    ".cache",
    // VCS
    ".git",
    // Test coverage
    "coverage",
    // repo-intel-specific: fixture repos used in Rust tests should not
    // pollute the folder map when scanning the repo-intel repo itself
    "fixtures",
    // Platform binary packages — these are empty shells with no source
    "repo-intel-linux-x64",
    "repo-intel-darwin-arm64",
    "repo-intel-darwin-x64",
    "repo-intel-win32-x64",
];

/// Maximum directory depth for the MVP.
/// Deep enough to catch monorepo package structures; shallow enough to stay fast.
pub const DEPTH_CAP: usize = 8;

/// Walk `root` up to `depth_cap` levels deep.
///
/// Returns:
/// - `FolderMap`   — every non-skipped directory name → direct child filenames
/// - `Vec<String>` — collected file pattern tokens (extensions + notable names)
///
/// Performance target: < 30ms on a 10,000-file repo.
pub fn walk(root: &Path, depth_cap: usize) -> (FolderMap, Vec<String>) {
    let mut folder_map: FolderMap = HashMap::new();
    let mut file_patterns: Vec<String> = Vec::new();

    for entry in WalkDir::new(root)
        .follow_links(false)
        .max_depth(depth_cap)
        .into_iter()
        .filter_entry(|e| !is_skip_dir(e))
    {
        let entry = match entry {
            Ok(e) => e,
            Err(_) => continue, // permission errors etc. — skip silently
        };

        let path = entry.path();

        if entry.file_type().is_dir() {
            // Register every directory in the map (value populated below).
            let name = entry.file_name().to_string_lossy().to_string();
            folder_map.entry(name).or_default();
        } else {
            // Add this filename to the parent folder's child list.
            if let Some(parent) = path.parent() {
                let parent_name = parent
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_default();

                let file_name = entry.file_name().to_string_lossy().to_string();
                folder_map
                    .entry(parent_name)
                    .or_default()
                    .push(file_name.clone());
            }

            collect_file_pattern(path, &mut file_patterns);
        }
    }

    // Deduplicate patterns — same extension can appear thousands of times.
    file_patterns.sort_unstable();
    file_patterns.dedup();

    (folder_map, file_patterns)
}

/// Returns `true` if `entry` is a directory that should be skipped entirely.
/// Using `filter_entry` means walkdir won't descend into skipped dirs at all.
fn is_skip_dir(entry: &walkdir::DirEntry) -> bool {
    entry.file_type().is_dir()
        && SKIP_DIRS.contains(&entry.file_name().to_str().unwrap_or(""))
}

/// Extract meaningful pattern tokens from a file path.
///
/// Tokens collected:
/// - File extension (e.g. `".ts"`, `".rs"`)
/// - Notable compound names (e.g. `"*.test.ts"`, `"*.stories.tsx"`)
/// - Specific notable filenames verbatim (e.g. `"Dockerfile"`)
fn collect_file_pattern(path: &Path, patterns: &mut Vec<String>) {
    let name = match path.file_name() {
        Some(n) => n.to_string_lossy().to_string(),
        None => return,
    };

    // 1. Primary extension
    if let Some(ext) = path.extension() {
        patterns.push(format!(".{}", ext.to_string_lossy()));
    }

    // 2. Compound patterns — checked by suffix for flexibility
    let compound_suffixes: &[&str] = &[
        ".test.ts",
        ".test.tsx",
        ".test.js",
        ".spec.ts",
        ".spec.tsx",
        ".spec.js",
        ".stories.tsx",
        ".stories.ts",
        ".stories.jsx",
        ".service.ts",
        ".controller.ts",
        ".schema.ts",
        ".model.ts",
        ".graphql",
        ".gql",
    ];

    for suffix in compound_suffixes {
        if name.ends_with(suffix) {
            patterns.push(format!("*{suffix}"));
            break; // one compound match per file is enough
        }
    }

    // 3. Notable standalone filenames
    let notable: &[&str] = &[
        "Dockerfile",
        "docker-compose.yml",
        "docker-compose.yaml",
        ".env",
        ".env.example",
        "jest.config.ts",
        "jest.config.js",
        "vitest.config.ts",
        "vite.config.ts",
        "next.config.js",
        "next.config.ts",
        "tailwind.config.ts",
        "tailwind.config.js",
        "prisma",        // matches prisma/schema.prisma parent dir
        "schema.prisma",
        "go.sum",
    ];

    if notable.contains(&name.as_str()) {
        patterns.push(name);
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn make_tree(root: &Path, files: &[&str]) {
        for f in files {
            let path = root.join(f);
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent).unwrap();
            }
            fs::write(path, "").unwrap();
        }
    }

    #[test]
    fn skips_node_modules() {
        let dir = TempDir::new().unwrap();
        make_tree(dir.path(), &[
            "src/index.ts",
            "node_modules/react/index.js",
        ]);

        let (folder_map, patterns) = walk(dir.path(), DEPTH_CAP);

        assert!(folder_map.contains_key("src"));
        assert!(!folder_map.contains_key("node_modules"));
        assert!(patterns.contains(&".ts".to_string()));
        assert!(!patterns.contains(&".js".to_string())); // node_modules was skipped
    }

    #[test]
    fn collects_compound_patterns() {
        let dir = TempDir::new().unwrap();
        make_tree(dir.path(), &["src/auth.test.ts", "src/ui.stories.tsx"]);

        let (_, patterns) = walk(dir.path(), DEPTH_CAP);

        assert!(patterns.contains(&"*.test.ts".to_string()));
        assert!(patterns.contains(&"*.stories.tsx".to_string()));
    }

    #[test]
    fn folder_map_has_children() {
        let dir = TempDir::new().unwrap();
        make_tree(dir.path(), &["components/Button.tsx", "components/Input.tsx"]);

        let (folder_map, _) = walk(dir.path(), DEPTH_CAP);

        let children = folder_map.get("components").unwrap();
        assert!(children.contains(&"Button.tsx".to_string()));
        assert!(children.contains(&"Input.tsx".to_string()));
    }
}