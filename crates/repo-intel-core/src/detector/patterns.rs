use crate::types::{Skill, SkillSource};

/// Layer 3 — file pattern recognition.
///
/// Infers skills from collected file patterns (extensions + compound names).
/// This layer fills gaps that deps and folders cannot: e.g. a project may have
/// Storybook without it being in `dependencies`, or use GraphQL schema files
/// without an Apollo dependency.
pub fn detect_from_patterns(file_patterns: &[String]) -> Vec<Skill> {
    let mut skills = Vec::new();

    for pattern in file_patterns {
        let matched = pattern_skill_rules(pattern.as_str());
        skills.extend(matched);
    }

    skills
}

fn pattern_skill_rules(pattern: &str) -> Vec<Skill> {
    let src = |p: &str| SkillSource::FilePattern(p.to_string());

    let skill = |name: &str, confidence: f32, pat: &str| Skill {
        name: name.to_string(),
        confidence,
        source: src(pat),
    };

    match pattern {
        // ── Testing patterns ──────────────────────────────────────────────────
        "*.test.ts" | "*.test.tsx" | "*.spec.ts" | "*.spec.tsx" => vec![
            skill("Testing", 0.88, pattern),
            skill("TDD Practice", 0.75, pattern),
        ],
        "*.test.js" | "*.spec.js" => vec![
            skill("Testing", 0.85, pattern),
        ],
        "*.e2e.ts" | "*.e2e-spec.ts" => vec![
            skill("E2E Testing", 0.88, pattern),
            skill("Testing", 0.85, pattern),
        ],

        // ── Service / architecture patterns ───────────────────────────────────
        "*.service.ts" => vec![
            skill("Service Layer Architecture", 0.85, pattern),
        ],
        "*.controller.ts" => vec![
            skill("Controller Pattern (NestJS)", 0.88, pattern),
            skill("NestJS", 0.80, pattern),
        ],
        "*.module.ts" => vec![
            skill("Module Architecture (NestJS)", 0.85, pattern),
            skill("NestJS", 0.80, pattern),
        ],
        "*.middleware.ts" => vec![
            skill("Middleware Pattern", 0.80, pattern),
        ],
        "*.guard.ts" | "*.interceptor.ts" => vec![
            skill("NestJS", 0.82, pattern),
        ],

        // ── Data modelling ────────────────────────────────────────────────────
        "*.schema.ts" => vec![
            skill("Data Modelling", 0.82, pattern),
        ],
        "*.model.ts" => vec![
            skill("Data Modelling", 0.80, pattern),
        ],
        "schema.prisma" => vec![
            skill("Database ORM (Prisma)", 0.95, pattern),
            skill("Database", 0.95, pattern),
        ],

        // ── GraphQL ───────────────────────────────────────────────────────────
        "*.graphql" | "*.gql" => vec![
            skill("GraphQL", 0.92, pattern),
            skill("GraphQL Schema-First", 0.88, pattern),
        ],

        // ── Storybook / component docs ────────────────────────────────────────
        "*.stories.tsx" | "*.stories.ts" | "*.stories.jsx" => vec![
            skill("Storybook", 0.92, pattern),
            skill("Component Documentation", 0.88, pattern),
        ],

        // ── Infrastructure ────────────────────────────────────────────────────
        "Dockerfile" => vec![
            skill("Containerized Deployment", 0.92, pattern),
            skill("Docker", 0.92, pattern),
            skill("__has_docker", 0.99, pattern),
        ],
        "docker-compose.yml" | "docker-compose.yaml" => vec![
            skill("Docker Compose", 0.92, pattern),
            skill("Containerized Deployment", 0.88, pattern),
            skill("__has_docker", 0.99, pattern),
        ],

        // ── Build / config files ──────────────────────────────────────────────
        "vite.config.ts" => vec![
            skill("Vite", 0.97, pattern),
        ],
        "next.config.js" | "next.config.ts" => vec![
            skill("Next.js", 0.97, pattern),
            skill("SSR", 0.90, pattern),
        ],
        "tailwind.config.ts" | "tailwind.config.js" => vec![
            skill("Tailwind CSS", 0.97, pattern),
        ],
        "jest.config.ts" | "jest.config.js" => vec![
            skill("Testing (Jest)", 0.95, pattern),
            skill("Testing", 0.95, pattern),
        ],
        "vitest.config.ts" => vec![
            skill("Testing (Vitest)", 0.95, pattern),
            skill("Testing", 0.95, pattern),
        ],

        // ── Language indicators ───────────────────────────────────────────────
        ".ts" | ".tsx" => vec![
            skill("TypeScript", 0.92, pattern),
        ],
        ".rs" => vec![
            skill("Rust", 0.99, pattern),
        ],
        ".py" => vec![
            skill("Python", 0.97, pattern),
        ],
        ".go" => vec![
            skill("Go", 0.97, pattern),
        ],
        ".java" => vec![
            skill("Java", 0.97, pattern),
        ],
        ".cs" => vec![
            skill("C#", 0.97, pattern),
        ],
        ".rb" => vec![
            skill("Ruby", 0.97, pattern),
        ],

        // ── CI ────────────────────────────────────────────────────────────────
        // (GitHub workflow YAML is detected in signal.rs; this catches others)
        "Jenkinsfile" => vec![
            skill("__has_ci", 0.95, pattern),
            skill("CI/CD (Jenkins)", 0.90, pattern),
        ],

        _ => vec![],
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn has_skill(skills: &[Skill], name: &str) -> bool {
        skills.iter().any(|s| s.name.contains(name))
    }

    fn detect(patterns: &[&str]) -> Vec<Skill> {
        let owned: Vec<String> = patterns.iter().map(|s| s.to_string()).collect();
        detect_from_patterns(&owned)
    }

    #[test]
    fn detects_testing_from_test_ts() {
        let skills = detect(&["*.test.ts"]);
        assert!(has_skill(&skills, "Testing"));
        assert!(has_skill(&skills, "TDD"));
    }

    #[test]
    fn detects_nestjs_from_controller() {
        let skills = detect(&["*.controller.ts", "*.module.ts"]);
        assert!(has_skill(&skills, "NestJS"));
        assert!(has_skill(&skills, "Controller Pattern"));
    }

    #[test]
    fn detects_graphql_from_schema_file() {
        let skills = detect(&["*.graphql"]);
        assert!(has_skill(&skills, "GraphQL"));
        assert!(has_skill(&skills, "Schema-First"));
    }

    #[test]
    fn detects_storybook_from_stories() {
        let skills = detect(&["*.stories.tsx"]);
        assert!(has_skill(&skills, "Storybook"));
    }

    #[test]
    fn detects_docker_flag() {
        let skills = detect(&["Dockerfile"]);
        assert!(has_skill(&skills, "__has_docker"));
        assert!(has_skill(&skills, "Docker"));
    }

    #[test]
    fn detects_typescript_from_extension() {
        let skills = detect(&[".ts", ".tsx"]);
        assert!(has_skill(&skills, "TypeScript"));
    }

    #[test]
    fn detects_rust_from_rs_extension() {
        let skills = detect(&[".rs"]);
        assert!(has_skill(&skills, "Rust"));
    }

    #[test]
    fn unknown_patterns_produce_no_skills() {
        let skills = detect(&[".unknown_ext", "random_file.xyz"]);
        assert!(skills.is_empty());
    }

    #[test]
    fn detects_prisma_from_schema_file() {
        let skills = detect(&["schema.prisma"]);
        assert!(has_skill(&skills, "Prisma"));
        assert!(has_skill(&skills, "Database"));
    }
}