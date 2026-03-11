/// Integration tests: run the full pipeline (scan → detect → build context)
/// on fixture repos and assert:
///   - `context.json` shape is valid and matches the Rust↔JS contract
///   - `agent_roles` are populated correctly
///   - `has_docker`, `has_ci`, `has_monorepo` flags are accurate
///   - README excerpt is included when README exists
use std::path::Path;

use repo_intel_core::{config::Config, context::serializer, run_pipeline};

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

/// Run the full pipeline on a fixture. Panics on error.
fn pipeline(name: &str) -> repo_intel_core::types::RepoContext {
    run_pipeline(&fixture(name), &default_cfg()).unwrap()
}

// ── contract: required fields ─────────────────────────────────────────────────

#[test]
fn context_json_has_all_required_fields() {
    let ctx = pipeline("nextjs-basic");
    let json = serializer::to_json(&ctx).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

    let required_top = [
        "version",
        "scanned_at",
        "root",
        "project",
        "stack",
        "architecture",
        "agent_roles",
    ];
    for field in &required_top {
        assert!(
            parsed.get(field).is_some(),
            "missing top-level field: {field}"
        );
    }

    let required_stack = [
        "language",
        "framework",
        "styling",
        "testing",
        "database",
        "runtime",
    ];
    for field in &required_stack {
        assert!(
            parsed["stack"].get(field).is_some(),
            "missing stack field: {field}"
        );
    }

    let required_arch = ["folders", "has_monorepo", "has_docker", "has_ci", "has_git"];
    for field in &required_arch {
        assert!(
            parsed["architecture"].get(field).is_some(),
            "missing arch field: {field}"
        );
    }
}

#[test]
fn context_json_is_valid_parseable_json() {
    for fixture_name in &[
        "nextjs-basic",
        "react-spa",
        "node-api",
        "rust-axum",
        "monorepo",
        "empty-repo",
    ] {
        let ctx = pipeline(fixture_name);
        let json = serializer::to_json(&ctx).unwrap();
        let result = serde_json::from_str::<serde_json::Value>(&json);
        assert!(
            result.is_ok(),
            "fixture '{fixture_name}' produced invalid JSON"
        );
    }
}

#[test]
fn context_version_matches_cargo_pkg_version() {
    let ctx = pipeline("nextjs-basic");
    assert_eq!(ctx.version, env!("CARGO_PKG_VERSION"));
}

#[test]
fn context_scanned_at_looks_like_iso8601() {
    let ctx = pipeline("nextjs-basic");
    assert!(
        ctx.scanned_at.ends_with('Z'),
        "scanned_at should end with Z"
    );
    assert!(ctx.scanned_at.contains('T'), "scanned_at should contain T");
    assert_eq!(ctx.scanned_at.len(), 20, "scanned_at should be 20 chars");
}

// ── nextjs-basic: roles + stack ───────────────────────────────────────────────

#[test]
fn context_nextjs_has_fullstack_role() {
    let ctx = pipeline("nextjs-basic");
    assert!(
        ctx.agent_roles.contains(&"Fullstack Engineer".to_string()),
        "expected Fullstack Engineer role, got: {:?}",
        ctx.agent_roles
    );
}

#[test]
fn context_nextjs_has_qa_role() {
    let ctx = pipeline("nextjs-basic");
    assert!(
        ctx.agent_roles
            .contains(&"QA & Testing Engineer".to_string()),
        "expected QA & Testing Engineer role, got: {:?}",
        ctx.agent_roles
    );
}

#[test]
fn context_nextjs_has_database_role() {
    let ctx = pipeline("nextjs-basic");
    assert!(
        ctx.agent_roles.contains(&"Database Engineer".to_string()),
        "expected Database Engineer role, got: {:?}",
        ctx.agent_roles
    );
}

#[test]
fn context_nextjs_no_frontend_role_when_fullstack_present() {
    let ctx = pipeline("nextjs-basic");
    assert!(
        !ctx.agent_roles.contains(&"Frontend Engineer".to_string()),
        "should not have Frontend Engineer when Fullstack is detected"
    );
}

#[test]
fn context_nextjs_project_name_is_set() {
    let ctx = pipeline("nextjs-basic");
    assert_eq!(ctx.project.name, "nextjs-basic");
}

#[test]
fn context_nextjs_project_description_is_set() {
    let ctx = pipeline("nextjs-basic");
    assert!(
        ctx.project.description.is_some(),
        "expected project description from package.json"
    );
}

