pub mod deps;
pub mod folders;
pub mod patterns;

use std::collections::HashMap;

use crate::error::Result;
use crate::types::{Skill, ScanResult, StackResult};

/// Confidence thresholds (from PLAN.md)
const INCLUDE_THRESHOLD: f32 = 0.50;   // below this → excluded from output
const PRIMARY_THRESHOLD: f32 = 0.90;   // at or above → "primary skill"

/// Run all 3 detection layers and merge into a `StackResult`.
///
/// Confidence scoring rule (from PLAN.md):
///   Final confidence = max(layer1, layer2, layer3)  ← NOT the sum
///
/// Skills below `INCLUDE_THRESHOLD` are stored internally but excluded from
/// the public `skills` vec. The context builder can still read them via
/// `stack.skills` with a lower threshold if needed.
pub fn detect(scan: &ScanResult) -> Result<StackResult> {
    // ── Run all 3 layers ──────────────────────────────────────────────────────
    let layer1 = deps::detect_from_deps(&scan.signal_files);
    let (layer2, arch_style) = folders::detect_from_folders(&scan.folder_map);
    let layer3 = patterns::detect_from_patterns(&scan.file_patterns);

    // ── Merge: max confidence per skill name ──────────────────────────────────
    let merged = merge_skills(vec![layer1, layer2, layer3]);

    // ── Separate internal markers from real skills ────────────────────────────
    let (internal, mut skills): (Vec<Skill>, Vec<Skill>) =
        merged.into_iter().partition(|s| s.name.starts_with("__"));

    // Apply inclusion threshold — keep below-threshold internally only
    skills.retain(|s| s.confidence >= INCLUDE_THRESHOLD);

    // Sort by confidence descending so highest-confidence skills come first
    skills.sort_by(|a, b| b.confidence.partial_cmp(&a.confidence).unwrap_or(std::cmp::Ordering::Equal));

    // ── Extract top-level stack fields ────────────────────────────────────────
    let language   = infer_language(&skills, &internal);
    let framework  = infer_framework(&skills);
    let styling    = infer_styling(&skills);
    let state_mgmt = infer_state_management(&skills);
    let testing    = infer_testing(&skills);
    let database   = infer_database(&skills);
    let runtime    = infer_runtime(&skills, &language);

    Ok(StackResult {
        language,
        framework,
        styling,
        state_management: state_mgmt,
        testing,
        database,
        runtime,
        skills,
        architecture_style: arch_style,
    })
}

// ── Skill merging ─────────────────────────────────────────────────────────────

/// Merge skill lists from all layers: keep the max confidence per skill name.
fn merge_skills(layers: Vec<Vec<Skill>>) -> Vec<Skill> {
    let mut map: HashMap<String, Skill> = HashMap::new();

    for skill in layers.into_iter().flatten() {
        map.entry(skill.name.clone())
            .and_modify(|existing| {
                // Keep the source of the higher-confidence detection
                if skill.confidence > existing.confidence {
                    *existing = skill.clone();
                }
            })
            .or_insert(skill);
    }

    map.into_values().collect()
}

// ── Top-level field inference ─────────────────────────────────────────────────

fn infer_language(skills: &[Skill], internal: &[Skill]) -> String {
    // Internal __project_name markers tell us the primary language via file presence
    // First check explicit language skills
    let candidates = [
        ("Rust", "Rust"),
        ("Python", "Python"),
        ("Go", "Go"),
        ("Java", "Java"),
        ("C#", "C#"),
        ("Ruby", "Ruby"),
        ("TypeScript", "TypeScript"),
    ];

    for (skill_needle, lang) in &candidates {
        if skills.iter().any(|s| s.name.contains(skill_needle) && s.confidence >= PRIMARY_THRESHOLD) {
            return lang.to_string();
        }
    }

    // Fallback: if we have any TypeScript skill at all
    if skills.iter().any(|s| s.name.contains("TypeScript")) {
        return "TypeScript".to_string();
    }

    // Check internal markers for Cargo.toml presence
    let _ = internal; // internal markers are checked upstream in deps.rs

    "Unknown".to_string()
}

fn infer_framework(skills: &[Skill]) -> Option<String> {
    let framework_priority = [
        "Next.js",
        "Nuxt.js",
        "Remix",
        "Astro",
        "Gatsby",
        "NestJS",
        "Rust Web Server (Axum)",
        "Rust Web Server (Actix)",
        "Node.js API Server (Fastify)",
        "Node.js API Server (Express)",
        "Node.js API Server (Hono)",
        "Vue.js",
        "Svelte",
        "Angular",
        "React",
    ];

    for needle in &framework_priority {
        if let Some(s) = skills.iter().find(|s| s.name.contains(needle) && s.confidence >= INCLUDE_THRESHOLD) {
            // Return clean names
            return Some(clean_framework_name(&s.name));
        }
    }

    None
}

fn infer_styling(skills: &[Skill]) -> Option<String> {
    let styling_options = ["Tailwind CSS", "Styled Components", "Emotion CSS", "Sass/SCSS"];

    for needle in &styling_options {
        if skills.iter().any(|s| s.name.contains(needle)) {
            return Some(needle.to_string());
        }
    }

    None
}

fn infer_state_management(skills: &[Skill]) -> Option<String> {
    if let Some(s) = skills.iter().find(|s| s.name.starts_with("State Management (")) {
        // Extract "(Zustand)" → "Zustand"
        let inner = s.name
            .trim_start_matches("State Management (")
            .trim_end_matches(')');
        return Some(inner.to_string());
    }
    None
}

