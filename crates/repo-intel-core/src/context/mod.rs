use crate::types::{RepoContext, StackResult};

pub mod enricher;
pub mod roles;
pub mod serializer;

/// Build a `RepoContext` from a `StackResult`.
pub fn build(stack: &StackResult) -> RepoContext {
    RepoContext {
        name: String::new(), // enricher fills this in
        stack: stack.clone(),
        agent_roles: roles::derive_roles(stack),
        readme_excerpt: None, // enricher fills this in
        has_git: false,       // enricher fills this in
        has_docker: false,    // enricher fills this in
        has_ci: false,        // enricher fills this in
        schema_version: "1".into(),
    }
}
