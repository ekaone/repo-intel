pub mod enricher;
pub mod roles;
pub mod serializer;

use std::path::Path;

use crate::config::Config;
use crate::error::{RepoIntelError, Result};
use crate::types::{RepoContext, ScanResult, StackResult};

/// Build the final `RepoContext` from scan + detection outputs.
///
/// Steps:
/// 1. Enrich → `ProjectMeta` + `ArchMeta` (git, docker, CI, monorepo, folders)
/// 2. Map roles → `Vec<String>` (which agent docs to generate)
/// 3. Stamp version + timestamp → `RepoContext`
pub fn build(
    root: &Path,
    scan: &ScanResult,
    stack: StackResult,
    _cfg: &Config,
) -> Result<RepoContext> {
    // ── Step 1: Enrich ────────────────────────────────────────────────────────
    let (project, arch) = enricher::enrich(
        root,
        &scan.signal_files,
        &scan.folder_map,
        scan.readme_excerpt.clone(),
        &stack.skills,
        stack.architecture_style.clone(),
    );

    // ── Step 2: Map agent roles ───────────────────────────────────────────────
    let agent_roles = roles::map_roles(&stack, &arch);

    if agent_roles.is_empty() {
        // Non-fatal: warn but continue — JS layer can still run with an empty roles list
        eprintln!("warn: no agent roles detected — the repo may be empty or unrecognised");
    }

    // ── Step 3: Assemble RepoContext ──────────────────────────────────────────
    let ctx = RepoContext {
        version: env!("CARGO_PKG_VERSION").to_string(),
        scanned_at: current_timestamp(),
        root: root.to_path_buf(),
        project,
        stack,
        architecture: arch,
        agent_roles,
    };

    Ok(ctx)
}

// ── Helpers ───────────────────────────────────────────────────────────────────

/// Return the current UTC time as an ISO 8601 string.
/// Uses a hand-rolled formatter to avoid pulling in `chrono` for MVP.
fn current_timestamp() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};

    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    // Convert Unix timestamp → approximate ISO 8601 without chrono
    // Accurate enough for a metadata field — not used for calculations.
    let days_since_epoch = secs / 86_400;
    let time_of_day = secs % 86_400;
    let hh = time_of_day / 3_600;
    let mm = (time_of_day % 3_600) / 60;
    let ss = time_of_day % 60;

    // Gregorian calendar math
    let (year, month, day) = days_to_ymd(days_since_epoch);

    format!("{year:04}-{month:02}-{day:02}T{hh:02}:{mm:02}:{ss:02}Z")
}

/// Convert days-since-epoch to (year, month, day).
fn days_to_ymd(mut days: u64) -> (u64, u64, u64) {
    // Algorithm: http://howardhinnant.github.io/date_algorithms.html
    days += 719_468;
    let era = days / 146_097;
    let doe = days % 146_097;
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    (y, m, d)
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{FolderMap, SignalFile, SignalKind, Skill, SkillSource};
    use std::collections::HashMap;
    use std::path::PathBuf;

    fn make_scan(pkg_json: &str, folders: &[&str]) -> ScanResult {
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
            file_patterns: vec![".ts".to_string(), ".tsx".to_string()],
            readme_excerpt: Some("A Next.js dashboard application.".into()),
        }
    }

    fn make_stack(skills: &[(&str, f32)]) -> StackResult {
        StackResult {
            language: "TypeScript".into(),
            framework: Some("Next.js".into()),
            styling: Some("Tailwind CSS".into()),
            state_management: None,
            testing: Some("Vitest".into()),
            database: Some("Prisma".into()),
            runtime: Some("Node.js".into()),
            skills: skills
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

    #[test]
    fn build_produces_valid_context() {
        let scan = make_scan(
            r#"{"name":"my-app","description":"A dashboard","dependencies":{"next":"14"}}"#,
            &["components", "hooks", "services"],
        );
        let stack = make_stack(&[
            ("Next.js", 0.99),
            ("SSR", 0.99),
            ("React", 0.97),
            ("Testing", 0.90),
        ]);

        let cfg = Config::default();
        let ctx = build(Path::new("."), &scan, stack, &cfg).unwrap();

        assert_eq!(ctx.project.name, "my-app");
        assert_eq!(ctx.project.description.as_deref(), Some("A dashboard"));
        assert!(ctx.readme_excerpt().is_some());
        assert!(!ctx.agent_roles.is_empty());
        assert!(ctx.agent_roles.contains(&"Fullstack Engineer".to_string()));
        assert!(!ctx.version.is_empty());
        assert!(!ctx.scanned_at.is_empty());
    }

    #[test]
    fn timestamp_looks_like_iso8601() {
        let ts = current_timestamp();
        // Should match pattern: YYYY-MM-DDTHH:MM:SSZ
        assert_eq!(ts.len(), 20);
        assert!(ts.ends_with('Z'));
        assert!(ts.contains('T'));
        assert!(ts.chars().nth(4) == Some('-'));
    }

    #[test]
    fn empty_stack_warns_but_does_not_error() {
        let scan = make_scan(r#"{"name":"empty-app"}"#, &[]);
        let stack = make_stack(&[]);
        let cfg = Config::default();
        let result = build(Path::new("."), &scan, stack, &cfg);
        assert!(result.is_ok());
        let ctx = result.unwrap();
        assert!(ctx.agent_roles.is_empty());
    }
}

// ── RepoContext convenience methods ───────────────────────────────────────────
// Placed here (not types.rs) since they depend on context-layer logic.

impl RepoContext {
    /// Convenience accessor for the README excerpt.
    pub fn readme_excerpt(&self) -> Option<&str> {
        self.project.readme_excerpt.as_deref()
    }

    /// Return only primary skills (confidence ≥ 0.90).
    pub fn primary_skills(&self) -> impl Iterator<Item = &crate::types::Skill> {
        self.stack.skills.iter().filter(|s| s.confidence >= 0.90)
    }

    /// Return secondary skills (0.50 ≤ confidence < 0.90).
    pub fn secondary_skills(&self) -> impl Iterator<Item = &crate::types::Skill> {
        self.stack
            .skills
            .iter()
            .filter(|s| s.confidence >= 0.50 && s.confidence < 0.90)
    }
}
