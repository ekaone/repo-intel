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
    // VCS — skipped from recursive walk, but probed separately in `probe_hidden_signals`
    ".git",
    // Test coverage
    "coverage",
    // Test directories — skill detection comes from file patterns, not folder names
    "tests",
    "test",
    "__tests__",
    "spec",
    // CI internals — `workflows` appears as a detached folder name when
    // `.github` is skipped by the walk but its children are still visited
    "workflows",
    // Documentation / examples — not application source
    "docs",
    "doc",
    "examples",
    "example",
    "demo",
    "demos",
    "bench",
    "benches",
    "benchmark",
    "benchmarks",
    // Bundler / framework internals
    "turbopack",
    "webpack",
    ".webpack",
    // Temporary / generated
    "tmp",
    "temp",
    ".tmp",
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

/// Signals derived from hidden directories that are excluded from the main walk.
/// These cannot be detected via `folder_map` because hidden dirs are in `SKIP_DIRS`.
#[derive(Debug, Default, PartialEq)]
pub struct HiddenSignals {
    pub has_git: bool,
    pub has_ci: bool,       // .github/workflows/ exists
    pub has_github: bool,   // .github/ exists (monorepo / community files signal)
    pub has_packages: bool, // .packages/ exists
}

/// Probe known hidden directories at `root` without descending into them.
///
/// This is a shallow existence check only — no recursive walk.
/// Called separately from `walk()` so callers get both the folder map
/// and the hidden signals in one pass over the root.
pub fn probe_hidden_signals(root: &Path) -> HiddenSignals {
    let mut signals = HiddenSignals::default();

    // .git/ — version control present
    if root.join(".git").is_dir() {
        signals.has_git = true;
    }

    // .github/ — GitHub-specific config present
    if root.join(".github").is_dir() {
        signals.has_github = true;

        // .github/workflows/ — CI pipeline present
        if root.join(".github").join("workflows").is_dir() {
            signals.has_ci = true;
        }
    }

    // .packages/ — used by some monorepo tooling
    if root.join(".packages").is_dir() {
        signals.has_packages = true;
    }

    signals
}

