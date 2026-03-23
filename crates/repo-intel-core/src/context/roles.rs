use crate::types::{ArchMeta, StackResult};

/// Map a detected stack to a list of agent role names.
///
/// Role priority rules (from PLAN.md):
/// - SSR frameworks → "Fullstack Engineer" (takes priority over plain Frontend)
/// - Plain UI frameworks → "Frontend Engineer"
/// - API servers → "Backend API Engineer"
/// - Testing signals → "QA & Testing Engineer"
/// - Database signals → "Database Engineer"
/// - Docker or CI → "DevOps Engineer"
/// - GraphQL → "GraphQL Engineer"
/// - Rust CLI → "Developer Tooling Engineer"
/// - WebAssembly → "WebAssembly Engineer"
pub fn map_roles(stack: &StackResult, arch: &ArchMeta) -> Vec<String> {
    let mut roles: Vec<String> = Vec::new();

    // Closure: does the stack contain a skill matching `needle` at confidence ≥ 0.50?
    let has = |needle: &str| {
        stack
            .skills
            .iter()
            .any(|s| s.name.to_lowercase().contains(&needle.to_lowercase()) && s.confidence >= 0.50)
    };

    // ── Frontend / Fullstack ──────────────────────────────────────────────────
    // SSR implies fullstack — check before plain frontend
    if has("SSR") || has("Next.js") || has("Nuxt") || has("Remix") || has("Gatsby") {
        roles.push("Fullstack Engineer".into());
    } else if has("React") || has("Vue") || has("Svelte") || has("Angular") || has("Astro") {
        roles.push("Frontend Engineer".into());
    }

    // ── Backend API ───────────────────────────────────────────────────────────
    if has("API Server")
        || has("Express")
        || has("Fastify")
        || has("Hono")
        || has("Koa")
        || has("Axum")
        || has("Actix")
        || has("NestJS")
        || has("Warp")
        || has("Rocket")
        || has("Elysia")
    {
        roles.push("Backend API Engineer".into());
    }

    // ── Testing ───────────────────────────────────────────────────────────────
    if has("Testing") || has("Vitest") || has("Jest") || has("Playwright") || has("Cypress") {
        roles.push("QA & Testing Engineer".into());
    }

    // ── Database ──────────────────────────────────────────────────────────────
    if has("Database")
        || has("Prisma")
        || has("SQLx")
        || has("Diesel")
        || has("Drizzle")
        || has("TypeORM")
        || has("Mongoose")
        || has("SeaORM")
    {
        roles.push("Database Engineer".into());
    }

    // ── DevOps ────────────────────────────────────────────────────────────────
    if arch.has_docker || arch.has_ci {
        roles.push("DevOps Engineer".into());
    }

    // ── GraphQL ───────────────────────────────────────────────────────────────
    if has("GraphQL") {
        roles.push("GraphQL Engineer".into());
    }

    // ── Rust-specific roles ───────────────────────────────────────────────────
    if has("Rust CLI") {
        roles.push("Developer Tooling Engineer".into());
    }

    if has("WebAssembly") {
        roles.push("WebAssembly Engineer".into());
    }

    // ── Monorepo ──────────────────────────────────────────────────────────────
    // If monorepo + multiple other roles, add a platform engineer role
    if arch.has_monorepo && roles.len() >= 2 {
        roles.push("Platform / Monorepo Engineer".into());
    }

    roles
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Skill, SkillSource, StackResult};

    fn make_stack(skill_names: &[(&str, f32)]) -> StackResult {
        StackResult {
            language: "TypeScript".into(),
            framework: None,
            styling: None,
            state_management: None,
            testing: None,
            database: None,
            runtime: None,
            skills: skill_names
                .iter()
                .map(|(name, conf)| Skill {
                    name: name.to_string(),
                    confidence: *conf,
                    source: SkillSource::PackageJson,
                })
                .collect(),
            architecture_style: None,
        }
    }

    fn make_arch(has_docker: bool, has_ci: bool, has_monorepo: bool) -> ArchMeta {
        ArchMeta {
            style: None,
            folders: vec![],
            has_monorepo,
            has_docker,
            has_ci,
            has_git: true,
        }
    }

    #[test]
    fn nextjs_produces_fullstack_not_frontend() {
        let stack = make_stack(&[("Next.js", 0.99), ("SSR", 0.99), ("React", 0.97)]);
        let arch = make_arch(false, false, false);
        let roles = map_roles(&stack, &arch);
        assert!(roles.contains(&"Fullstack Engineer".to_string()));
        assert!(!roles.contains(&"Frontend Engineer".to_string()));
    }

    #[test]
    fn plain_react_produces_frontend() {
        let stack = make_stack(&[("React", 0.99), ("Component Architecture", 0.90)]);
        let arch = make_arch(false, false, false);
        let roles = map_roles(&stack, &arch);
        assert!(roles.contains(&"Frontend Engineer".to_string()));
        assert!(!roles.contains(&"Fullstack Engineer".to_string()));
    }

    #[test]
    fn axum_produces_backend_api_engineer() {
        let stack = make_stack(&[("Rust", 0.99), ("Rust Web Server (Axum)", 0.99)]);
        let arch = make_arch(false, false, false);
        let roles = map_roles(&stack, &arch);
        assert!(roles.contains(&"Backend API Engineer".to_string()));
    }

    #[test]
    fn docker_produces_devops_engineer() {
        let stack = make_stack(&[("React", 0.99)]);
        let arch = make_arch(true, false, false);
        let roles = map_roles(&stack, &arch);
        assert!(roles.contains(&"DevOps Engineer".to_string()));
    }

    #[test]
    fn ci_produces_devops_engineer() {
        let stack = make_stack(&[]);
        let arch = make_arch(false, true, false);
        let roles = map_roles(&stack, &arch);
        assert!(roles.contains(&"DevOps Engineer".to_string()));
    }

    #[test]
    fn monorepo_with_multiple_roles_adds_platform_engineer() {
        let stack = make_stack(&[("Next.js", 0.99), ("SSR", 0.99), ("Testing", 0.90)]);
        let arch = make_arch(false, false, true);
        let roles = map_roles(&stack, &arch);
        assert!(roles.contains(&"Platform / Monorepo Engineer".to_string()));
    }

    #[test]
    fn graphql_adds_graphql_engineer() {
        let stack = make_stack(&[("GraphQL", 0.97), ("React", 0.99)]);
        let arch = make_arch(false, false, false);
        let roles = map_roles(&stack, &arch);
        assert!(roles.contains(&"GraphQL Engineer".to_string()));
    }

    #[test]
    fn empty_stack_produces_no_roles() {
        let stack = make_stack(&[]);
        let arch = make_arch(false, false, false);
        let roles = map_roles(&stack, &arch);
        assert!(roles.is_empty());
    }

    #[test]
    fn low_confidence_skills_excluded_from_role_mapping() {
        // confidence 0.40 — below INCLUDE_THRESHOLD of 0.50
        let stack = make_stack(&[("React", 0.40)]);
        let arch = make_arch(false, false, false);
        let roles = map_roles(&stack, &arch);
        assert!(!roles.contains(&"Frontend Engineer".to_string()));
    }
}
