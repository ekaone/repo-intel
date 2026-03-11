use crate::error::{RepoIntelError, Result};
use crate::types::RepoContext;
use std::path::Path;

/// Serialize `ctx` to a compact JSON string.
pub fn to_json(ctx: &RepoContext) -> Result<String> {
    serde_json::to_string(ctx).map_err(|e| RepoIntelError::JsonSerialize { source: e })
}

/// Serialize `ctx` to a pretty-printed JSON string.
pub fn to_json_pretty(ctx: &RepoContext) -> Result<String> {
    serde_json::to_string_pretty(ctx).map_err(|e| RepoIntelError::JsonSerialize { source: e })
}

/// Write a JSON string to a file at `path`.
pub fn write_to_file(json: &str, path: &Path) -> Result<()> {
    std::fs::write(path, json).map_err(|e| RepoIntelError::FileWrite {
        path: path.to_path_buf(),
        source: e,
    })
}

/// Print compact JSON to stdout (the default pipeline output consumed by JS).
pub fn print_to_stdout(ctx: &RepoContext) -> Result<()> {
    let json = to_json(ctx)?;
    println!("{json}");
    Ok(())
}

/// Print pretty JSON to stdout (for `--pretty` / human inspection).
pub fn print_pretty(ctx: &RepoContext) -> Result<()> {
    let json = to_json_pretty(ctx)?;
    println!("{json}");
    Ok(())
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{ArchMeta, ProjectMeta, RepoContext, StackResult};
    use std::path::PathBuf;
    use tempfile::TempDir;

    fn minimal_context() -> RepoContext {
        RepoContext {
            version: "0.1.0".into(),
            scanned_at: "2026-03-09T00:00:00Z".into(),
            root: PathBuf::from("/tmp/my-app"),
            project: ProjectMeta {
                name: "my-app".into(),
                description: Some("A test app".into()),
                readme_excerpt: None,
            },
            stack: StackResult {
                language: "TypeScript".into(),
                framework: Some("Next.js".into()),
                styling: Some("Tailwind CSS".into()),
                state_management: None,
                testing: Some("Vitest".into()),
                database: Some("Prisma".into()),
                runtime: Some("Node.js".into()),
                skills: vec![],
                architecture_style: None,
            },
            architecture: ArchMeta {
                style: None,
                folders: vec!["components".into(), "hooks".into()],
                has_monorepo: false,
                has_docker: true,
                has_ci: true,
                has_git: true,
            },
            agent_roles: vec!["Fullstack Engineer".into(), "DevOps Engineer".into()],
        }
    }

    #[test]
    fn serializes_to_valid_json() {
        let ctx = minimal_context();
        let json = to_json(&ctx).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed["version"], "0.1.0");
        assert_eq!(parsed["project"]["name"], "my-app");
        assert_eq!(parsed["stack"]["language"], "TypeScript");
        assert_eq!(parsed["stack"]["framework"], "Next.js");
        assert!(parsed["architecture"]["has_docker"].as_bool().unwrap());
        assert!(parsed["architecture"]["has_ci"].as_bool().unwrap());
        assert_eq!(parsed["agent_roles"][0], "Fullstack Engineer");
    }

    #[test]
    fn pretty_json_is_larger_than_compact() {
        let ctx = minimal_context();
        let compact = to_json(&ctx).unwrap();
        let pretty = to_json_pretty(&ctx).unwrap();
        assert!(pretty.len() > compact.len());
        assert!(pretty.contains('\n'));
    }

    #[test]
    fn write_to_file_roundtrip() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("context.json");
        let ctx = minimal_context();

        let json = to_json(&ctx).unwrap();
        write_to_file(&json, &path).unwrap();

        let read_back = std::fs::read_to_string(&path).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&read_back).unwrap();
        assert_eq!(parsed["project"]["name"], "my-app");
    }

    #[test]
    fn json_contains_required_contract_fields() {
        // Validate the Rust↔JS contract fields are all present
        let ctx = minimal_context();
        let json = to_json(&ctx).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

        let required = [
            "version",
            "scanned_at",
            "root",
            "project",
            "stack",
            "architecture",
            "agent_roles",
        ];
        for field in &required {
            assert!(parsed.get(field).is_some(), "Missing field: {field}");
        }

        let stack_fields = [
            "language",
            "framework",
            "styling",
            "testing",
            "database",
            "runtime",
        ];
        for field in &stack_fields {
            assert!(
                parsed["stack"].get(field).is_some(),
                "Missing stack field: {field}"
            );
        }

        let arch_fields = ["folders", "has_monorepo", "has_docker", "has_ci", "has_git"];
        for field in &arch_fields {
            assert!(
                parsed["architecture"].get(field).is_some(),
                "Missing arch field: {field}"
            );
        }
    }
}