#[test]
fn context_nextjs_folders_contains_components() {
    let ctx = pipeline("nextjs-basic");
    assert!(
        ctx.architecture.folders.contains(&"components".to_string()),
        "expected 'components' in folders"
    );
}

// ── node-api: docker flag ─────────────────────────────────────────────────────

#[test]
fn context_node_api_has_docker_true() {
    let ctx = pipeline("node-api");
    assert!(
        ctx.architecture.has_docker,
        "expected has_docker = true for node-api fixture"
    );
}

#[test]
fn context_node_api_has_backend_role() {
    let ctx = pipeline("node-api");
    assert!(
        ctx.agent_roles
            .contains(&"Backend API Engineer".to_string()),
        "expected Backend API Engineer role, got: {:?}",
        ctx.agent_roles
    );
}

// ── rust-axum: language + roles ───────────────────────────────────────────────

#[test]
fn context_rust_axum_language_is_rust() {
    let ctx = pipeline("rust-axum");
    assert_eq!(ctx.stack.language, "Rust");
}

#[test]
fn context_rust_axum_has_backend_role() {
    let ctx = pipeline("rust-axum");
    assert!(
        ctx.agent_roles
            .contains(&"Backend API Engineer".to_string()),
        "expected Backend API Engineer, got: {:?}",
        ctx.agent_roles
    );
}

#[test]
fn context_rust_axum_has_database_role() {
    let ctx = pipeline("rust-axum");
    assert!(
        ctx.agent_roles.contains(&"Database Engineer".to_string()),
        "expected Database Engineer, got: {:?}",
        ctx.agent_roles
    );
}

#[test]
fn context_rust_axum_project_name_from_cargo() {
    let ctx = pipeline("rust-axum");
    assert_eq!(ctx.project.name, "rust-axum");
}

// ── monorepo: monorepo flag + roles ───────────────────────────────────────────

#[test]
fn context_monorepo_has_monorepo_true() {
    let ctx = pipeline("monorepo");
    assert!(
        ctx.architecture.has_monorepo,
        "expected has_monorepo = true"
    );
}

#[test]
fn context_monorepo_has_platform_engineer_role() {
    let ctx = pipeline("monorepo");
    assert!(
        ctx.agent_roles
            .contains(&"Platform / Monorepo Engineer".to_string()),
        "expected Platform / Monorepo Engineer, got: {:?}",
        ctx.agent_roles
    );
}

// ── empty-repo: graceful degradation ─────────────────────────────────────────

#[test]
fn context_empty_repo_pipeline_does_not_panic() {
    let result = run_pipeline(&fixture("empty-repo"), &default_cfg());
    assert!(result.is_ok(), "pipeline on empty repo should not error");
}

#[test]
fn context_empty_repo_agent_roles_is_empty() {
    let ctx = pipeline("empty-repo");
    assert!(
        ctx.agent_roles.is_empty(),
        "empty repo should have no roles"
    );
}

#[test]
fn context_empty_repo_json_is_still_valid() {
    let ctx = pipeline("empty-repo");
    let json = serializer::to_json(&ctx).unwrap();
    let parsed = serde_json::from_str::<serde_json::Value>(&json);
    assert!(
        parsed.is_ok(),
        "empty repo context should still produce valid JSON"
    );
}

// ── serializer: pretty vs compact ────────────────────────────────────────────

#[test]
fn context_pretty_json_contains_newlines() {
    let ctx = pipeline("nextjs-basic");
    let pretty = serializer::to_json_pretty(&ctx).unwrap();
    assert!(pretty.contains('\n'), "pretty JSON should contain newlines");
}

#[test]
fn context_compact_json_has_no_newlines() {
    let ctx = pipeline("nextjs-basic");
    let compact = serializer::to_json(&ctx).unwrap();
    assert!(
        !compact.contains('\n'),
        "compact JSON should have no newlines"
    );
}

// ── convenience methods ───────────────────────────────────────────────────────

#[test]
fn context_primary_skills_all_above_09() {
    let ctx = pipeline("nextjs-basic");
    for skill in ctx.primary_skills() {
        assert!(
            skill.confidence >= 0.90,
            "primary skill '{}' has confidence {:.2}",
            skill.name,
            skill.confidence
        );
    }
}

#[test]
fn context_secondary_skills_between_05_and_09() {
    let ctx = pipeline("nextjs-basic");
    for skill in ctx.secondary_skills() {
        assert!(
            skill.confidence >= 0.50 && skill.confidence < 0.90,
            "secondary skill '{}' has confidence {:.2}",
            skill.name,
            skill.confidence
        );
    }
}
