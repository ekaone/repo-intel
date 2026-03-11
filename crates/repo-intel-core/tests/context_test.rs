use repo_intel_core::{config::Config, context::build, detector::detect, scanner::scan};
use std::path::Path;

#[test]
fn test_full_pipeline_produces_valid_json_nextjs() {
    let fixture = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/nextjs-basic");
    let cfg = Config::default();

    let scan_result = scan(&fixture, &cfg).expect("scan should succeed");
    let stack = detect(&scan_result);
    let context = build(&stack);
    let json = repo_intel_core::context::serializer::to_json(&context)
        .expect("serialization should succeed");

    // Must be valid JSON
    let parsed: serde_json::Value = serde_json::from_str(&json).expect("output must be valid JSON");

    assert!(
        parsed.get("stack").is_some(),
        "context.json must have 'stack'"
    );
    assert!(
        parsed.get("agent_roles").is_some(),
        "context.json must have 'agent_roles'"
    );
    assert!(
        parsed.get("schema_version").is_some(),
        "context.json must have 'schema_version'"
    );
}

#[test]
fn test_full_pipeline_empty_repo() {
    let fixture = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/empty-repo");
    let cfg = Config::default();

    let scan_result = scan(&fixture, &cfg).expect("scan should succeed");
    let stack = detect(&scan_result);
    let context = build(&stack);
    let json = repo_intel_core::context::serializer::to_json(&context)
        .expect("serialization should succeed");

    let parsed: serde_json::Value = serde_json::from_str(&json).expect("output must be valid JSON");
    assert!(parsed.is_object(), "output should be a JSON object");
}
