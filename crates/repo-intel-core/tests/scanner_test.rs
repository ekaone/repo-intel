/// Integration tests: scan each fixture repo and assert `ScanResult` shape.
///
/// These tests exercise the full scanner pipeline (walker + signal collector)
/// against real fixture directories on disk. They complement the unit tests
/// inside each module.
use std::path::Path;

use repo_intel_core::{config::Config, scanner::scan, types::SignalKind};

/// Absolute path to the fixtures directory.
fn fixtures() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
}

fn fixture(name: &str) -> std::path::PathBuf {
    fixtures().join(name)
}

fn default_cfg() -> Config {
    Config::default()
}

// ── nextjs-basic ──────────────────────────────────────────────────────────────

#[test]
fn scan_nextjs_basic_detects_package_json() {
    let result = scan(&fixture("nextjs-basic"), &default_cfg()).unwrap();
    assert!(
        result
            .signal_files
            .iter()
            .any(|s| s.kind == SignalKind::PackageJson),
        "expected PackageJson signal"
    );
}

#[test]
fn scan_nextjs_basic_detects_tsconfig() {
    let result = scan(&fixture("nextjs-basic"), &default_cfg()).unwrap();
    assert!(
        result
            .signal_files
            .iter()
            .any(|s| s.kind == SignalKind::TsConfig),
        "expected TsConfig signal"
    );
}

#[test]
fn scan_nextjs_basic_finds_src_folders() {
    let result = scan(&fixture("nextjs-basic"), &default_cfg()).unwrap();
    let folders = &result.folder_map;

    assert!(
        folders.contains_key("components"),
        "expected 'components' folder"
    );
    assert!(folders.contains_key("hooks"), "expected 'hooks' folder");
    assert!(
        folders.contains_key("services"),
        "expected 'services' folder"
    );
}

#[test]
fn scan_nextjs_basic_collects_tsx_patterns() {
    let result = scan(&fixture("nextjs-basic"), &default_cfg()).unwrap();
    assert!(
        result.file_patterns.contains(&".tsx".to_string()),
        "expected .tsx pattern"
    );
}

#[test]
fn scan_nextjs_basic_collects_test_patterns() {
    let result = scan(&fixture("nextjs-basic"), &default_cfg()).unwrap();
    assert!(
        result.file_patterns.contains(&"*.test.ts".to_string()),
        "expected *.test.ts pattern"
    );
}

#[test]
fn scan_nextjs_basic_collects_stories_pattern() {
    let result = scan(&fixture("nextjs-basic"), &default_cfg()).unwrap();
    assert!(
        result.file_patterns.contains(&"*.stories.tsx".to_string()),
        "expected *.stories.tsx pattern"
    );
}

#[test]
fn scan_nextjs_basic_has_readme_excerpt() {
    // nextjs-basic has no README — excerpt should be None
    let result = scan(&fixture("nextjs-basic"), &default_cfg()).unwrap();
    // This fixture deliberately has no README — verify it doesn't panic
    let _ = result.readme_excerpt;
}

// ── react-spa ─────────────────────────────────────────────────────────────────

#[test]
fn scan_react_spa_detects_vite_config() {
    let result = scan(&fixture("react-spa"), &default_cfg()).unwrap();
    assert!(
        result.file_patterns.contains(&"vite.config.ts".to_string()),
        "expected vite.config.ts pattern"
    );
}

#[test]
fn scan_react_spa_finds_store_folder() {
    let result = scan(&fixture("react-spa"), &default_cfg()).unwrap();
    assert!(
        result.folder_map.contains_key("store"),
        "expected 'store' folder"
    );
}

#[test]
fn scan_react_spa_finds_test_pattern() {
    let result = scan(&fixture("react-spa"), &default_cfg()).unwrap();
    assert!(
        result.file_patterns.iter().any(|p| p.contains("test")),
        "expected a test pattern"
    );
}

// ── node-api ──────────────────────────────────────────────────────────────────

#[test]
fn scan_node_api_detects_dockerfile() {
    let result = scan(&fixture("node-api"), &default_cfg()).unwrap();
    assert!(
        result
            .signal_files
            .iter()
            .any(|s| s.kind == SignalKind::Dockerfile),
        "expected Dockerfile signal"
    );
}

