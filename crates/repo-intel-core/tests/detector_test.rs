/// Integration tests: run the full scan → detect pipeline on fixture repos
/// and assert that the correct skills and stack fields are inferred.
use std::path::Path;

use repo_intel_core::{config::Config, detector::detect, scanner::scan};

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

/// Run scan + detect on a fixture. Panics on error.
fn detect_fixture(name: &str) -> repo_intel_core::types::StackResult {
    let root = fixture(name);
    let scan = scan(&root, &default_cfg()).unwrap();
    detect(&scan).unwrap()
}

/// Returns true if any skill name contains `needle` at confidence ≥ threshold.
fn has_skill(stack: &repo_intel_core::types::StackResult, needle: &str, min_conf: f32) -> bool {
    stack
        .skills
        .iter()
        .any(|s| s.name.to_lowercase().contains(&needle.to_lowercase()) && s.confidence >= min_conf)
}

// ── nextjs-basic ──────────────────────────────────────────────────────────────

#[test]
fn detect_nextjs_language_is_typescript() {
    let stack = detect_fixture("nextjs-basic");
    assert_eq!(stack.language, "TypeScript");
}

#[test]
fn detect_nextjs_framework_is_nextjs() {
    let stack = detect_fixture("nextjs-basic");
    assert_eq!(stack.framework.as_deref(), Some("Next.js"));
}

#[test]
fn detect_nextjs_styling_is_tailwind() {
    let stack = detect_fixture("nextjs-basic");
    assert_eq!(stack.styling.as_deref(), Some("Tailwind CSS"));
}

#[test]
fn detect_nextjs_has_ssr_skill() {
    let stack = detect_fixture("nextjs-basic");
    assert!(has_skill(&stack, "SSR", 0.90), "expected SSR skill ≥ 0.90");
}

#[test]
fn detect_nextjs_has_react_skill() {
    let stack = detect_fixture("nextjs-basic");
    assert!(
        has_skill(&stack, "React", 0.90),
        "expected React skill ≥ 0.90"
    );
}

#[test]
fn detect_nextjs_has_prisma_skill() {
    let stack = detect_fixture("nextjs-basic");
    assert!(
        has_skill(&stack, "Prisma", 0.90),
        "expected Prisma skill ≥ 0.90"
    );
}

#[test]
fn detect_nextjs_has_testing_skill() {
    let stack = detect_fixture("nextjs-basic");
    assert!(
        has_skill(&stack, "Testing", 0.80),
        "expected Testing skill ≥ 0.80"
    );
}

#[test]
fn detect_nextjs_has_storybook_skill() {
    let stack = detect_fixture("nextjs-basic");
    assert!(
        has_skill(&stack, "Storybook", 0.50),
        "expected Storybook skill ≥ 0.50"
    );
}

#[test]
fn detect_nextjs_skills_all_above_threshold() {
    let stack = detect_fixture("nextjs-basic");
    for skill in &stack.skills {
        assert!(
            skill.confidence >= 0.50,
            "skill '{}' has confidence {:.2} below threshold",
            skill.name,
            skill.confidence
        );
    }
}

#[test]
fn detect_nextjs_skills_sorted_descending() {
    let stack = detect_fixture("nextjs-basic");
    let confs: Vec<f32> = stack.skills.iter().map(|s| s.confidence).collect();
    let mut sorted = confs.clone();
    sorted.sort_by(|a, b| b.partial_cmp(a).unwrap());
    assert_eq!(
        confs, sorted,
        "skills should be sorted by confidence descending"
    );
}

// ── react-spa ─────────────────────────────────────────────────────────────────

#[test]
fn detect_react_spa_framework_is_not_ssr() {
    let stack = detect_fixture("react-spa");
    // Plain React SPA — should NOT detect SSR
    assert!(
        !has_skill(&stack, "SSR", 0.90),
        "react-spa should not have SSR skill"
    );
}

#[test]
fn detect_react_spa_has_react_skill() {
    let stack = detect_fixture("react-spa");
    assert!(
        has_skill(&stack, "React", 0.90),
        "expected React skill ≥ 0.90"
    );
}

#[test]
fn detect_react_spa_has_vite_skill() {
    let stack = detect_fixture("react-spa");
    assert!(
        has_skill(&stack, "Vite", 0.90),
        "expected Vite skill ≥ 0.90"
    );
}