/// Walk `root` up to `depth_cap` levels deep.
///
/// Returns:
/// - `FolderMap`   — every non-skipped directory name → direct child filenames
/// - `Vec<String>` — collected file pattern tokens (extensions + notable names)
///
/// Hidden directory signals (`has_git`, `has_ci`, etc.) are NOT in the folder map.
/// Call `probe_hidden_signals(root)` separately to get those.
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
            let name = entry.file_name().to_string_lossy().to_string();

            // Filter noisy directory name patterns that slip past SKIP_DIRS:
            //   - Numbered dirs:              "01-components", "02-hooks"
            //   - Next.js route interception: "(.)page", "(..)slot", "(auth)"
            //   - Single-char noise:          "e", "x"
            if is_noisy_dir_name(&name) {
                continue;
            }

            // Register every directory in the map (value populated below).
            folder_map.entry(name).or_default();
        } else {
            // Add this filename to the parent folder's child list.
            if let Some(parent) = path.parent() {
                let parent_name = parent
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_default();

                // Don't add children to a noisy parent that was skipped above.
                if !is_noisy_dir_name(&parent_name) {
                    let file_name = entry.file_name().to_string_lossy().to_string();
                    folder_map
                        .entry(parent_name)
                        .or_default()
                        .push(file_name.clone());
                }
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
    entry.file_type().is_dir() && SKIP_DIRS.contains(&entry.file_name().to_str().unwrap_or(""))
}

/// Returns `true` if a directory name is noisy and should be excluded from
/// the folder map even though it passed the `SKIP_DIRS` check.
///
/// Catches patterns that can't be enumerated statically:
/// - Numbered dirs:              "01-components", "2-api", "03_utils"
/// - Next.js route interception: "(.)page", "(..)slot", "(auth)"
/// - Next.js dynamic segments:   "[slug]", "[id]", "[...params]"
/// - Single-char dirs:           "e", "x", "a"
fn is_noisy_dir_name(name: &str) -> bool {
    // Single character — never a meaningful application folder
    if name.len() <= 1 {
        return true;
    }

    // Starts with a digit — numbered fixture / tutorial dirs
    if name.starts_with(|c: char| c.is_ascii_digit()) {
        return true;
    }

    // Starts with '(' — Next.js route groups / interception patterns
    // e.g. "(auth)", "(.)photo", "(..)feed"
    if name.starts_with('(') {
        return true;
    }

    // Starts with '[' — Next.js dynamic route segments
    // e.g. "[slug]", "[id]", "[...params]"
    if name.starts_with('[') {
        return true;
    }

    false
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
        "prisma",
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

    // ── walk() — existing tests ───────────────────────────────────────────────

    #[test]
    fn skips_node_modules() {
        let dir = TempDir::new().unwrap();
        make_tree(dir.path(), &["src/index.ts", "node_modules/react/index.js"]);

        let (folder_map, patterns) = walk(dir.path(), DEPTH_CAP);

        assert!(folder_map.contains_key("src"));
        assert!(!folder_map.contains_key("node_modules"));
        assert!(patterns.contains(&".ts".to_string()));
        assert!(!patterns.contains(&".js".to_string()));
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
        make_tree(
            dir.path(),
            &["components/Button.tsx", "components/Input.tsx"],
        );

        let (folder_map, _) = walk(dir.path(), DEPTH_CAP);

        let children = folder_map.get("components").unwrap();
        assert!(children.contains(&"Button.tsx".to_string()));
        assert!(children.contains(&"Input.tsx".to_string()));
    }

    // ── walk() — folder pollution tests ──────────────────────────────────────

    #[test]
    fn skips_docs_and_examples() {
        let dir = TempDir::new().unwrap();
        make_tree(
            dir.path(),
            &[
                "src/index.ts",
                "docs/guide.md",
                "examples/basic/index.ts",
                "bench/run.ts",
                "tests/unit/foo.test.ts",
                "__tests__/bar.test.ts",
                "workflows/ci.yml",
            ],
        );

        let (folder_map, _) = walk(dir.path(), DEPTH_CAP);

        assert!(folder_map.contains_key("src"));
        assert!(!folder_map.contains_key("docs"));
        assert!(!folder_map.contains_key("examples"));
        assert!(!folder_map.contains_key("bench"));
        assert!(!folder_map.contains_key("tests"));
        assert!(!folder_map.contains_key("__tests__"));
        assert!(!folder_map.contains_key("workflows"));
    }

    #[test]
    fn skips_numbered_dirs() {
        let dir = TempDir::new().unwrap();
        make_tree(
            dir.path(),
            &[
                "src/index.ts",
                "01-components/Button.tsx",
                "02-hooks/useAuth.ts",
                "03_utils/format.ts",
            ],
        );

        let (folder_map, _) = walk(dir.path(), DEPTH_CAP);

        assert!(folder_map.contains_key("src"));
        assert!(!folder_map.contains_key("01-components"));
        assert!(!folder_map.contains_key("02-hooks"));
        assert!(!folder_map.contains_key("03_utils"));
    }

    #[test]
    fn skips_nextjs_route_interception_dirs() {
        let dir = TempDir::new().unwrap();
        make_tree(
            dir.path(),
            &[
                "src/index.ts",
                "app/(auth)/login/page.tsx",
                "app/(.)photo/page.tsx",
                "app/(..)feed/page.tsx",
            ],
        );

        let (folder_map, _) = walk(dir.path(), DEPTH_CAP);

        assert!(folder_map.contains_key("src"));
        assert!(!folder_map.contains_key("(auth)"));
        assert!(!folder_map.contains_key("(.)photo"));
        assert!(!folder_map.contains_key("(..)feed"));
    }

    #[test]
    fn skips_single_char_dirs() {
        let dir = TempDir::new().unwrap();
        make_tree(dir.path(), &["src/index.ts", "e/index.ts", "x/main.ts"]);

        let (folder_map, _) = walk(dir.path(), DEPTH_CAP);

        assert!(folder_map.contains_key("src"));
        assert!(!folder_map.contains_key("e"));
        assert!(!folder_map.contains_key("x"));
    }

    #[test]
    fn legitimate_folders_not_filtered() {
        let dir = TempDir::new().unwrap();
        make_tree(
            dir.path(),
            &[
                "src/index.ts",
                "components/Button.tsx",
                "hooks/useAuth.ts",
                "services/api.ts",
                "lib/utils.ts",
                "app/page.tsx",
                "packages/ui/index.ts",
            ],
        );

        let (folder_map, _) = walk(dir.path(), DEPTH_CAP);

        for expected in &[
            "src",
            "components",
            "hooks",
            "services",
            "lib",
            "app",
            "packages",
        ] {
            assert!(
                folder_map.contains_key(*expected),
                "expected '{}' in folder_map",
                expected
            );
        }
    }

    // ── probe_hidden_signals() tests ──────────────────────────────────────────

    #[test]
    fn detects_git_dir() {
        let dir = TempDir::new().unwrap();
        fs::create_dir(dir.path().join(".git")).unwrap();

        let signals = probe_hidden_signals(dir.path());

        assert!(signals.has_git);
        assert!(!signals.has_ci);
        assert!(!signals.has_github);
    }

    #[test]
    fn detects_github_workflows() {
        let dir = TempDir::new().unwrap();
        make_tree(dir.path(), &[".github/workflows/ci.yml"]);

        let signals = probe_hidden_signals(dir.path());

        assert!(signals.has_github);
        assert!(signals.has_ci);
        assert!(!signals.has_git);
    }

    #[test]
    fn github_without_workflows_is_not_ci() {
        let dir = TempDir::new().unwrap();
        make_tree(dir.path(), &[".github/CODEOWNERS"]);

        let signals = probe_hidden_signals(dir.path());

        assert!(signals.has_github);
        assert!(!signals.has_ci);
        assert!(!signals.has_git);
    }

    #[test]
    fn no_hidden_dirs_returns_all_false() {
        let dir = TempDir::new().unwrap();
        make_tree(dir.path(), &["src/index.ts"]);

        let signals = probe_hidden_signals(dir.path());

        assert_eq!(signals, HiddenSignals::default());
    }

    #[test]
    fn git_dir_not_in_folder_map() {
        let dir = TempDir::new().unwrap();
        fs::create_dir(dir.path().join(".git")).unwrap();
        make_tree(dir.path(), &["src/index.ts"]);

        let (folder_map, _) = walk(dir.path(), DEPTH_CAP);

        assert!(!folder_map.contains_key(".git"));
        assert!(folder_map.contains_key("src"));
    }

    // ── is_noisy_dir_name() unit tests ────────────────────────────────────────

    #[test]
    fn noisy_name_single_char() {
        assert!(is_noisy_dir_name("e"));
        assert!(is_noisy_dir_name("x"));
        assert!(is_noisy_dir_name("a"));
    }

    #[test]
    fn noisy_name_numbered() {
        assert!(is_noisy_dir_name("01-components"));
        assert!(is_noisy_dir_name("2-api"));
        assert!(is_noisy_dir_name("03_utils"));
        assert!(is_noisy_dir_name("10-advanced"));
    }

    #[test]
    fn noisy_name_nextjs_route_groups() {
        assert!(is_noisy_dir_name("(auth)"));
        assert!(is_noisy_dir_name("(.)photo"));
        assert!(is_noisy_dir_name("(..)feed"));
        assert!(is_noisy_dir_name("(marketing)"));
    }

    #[test]
    fn noisy_name_nextjs_dynamic_segments() {
        assert!(is_noisy_dir_name("[slug]"));
        assert!(is_noisy_dir_name("[id]"));
        assert!(is_noisy_dir_name("[...params]"));
        assert!(is_noisy_dir_name("[userId]"));
    }

    #[test]
    fn clean_names_not_noisy() {
        assert!(!is_noisy_dir_name("src"));
        assert!(!is_noisy_dir_name("components"));
        assert!(!is_noisy_dir_name("hooks"));
        assert!(!is_noisy_dir_name("services"));
        assert!(!is_noisy_dir_name("lib"));
        assert!(!is_noisy_dir_name("app"));
        assert!(!is_noisy_dir_name("packages"));
        assert!(!is_noisy_dir_name("api"));
        assert!(!is_noisy_dir_name("ui"));
    }
}
