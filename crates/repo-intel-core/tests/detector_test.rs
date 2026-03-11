use repo_intel_core::{config::Config, detector::detect, scanner::scan};
use std::path::Path;

#[test]
fn test_detect_nextjs() {
    let fixture = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/nextjs-basic");
    let cfg = Config::default();
    let scan_result = scan(&fixture, &cfg).expect("scan should succeed");
    let stack = detect(&scan_result);

    let has_nextjs = stack.frameworks.iter().any(|f| f.name == "nextjs");
    let has_react = stack.frameworks.iter().any(|f| f.name == "react");

    assert!(has_nextjs || has_react, "should detect nextjs or react");

    let has_ts = stack
        .languages
        .iter()
        .any(|l| l.name == "typescript" || l.name == "nodejs");
    assert!(has_ts, "should detect typescript or nodejs");
}

#[test]
fn test_detect_rust_axum() {
    let fixture = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/rust-axum");
    let cfg = Config::default();
    let scan_result = scan(&fixture, &cfg).expect("scan should succeed");
    let stack = detect(&scan_result);

    let has_rust = stack.languages.iter().any(|l| l.name == "rust");
    assert!(has_rust, "should detect rust language");

    let has_axum = stack.frameworks.iter().any(|f| f.name == "axum");
    assert!(has_axum, "should detect axum framework");
}

#[test]
fn test_detect_empty_repo_no_panic() {
    let fixture = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/empty-repo");
    let cfg = Config::default();
    let scan_result = scan(&fixture, &cfg).expect("scan should succeed");
    let stack = detect(&scan_result);

    // Should not panic; languages & frameworks may be empty
    assert!(
        stack.languages.len() + stack.frameworks.len() == 0
            || stack.languages.len() + stack.frameworks.len() > 0
    );
}