#[test]
fn detect_react_spa_has_zustand_skill() {
    let stack = detect_fixture("react-spa");
    assert!(
        has_skill(&stack, "Zustand", 0.90),
        "expected Zustand skill ≥ 0.90"
    );
}

#[test]
fn detect_react_spa_has_tanstack_query_skill() {
    let stack = detect_fixture("react-spa");
    assert!(
        has_skill(&stack, "Server State", 0.90),
        "expected Server State skill ≥ 0.90"
    );
}

// ── node-api ──────────────────────────────────────────────────────────────────

#[test]
fn detect_node_api_has_fastify_skill() {
    let stack = detect_fixture("node-api");
    assert!(
        has_skill(&stack, "Fastify", 0.90),
        "expected Fastify skill ≥ 0.90"
    );
}

#[test]
fn detect_node_api_has_prisma_skill() {
    let stack = detect_fixture("node-api");
    assert!(
        has_skill(&stack, "Prisma", 0.90),
        "expected Prisma skill ≥ 0.90"
    );
}

#[test]
fn detect_node_api_has_jest_testing() {
    let stack = detect_fixture("node-api");
    assert!(
        has_skill(&stack, "Jest", 0.90),
        "expected Jest skill ≥ 0.90"
    );
}

#[test]
fn detect_node_api_has_controller_pattern() {
    let stack = detect_fixture("node-api");
    assert!(
        has_skill(&stack, "Controller", 0.80),
        "expected Controller Pattern skill ≥ 0.80"
    );
}

#[test]
fn detect_node_api_has_docker_skill() {
    let stack = detect_fixture("node-api");
    assert!(
        has_skill(&stack, "Docker", 0.80),
        "expected Docker skill ≥ 0.80"
    );
}

// ── rust-axum ─────────────────────────────────────────────────────────────────

#[test]
fn detect_rust_axum_language_is_rust() {
    let stack = detect_fixture("rust-axum");
    assert_eq!(stack.language, "Rust");
}

#[test]
fn detect_rust_axum_framework_is_axum() {
    let stack = detect_fixture("rust-axum");
    assert_eq!(
        stack.framework.as_deref(),
        Some("Axum"),
        "expected framework = Axum, got {:?}",
        stack.framework
    );
}

#[test]
fn detect_rust_axum_has_sqlx_skill() {
    let stack = detect_fixture("rust-axum");
    assert!(
        has_skill(&stack, "SQLx", 0.90),
        "expected SQLx skill ≥ 0.90"
    );
}

#[test]
fn detect_rust_axum_has_tokio_skill() {
    let stack = detect_fixture("rust-axum");
    assert!(
        has_skill(&stack, "Tokio", 0.90),
        "expected Tokio skill ≥ 0.90"
    );
}

#[test]
fn detect_rust_axum_has_database_skill() {
    let stack = detect_fixture("rust-axum");
    assert!(
        has_skill(&stack, "Database", 0.90),
        "expected Database skill ≥ 0.90"
    );
}

// ── monorepo ──────────────────────────────────────────────────────────────────

#[test]
fn detect_monorepo_has_nextjs_from_workspace_pkg() {
    let stack = detect_fixture("monorepo");
    // web package has next → should be detected even in monorepo
    assert!(
        has_skill(&stack, "Next.js", 0.90),
        "expected Next.js skill from workspace pkg"
    );
}

#[test]
fn detect_monorepo_has_fastify_from_api_pkg() {
    let stack = detect_fixture("monorepo");
    assert!(
        has_skill(&stack, "Fastify", 0.90),
        "expected Fastify skill from api pkg"
    );
}

// ── empty-repo ────────────────────────────────────────────────────────────────

#[test]
fn detect_empty_repo_language_is_unknown() {
    let stack = detect_fixture("empty-repo");
    assert_eq!(stack.language, "Unknown");
}

#[test]
fn detect_empty_repo_no_framework() {
    let stack = detect_fixture("empty-repo");
    assert!(stack.framework.is_none());
}

#[test]
fn detect_empty_repo_skills_all_above_threshold() {
    // Empty repo may still emit some pattern-based skills — all must be ≥ 0.50
    let stack = detect_fixture("empty-repo");
    for skill in &stack.skills {
        assert!(
            skill.confidence >= 0.50,
            "skill below threshold: {:?}",
            skill
        );
    }
}
