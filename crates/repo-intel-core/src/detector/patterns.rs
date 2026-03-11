use crate::types::{ScanResult, Skill};

/// Layer 3: Detect skills from specific filename patterns across the repo.
pub fn detect_from_patterns(scan: &ScanResult) -> Vec<Skill> {
    let mut skills: Vec<Skill> = Vec::new();

    let all_files: Vec<&str> = scan
        .folder_map
        .values()
        .flat_map(|files| files.iter().map(|f| f.as_str()))
        .collect();

    let pattern_map: &[(&str, &str, &str, f32, bool)] = &[
        // (filename_contains, skill_name, category, confidence, exact_match)
        ("Dockerfile", "docker", "tooling", 0.95, true),
        ("docker-compose.yml", "docker-compose", "tooling", 0.95, true),
        ("docker-compose.yaml", "docker-compose", "tooling", 0.95, true),
        (".eslintrc", "eslint", "tooling", 0.9, false),
        (".prettier", "prettier", "tooling", 0.9, false),
        ("biome.json", "biome", "tooling", 0.95, true),
        ("tsconfig.json", "typescript", "language", 0.9, true),
        ("jest.config", "jest", "tooling", 0.9, false),
        ("vitest.config", "vitest", "tooling", 0.9, false),
        ("vite.config", "vite", "tooling", 0.9, false),
        ("next.config", "nextjs", "framework", 0.95, false),
        ("nuxt.config", "nuxt", "framework", 0.95, false),
        ("svelte.config", "svelte", "framework", 0.95, false),
        ("astro.config", "astro", "framework", 0.95, false),
        ("turbo.json", "turborepo", "tooling", 0.95, true),
        ("nx.json", "nx", "tooling", 0.95, true),
        (".terraform", "terraform", "tooling", 0.9, false),
        ("Makefile", "make", "tooling", 0.8, true),
        ("Justfile", "just", "tooling", 0.85, true),
    ];

    for file in &all_files {
        for (pattern, name, category, confidence, exact) in pattern_map {
            let matched = if *exact {
                file == pattern
            } else {
                file.contains(pattern)
            };

            if matched {
                skills.push(Skill {
                    name: (*name).into(),
                    category: (*category).into(),
                    confidence: *confidence,
                    signals: vec![format!("file: {file}")],
                });
            }
        }
    }

    skills
}
