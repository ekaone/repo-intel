use crate::types::{ScanResult, Skill};
use serde_json::Value;

/// Layer 1: Parse dependency files to detect skills with confidence scores.
pub fn detect_from_deps(scan: &ScanResult) -> Vec<Skill> {
    let mut skills: Vec<Skill> = Vec::new();

    if let Some(pkg) = scan.signal_files.get("package.json") {
        skills.extend(detect_from_package_json(pkg));
    }

    if let Some(cargo) = scan.signal_files.get("Cargo.toml") {
        skills.extend(detect_from_cargo_toml(cargo));
    }

    if scan.signal_files.contains_key("go.mod") {
        skills.push(Skill {
            name: "go".into(),
            category: "language".into(),
            confidence: 0.95,
            signals: vec!["go.mod".into()],
        });
    }

    if scan.signal_files.contains_key("requirements.txt")
        || scan.signal_files.contains_key("pyproject.toml")
        || scan.signal_files.contains_key("Pipfile")
    {
        skills.push(Skill {
            name: "python".into(),
            category: "language".into(),
            confidence: 0.95,
            signals: vec!["requirements.txt/pyproject.toml".into()],
        });
    }

    skills
}

fn detect_from_package_json(content: &str) -> Vec<Skill> {
    let mut skills: Vec<Skill> = Vec::new();

    // Base: Node.js / JS detected
    skills.push(Skill {
        name: "nodejs".into(),
        category: "language".into(),
        confidence: 0.9,
        signals: vec!["package.json".into()],
    });

    let Ok(value): Result<Value, _> = serde_json::from_str(content) else {
        return skills;
    };

    let all_deps = {
        let mut d: Vec<String> = Vec::new();
        for key in &["dependencies", "devDependencies", "peerDependencies"] {
            if let Some(obj) = value.get(key).and_then(Value::as_object) {
                d.extend(obj.keys().cloned());
            }
        }
        d
    };

    // TypeScript
    if all_deps.iter().any(|d| d == "typescript") {
        skills.push(Skill {
            name: "typescript".into(),
            category: "language".into(),
            confidence: 0.95,
            signals: vec!["typescript in package.json".into()],
        });
    }

    // Framework detection map: (dep, skill_name, category, confidence)
    let framework_map: &[(&str, &str, &str, f32)] = &[
        ("next", "nextjs", "framework", 0.95),
        ("react", "react", "framework", 0.9),
        ("vue", "vue", "framework", 0.9),
        ("@angular/core", "angular", "framework", 0.95),
        ("svelte", "svelte", "framework", 0.9),
        ("express", "express", "framework", 0.9),
        ("fastify", "fastify", "framework", 0.9),
        ("hono", "hono", "framework", 0.9),
        ("koa", "koa", "framework", 0.9),
        ("@nestjs/core", "nestjs", "framework", 0.95),
        ("nuxt", "nuxt", "framework", 0.95),
        ("remix", "remix", "framework", 0.95),
        ("astro", "astro", "framework", 0.9),
        ("vite", "vite", "tooling", 0.9),
        ("webpack", "webpack", "tooling", 0.85),
        ("turbo", "turborepo", "tooling", 0.85),
        ("nx", "nx", "tooling", 0.85),
        ("vitest", "vitest", "tooling", 0.85),
        ("jest", "jest", "tooling", 0.85),
        ("eslint", "eslint", "tooling", 0.8),
        ("prettier", "prettier", "tooling", 0.8),
        ("@biomejs/biome", "biome", "tooling", 0.85),
        ("prisma", "prisma", "tooling", 0.9),
        ("drizzle-orm", "drizzle", "tooling", 0.9),
        ("trpc", "@trpc/server", "framework", 0.9),
        ("graphql", "graphql", "framework", 0.85),
    ];

    for dep in &all_deps {
        for (pattern, name, category, confidence) in framework_map {
            if dep == pattern || dep.starts_with(pattern) {
                skills.push(Skill {
                    name: (*name).into(),
                    category: (*category).into(),
                    confidence: *confidence,
                    signals: vec![format!("{dep} in package.json")],
                });
            }
        }
    }

    skills
}

fn detect_from_cargo_toml(content: &str) -> Vec<Skill> {
    let mut skills: Vec<Skill> = Vec::new();

    skills.push(Skill {
        name: "rust".into(),
        category: "language".into(),
        confidence: 0.99,
        signals: vec!["Cargo.toml".into()],
    });

    let framework_map: &[(&str, &str, &str, f32)] = &[
        ("axum", "axum", "framework", 0.95),
        ("actix-web", "actix", "framework", 0.95),
        ("warp", "warp", "framework", 0.9),
        ("rocket", "rocket", "framework", 0.9),
        ("tokio", "tokio", "tooling", 0.9),
        ("serde", "serde", "tooling", 0.8),
        ("sqlx", "sqlx", "tooling", 0.9),
        ("diesel", "diesel", "tooling", 0.9),
    ];

    for (pattern, name, category, confidence) in framework_map {
        if content.contains(pattern) {
            skills.push(Skill {
                name: (*name).into(),
                category: (*category).into(),
                confidence: *confidence,
                signals: vec![format!("{pattern} in Cargo.toml")],
            });
        }
    }

    skills
}