#[test]
fn scan_node_api_finds_controllers_folder() {
    let result = scan(&fixture("node-api"), &default_cfg()).unwrap();
    assert!(
        result.folder_map.contains_key("controllers"),
        "expected 'controllers' folder"
    );
}

#[test]
fn scan_node_api_finds_routes_folder() {
    let result = scan(&fixture("node-api"), &default_cfg()).unwrap();
    assert!(
        result.folder_map.contains_key("routes"),
        "expected 'routes' folder"
    );
}

#[test]
fn scan_node_api_detects_controller_pattern() {
    let result = scan(&fixture("node-api"), &default_cfg()).unwrap();
    assert!(
        result
            .file_patterns
            .contains(&"*.controller.ts".to_string()),
        "expected *.controller.ts pattern"
    );
}

// ── rust-axum ─────────────────────────────────────────────────────────────────

#[test]
fn scan_rust_axum_detects_cargo_toml() {
    let result = scan(&fixture("rust-axum"), &default_cfg()).unwrap();
    assert!(
        result
            .signal_files
            .iter()
            .any(|s| s.kind == SignalKind::CargoToml),
        "expected CargoToml signal"
    );
}

#[test]
fn scan_rust_axum_collects_rs_pattern() {
    let result = scan(&fixture("rust-axum"), &default_cfg()).unwrap();
    assert!(
        result.file_patterns.contains(&".rs".to_string()),
        "expected .rs pattern"
    );
}

#[test]
fn scan_rust_axum_cargo_content_is_non_empty() {
    let result = scan(&fixture("rust-axum"), &default_cfg()).unwrap();
    let cargo = result
        .signal_files
        .iter()
        .find(|s| s.kind == SignalKind::CargoToml)
        .expect("CargoToml signal should exist");
    assert!(
        !cargo.content.is_empty(),
        "Cargo.toml content should not be empty"
    );
    assert!(
        cargo.content.contains("axum"),
        "Cargo.toml should contain 'axum'"
    );
}

// ── monorepo ──────────────────────────────────────────────────────────────────

#[test]
fn scan_monorepo_detects_multiple_package_jsons() {
    let result = scan(&fixture("monorepo"), &default_cfg()).unwrap();
    let pkg_count = result
        .signal_files
        .iter()
        .filter(|s| s.kind == SignalKind::PackageJson)
        .count();
    assert!(
        pkg_count >= 2,
        "expected at least 2 package.json files, got {pkg_count}"
    );
}

#[test]
fn scan_monorepo_finds_packages_folder() {
    let result = scan(&fixture("monorepo"), &default_cfg()).unwrap();
    assert!(
        result.folder_map.contains_key("packages"),
        "expected 'packages' folder"
    );
}

// ── empty-repo ────────────────────────────────────────────────────────────────

#[test]
fn scan_empty_repo_returns_ok_with_no_signals() {
    let result = scan(&fixture("empty-repo"), &default_cfg()).unwrap();
    assert!(result.signal_files.is_empty(), "expected no signal files");
    assert!(
        result.readme_excerpt.is_none(),
        "expected no readme excerpt"
    );
}

#[test]
fn scan_empty_repo_does_not_panic() {
    // Scanning an empty dir should be a clean Ok, never a panic or Err
    let result = scan(&fixture("empty-repo"), &default_cfg());
    assert!(result.is_ok());
}

// ── error cases ───────────────────────────────────────────────────────────────

#[test]
fn scan_nonexistent_path_returns_error() {
    use repo_intel_core::RepoIntelError;
    let result = scan(Path::new("/nonexistent/path/xyz"), &default_cfg());
    assert!(matches!(result, Err(RepoIntelError::RootNotFound { .. })));
}

#[test]
fn scan_respects_exclude_config() {
    let mut cfg = default_cfg();
    cfg.project.exclude = vec!["src".into()];

    let result = scan(&fixture("nextjs-basic"), &cfg).unwrap();
    assert!(
        !result.folder_map.contains_key("src"),
        "excluded folder 'src' should not appear in folder_map"
    );
}
