use crate::types::{ScanResult, Skill};

/// Layer 2: Infer skills from folder names found in the repo.
pub fn detect_from_folders(scan: &ScanResult) -> Vec<Skill> {
    let mut skills: Vec<Skill> = Vec::new();
    let dirs: Vec<&str> = scan.folder_map.keys().map(|s| s.as_str()).collect();

    let folder_map: &[(&str, &str, &str, f32)] = &[
        ("prisma", "prisma", "tooling", 0.8),
        ("migrations", "database-migrations", "tooling", 0.7),
        ("__tests__", "jest", "tooling", 0.75),
        ("cypress", "cypress", "tooling", 0.9),
        ("e2e", "e2e-testing", "tooling", 0.7),
        ("storybook", "storybook", "tooling", 0.9),
        (".storybook", "storybook", "tooling", 0.95),
        ("terraform", "terraform", "tooling", 0.9),
        ("k8s", "kubernetes", "tooling", 0.85),
        ("kubernetes", "kubernetes", "tooling", 0.9),
        ("helm", "helm", "tooling", 0.9),
        ("proto", "protobuf", "tooling", 0.8),
        ("graphql", "graphql", "framework", 0.8),
    ];

    for dir in &dirs {
        let last_segment = dir.split('/').last().unwrap_or(dir);
        for (pattern, name, category, confidence) in folder_map {
            if last_segment == *pattern {
                skills.push(Skill {
                    name: (*name).into(),
                    category: (*category).into(),
                    confidence: *confidence,
                    signals: vec![format!("folder: {dir}")],
                });
            }
        }
    }

    skills
}

/// Infer high-level architecture patterns from folder structure.
pub fn infer_architecture(scan: &ScanResult) -> Vec<String> {
    let mut patterns: Vec<String> = Vec::new();
    let dirs: Vec<&str> = scan.folder_map.keys().map(|s| s.as_str()).collect();

    let has_dir = |name: &str| dirs.iter().any(|d| d.split('/').any(|seg| seg == name));

    if has_dir("packages") || has_dir("apps") {
        patterns.push("monorepo".into());
    }
    if has_dir("components") && has_dir("hooks") {
        patterns.push("react-component-architecture".into());
    }
    if has_dir("controllers") && has_dir("services") {
        patterns.push("mvc".into());
    }
    if has_dir("domain") || has_dir("application") || has_dir("infrastructure") {
        patterns.push("clean-architecture".into());
    }
    if has_dir("handlers") && has_dir("middleware") {
        patterns.push("handler-middleware-pattern".into());
    }

    patterns
}
