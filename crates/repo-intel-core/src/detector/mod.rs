use crate::types::{ScanResult, Skill, StackResult};

pub mod deps;
pub mod folders;
pub mod patterns;

/// Run all detection layers on a `ScanResult` and return a `StackResult`.
pub fn detect(scan: &ScanResult) -> StackResult {
    let mut skills: Vec<Skill> = Vec::new();

    // Layer 1: dependency-file analysis
    skills.extend(deps::detect_from_deps(scan));

    // Layer 2: folder-name inference
    skills.extend(folders::detect_from_folders(scan));

    // Layer 3: filename pattern signals
    skills.extend(patterns::detect_from_patterns(scan));

    // Partition into languages / frameworks / tooling
    let languages = skills
        .iter()
        .filter(|s| s.category == "language")
        .cloned()
        .collect();
    let frameworks = skills
        .iter()
        .filter(|s| s.category == "framework")
        .cloned()
        .collect();
    let tooling = skills
        .iter()
        .filter(|s| s.category == "tooling")
        .cloned()
        .collect();

    let architecture = folders::infer_architecture(scan);

    StackResult {
        languages,
        frameworks,
        tooling,
        architecture,
    }
}
