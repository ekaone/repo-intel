use repo_intel_core::{config::Config, scanner::scan};
use std::path::Path;

#[test]
fn test_scan_nextjs_basic() {
    let fixture = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/nextjs-basic");
    let cfg = Config::default();
    let result = scan(&fixture, &cfg).expect("scan should succeed");

    assert!(
        result.signal_files.contains_key("package.json"),
        "nextjs-basic should have a package.json signal"
    );
    assert!(
        !result.folder_map.is_empty(),
        "folder_map should not be empty"
    );
}

#[test]
fn test_scan_rust_axum() {
    let fixture = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/rust-axum");
    let cfg = Config::default();
    let result = scan(&fixture, &cfg).expect("scan should succeed");

    assert!(
        result.signal_files.contains_key("Cargo.toml"),
        "rust-axum should have a Cargo.toml signal"
    );
}

#[test]
fn test_scan_empty_repo() {
    let fixture = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/empty-repo");
    let cfg = Config::default();
    let result = scan(&fixture, &cfg).expect("scan should succeed on empty repo");

    assert!(
        result.signal_files.is_empty(),
        "empty-repo should have no signal files"
    );
}

#[test]
fn test_scan_monorepo() {
    let fixture = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/monorepo");
    let cfg = Config::default();
    let result = scan(&fixture, &cfg).expect("scan should succeed");

    assert!(
        result.signal_files.contains_key("package.json"),
        "monorepo should have root package.json"
    );
}
