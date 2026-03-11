use crate::types::{AgentRole, StackResult};

/// Derive the list of agent roles to generate based on the detected stack.
pub fn derive_roles(stack: &StackResult) -> Vec<AgentRole> {
    let mut roles: Vec<AgentRole> = Vec::new();

    let has_framework = |name: &str| stack.frameworks.iter().any(|f| f.name == name);
    let has_language = |name: &str| stack.languages.iter().any(|l| l.name == name);
    let has_tooling = |name: &str| stack.tooling.iter().any(|t| t.name == name);

    // Always include a general architect role
    roles.push(AgentRole {
        id: "architect".into(),
        title: "Software Architect".into(),
        description: "High-level system design, module boundaries, and architectural decisions."
            .into(),
    });

    // Frontend roles
    if has_framework("nextjs") || has_framework("react") || has_framework("vue") {
        roles.push(AgentRole {
            id: "frontend".into(),
            title: "Frontend Engineer".into(),
            description: "UI components, routing, state management, and rendering patterns.".into(),
        });
    }

    // Backend roles
    if has_framework("express")
        || has_framework("fastify")
        || has_framework("nestjs")
        || has_framework("axum")
        || has_framework("actix")
    {
        roles.push(AgentRole {
            id: "backend".into(),
            title: "Backend Engineer".into(),
            description: "API design, request handling, middleware, and service layer logic.".into(),
        });
    }

    // Rust-specific role
    if has_language("rust") {
        roles.push(AgentRole {
            id: "rust-engineer".into(),
            title: "Rust Engineer".into(),
            description:
                "Memory safety, ownership patterns, async with Tokio, and Cargo workspace management."
                    .into(),
        });
    }

    // DevOps / infrastructure roles
    if has_tooling("docker") || has_tooling("docker-compose") || has_tooling("kubernetes") {
        roles.push(AgentRole {
            id: "devops".into(),
            title: "DevOps / Infrastructure Engineer".into(),
            description: "Containerization, orchestration, CI/CD pipelines, and deployment.".into(),
        });
    }

    // Testing roles
    if has_tooling("jest") || has_tooling("vitest") || has_tooling("cypress") {
        roles.push(AgentRole {
            id: "qa".into(),
            title: "QA / Test Engineer".into(),
            description: "Test strategy, unit tests, integration tests, and E2E coverage.".into(),
        });
    }

    // Monorepo role
    if stack.architecture.iter().any(|a| a == "monorepo") {
        roles.push(AgentRole {
            id: "monorepo".into(),
            title: "Monorepo Maintainer".into(),
            description: "Workspace tooling, shared packages, dependency management, and build orchestration.".into(),
        });
    }

    roles
}
