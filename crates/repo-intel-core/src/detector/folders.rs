use crate::types::{ArchStyle, FolderMap, Skill, SkillSource};

/// Layer 2 — folder architecture detection.
///
/// Infers skills and architecture style from the presence of well-known
/// folder names. Complements Layer 1 (deps) when dependency names don't
/// directly reveal architecture patterns.
pub fn detect_from_folders(folder_map: &FolderMap) -> (Vec<Skill>, Option<ArchStyle>) {
    let mut skills = Vec::new();
    let folders: Vec<&str> = folder_map.keys().map(|s| s.as_str()).collect();

    let src = SkillSource::FolderName;

    let skill = |name: &str, confidence: f32, folder: &str| Skill {
        name: name.to_string(),
        confidence,
        source: SkillSource::FolderName(folder.to_string()),
    };

    // ── Component-driven architecture ─────────────────────────────────────────
    if has_any(&folders, &["components", "ui", "widgets"]) {
        let folder = first_match(&folders, &["components", "ui", "widgets"]).unwrap();
        skills.push(skill("Component Architecture", 0.85, folder));
    }

    // ── React patterns ────────────────────────────────────────────────────────
    if has_any(&folders, &["hooks"]) {
        skills.push(skill("React Hooks Pattern", 0.80, "hooks"));
        // hooks folder strongly implies React
        skills.push(skill("React", 0.75, "hooks"));
    }

    // ── Service / API abstraction ─────────────────────────────────────────────
    if has_any(&folders, &["services", "service"]) {
        skills.push(skill("Service Layer Architecture", 0.82, "services"));
    }
    if has_any(&folders, &["api"]) {
        skills.push(skill("API Layer", 0.78, "api"));
    }

    // ── State management ──────────────────────────────────────────────────────
    if has_any(&folders, &["store", "stores", "state"]) {
        let folder = first_match(&folders, &["store", "stores", "state"]).unwrap();
        skills.push(skill("State Management Architecture", 0.80, folder));
    }

    // ── Routing ───────────────────────────────────────────────────────────────
    if has_any(&folders, &["pages", "routes", "app"]) {
        let folder = first_match(&folders, &["pages", "routes", "app"]).unwrap();
        skills.push(skill("Routing System", 0.75, folder));
    }

    // ── Shared utilities ──────────────────────────────────────────────────────
    if has_any(&folders, &["lib", "utils", "helpers", "shared"]) {
        let folder = first_match(&folders, &["lib", "utils", "helpers", "shared"]).unwrap();
        skills.push(skill("Shared Utility Layer", 0.70, folder));
    }

    // ── Testing ───────────────────────────────────────────────────────────────
    if has_any(&folders, &["tests", "__tests__", "spec", "test"]) {
        let folder = first_match(&folders, &["tests", "__tests__", "spec", "test"]).unwrap();
        skills.push(skill("Testing", 0.80, folder));
    }

    // ── Database layer ────────────────────────────────────────────────────────
    if has_any(&folders, &["prisma"]) {
        skills.push(skill("Database ORM (Prisma)", 0.90, "prisma"));
        skills.push(skill("Database", 0.90, "prisma"));
    }
    if has_any(&folders, &["migrations", "migration"]) {
        let folder = first_match(&folders, &["migrations", "migration"]).unwrap();
        skills.push(skill("Database Migrations", 0.85, folder));
        skills.push(skill("Database", 0.80, folder));
    }
    if has_any(&folders, &["db", "database"]) {
        let folder = first_match(&folders, &["db", "database"]).unwrap();
        skills.push(skill("Database Layer", 0.78, folder));
        skills.push(skill("Database", 0.78, folder));
    }

    // ── Monorepo indicators ───────────────────────────────────────────────────
    if has_any(&folders, &["packages", "apps"]) {
        let folder = first_match(&folders, &["packages", "apps"]).unwrap();
        skills.push(Skill {
            name: "__has_monorepo".to_string(),
            confidence: 0.90,
            source: SkillSource::FolderName(folder.to_string()),
        });
    }

    // ── Docker / CI indicators ────────────────────────────────────────────────
    if has_any(&folders, &[".github"]) {
        skills.push(Skill {
            name: "__has_ci".to_string(),
            confidence: 0.90,
            source: src(".github"),
        });
    }

    // ── Architecture style inference ──────────────────────────────────────────
    let arch_style = infer_arch_style(&folders);

    (skills, arch_style)
}

/// Infer the overall architecture style from folder presence.
fn infer_arch_style(folders: &[&str]) -> Option<ArchStyle> {
    // Feature-based: `modules/` or `features/` at top level
    if has_any(folders, &["modules", "features"]) {
        return Some(ArchStyle::FeatureBased);
    }

    // Layer-based: classic separation of concerns
    let layer_indicators = ["components", "services", "hooks", "utils", "lib", "pages"];
    let layer_count = layer_indicators.iter().filter(|&&l| folders.contains(&l)).count();
    if layer_count >= 2 {
        return Some(ArchStyle::LayerBased);
    }

    // Flat: src/ or very few top-level folders
    if has_any(folders, &["src"]) && layer_count == 0 {
        return Some(ArchStyle::Flat);
    }

    None
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn has_any(folders: &[&str], targets: &[&str]) -> bool {
    targets.iter().any(|t| folders.contains(t))
}

fn first_match<'a>(folders: &[&'a str], targets: &[&str]) -> Option<&'a str> {
    targets.iter().find_map(|t| folders.iter().find(|&&f| f == *t).copied())
}

fn src(folder: &str) -> SkillSource {
    SkillSource::FolderName(folder.to_string())
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn folder_map(names: &[&str]) -> FolderMap {
        names.iter().map(|&n| (n.to_string(), vec![])).collect::<HashMap<_, _>>()
    }

    fn has_skill(skills: &[Skill], name: &str) -> bool {
        skills.iter().any(|s| s.name.contains(name))
    }

    #[test]
    fn detects_layer_based_from_components_and_hooks() {
        let fm = folder_map(&["components", "hooks", "services", "pages"]);
        let (skills, arch) = detect_from_folders(&fm);

        assert!(has_skill(&skills, "Component Architecture"));
        assert!(has_skill(&skills, "React Hooks Pattern"));
        assert!(has_skill(&skills, "Service Layer"));
        assert!(matches!(arch, Some(ArchStyle::LayerBased)));
    }

    #[test]
    fn detects_feature_based_from_modules() {
        let fm = folder_map(&["modules", "lib"]);
        let (_, arch) = detect_from_folders(&fm);
        assert!(matches!(arch, Some(ArchStyle::FeatureBased)));
    }

    #[test]
    fn detects_prisma_folder() {
        let fm = folder_map(&["prisma", "src"]);
        let (skills, _) = detect_from_folders(&fm);
        assert!(has_skill(&skills, "Prisma"));
        assert!(has_skill(&skills, "Database"));
    }

    #[test]
    fn detects_monorepo_flag() {
        let fm = folder_map(&["packages", "apps", "docs"]);
        let (skills, _) = detect_from_folders(&fm);
        assert!(has_skill(&skills, "__has_monorepo"));
    }

    #[test]
    fn empty_folder_map_returns_no_skills() {
        let fm = folder_map(&[]);
        let (skills, arch) = detect_from_folders(&fm);
        assert!(skills.is_empty());
        assert!(arch.is_none());
    }

    #[test]
    fn detects_testing_folder() {
        let fm = folder_map(&["__tests__", "src"]);
        let (skills, _) = detect_from_folders(&fm);
        assert!(has_skill(&skills, "Testing"));
    }
}