fn infer_testing(skills: &[Skill]) -> Option<String> {
    let testing_options = [
        "Testing (Vitest)", "Testing (Jest)",
        "E2E Testing (Playwright)", "E2E Testing (Cypress)",
    ];

    for needle in &testing_options {
        if skills.iter().any(|s| s.name.contains(needle)) {
            return Some(needle.trim_start_matches("Testing (").trim_end_matches(')').to_string());
        }
    }

    if skills.iter().any(|s| s.name == "Testing") {
        return Some("Unknown".to_string());
    }

    None
}

fn infer_database(skills: &[Skill]) -> Option<String> {
    let db_options = [
        "Database ORM (Prisma)", "Database ORM (Drizzle)", "Database ORM (TypeORM)",
        "MongoDB (Mongoose)", "Rust Database (SQLx)", "Rust Database (Diesel)",
        "PostgreSQL", "MySQL", "SQLite",
    ];

    for needle in &db_options {
        if skills.iter().any(|s| s.name.contains(needle)) {
            return Some(needle.trim_start_matches("Database ORM (")
                               .trim_start_matches("Rust Database (")
                               .trim_end_matches(')').to_string());
        }
    }

    None
}

fn infer_runtime(skills: &[Skill], language: &str) -> Option<String> {
    match language {
        "Rust"   => Some("Rust (native)".to_string()),
        "Go"     => Some("Go runtime".to_string()),
        "Python" => Some("CPython".to_string()),
        _ => {
            // Node.js vs Bun vs Deno
            if skills.iter().any(|s| s.name.contains("Bun")) {
                Some("Bun".to_string())
            } else {
                Some("Node.js".to_string())
            }
        }
    }
}

fn clean_framework_name(raw: &str) -> String {
    // "Rust Web Server (Axum)" → "Axum"
    if raw.contains('(') {
        raw.split('(')
           .nth(1)
           .unwrap_or(raw)
           .trim_end_matches(')')
           .to_string()
    } else {
        raw.to_string()
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{FolderMap, SignalFile, SignalKind};
    use std::collections::HashMap;
    use std::path::PathBuf;

    fn make_scan(pkg_json: &str, folders: &[&str], patterns: &[&str]) -> ScanResult {
        let signal = SignalFile {
            kind: SignalKind::PackageJson,
            path: PathBuf::from("package.json"),
            content: pkg_json.to_string(),
        };

        let folder_map: FolderMap = folders
            .iter()
            .map(|&f| (f.to_string(), vec![]))
            .collect::<HashMap<_, _>>();

        ScanResult {
            root: PathBuf::from("."),
            signal_files: vec![signal],
            folder_map,
            file_patterns: patterns.iter().map(|s| s.to_string()).collect(),
            readme_excerpt: None,
        }
    }

    #[test]
    fn detects_nextjs_stack() {
        let scan = make_scan(
            r#"{"dependencies":{"next":"14","react":"18","tailwindcss":"3","prisma":"5"},"devDependencies":{"vitest":"1"}}"#,
            &["components", "hooks", "services", "prisma"],
            &["*.test.ts", ".ts", ".tsx", "tailwind.config.ts"],
        );

        let stack = detect(&scan).unwrap();

        assert_eq!(stack.language, "TypeScript");
        assert_eq!(stack.framework.as_deref(), Some("Next.js"));
        assert_eq!(stack.styling.as_deref(), Some("Tailwind CSS"));
        assert!(stack.database.is_some());
        assert!(stack.testing.is_some());
        assert!(stack.skills.iter().all(|s| s.confidence >= INCLUDE_THRESHOLD));
    }

    #[test]
    fn detects_rust_axum_stack() {
        let cargo = r#"
            [package]
            name = "my-api"
            [dependencies]
            axum = "0.7"
            sqlx = "0.7"
            tokio = "1"
        "#;

        let signal = SignalFile {
            kind: SignalKind::CargoToml,
            path: PathBuf::from("Cargo.toml"),
            content: cargo.to_string(),
        };

        let scan = ScanResult {
            root: PathBuf::from("."),
            signal_files: vec![signal],
            folder_map: HashMap::new(),
            file_patterns: vec![".rs".to_string()],
            readme_excerpt: None,
        };

        let stack = detect(&scan).unwrap();

        assert_eq!(stack.language, "Rust");
        assert_eq!(stack.framework.as_deref(), Some("Axum"));
        assert!(stack.database.is_some());
    }

    #[test]
    fn skills_sorted_by_confidence_descending() {
        let scan = make_scan(
            r#"{"dependencies":{"next":"14","react":"18"}}"#,
            &[],
            &[],
        );

        let stack = detect(&scan).unwrap();
        let confidences: Vec<f32> = stack.skills.iter().map(|s| s.confidence).collect();
        let mut sorted = confidences.clone();
        sorted.sort_by(|a, b| b.partial_cmp(a).unwrap());
        assert_eq!(confidences, sorted);
    }

    #[test]
    fn no_skills_below_threshold_in_output() {
        let scan = make_scan(r#"{"dependencies":{}}"#, &[], &[]);
        let stack = detect(&scan).unwrap();
        assert!(stack.skills.iter().all(|s| s.confidence >= INCLUDE_THRESHOLD));
    }
}