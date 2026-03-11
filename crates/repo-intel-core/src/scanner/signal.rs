use crate::error::Error;
use std::collections::HashMap;
use std::path::Path;

/// File names that carry dependency / stack information.
const SIGNAL_FILENAMES: &[&str] = &[
    "package.json",
    "Cargo.toml",
    "go.mod",
    "requirements.txt",
    "pyproject.toml",
    "Pipfile",
    "pom.xml",
    "build.gradle",
    "build.gradle.kts",
    "composer.json",
    "Gemfile",
    "mix.exs",
    "pubspec.yaml",
    "deno.json",
    "bun.lockb",
    "docker-compose.yml",
    "docker-compose.yaml",
    "Dockerfile",
    ".github/workflows",
    "turbo.json",
    "nx.json",
    "lerna.json",
    "pnpm-workspace.yaml",
];

/// Detect signal files present in the folder_map and read their contents.
pub fn detect_signals(
    root: &Path,
    folder_map: &HashMap<String, Vec<String>>,
) -> Result<HashMap<String, String>, Error> {
    let mut signals: HashMap<String, String> = HashMap::new();

    for (dir, files) in folder_map {
        for file in files {
            let rel_path = if dir.is_empty() {
                file.clone()
            } else {
                format!("{dir}/{file}")
            };

            if SIGNAL_FILENAMES
                .iter()
                .any(|sig| rel_path == *sig || file == sig.split('/').last().unwrap_or(sig))
            {
                let abs = root.join(&rel_path);
                match std::fs::read_to_string(&abs) {
                    Ok(contents) => {
                        signals.insert(rel_path, contents);
                    }
                    Err(e) => {
                        // Non-UTF8 or binary — log and skip
                        eprintln!("warn: could not read signal file {rel_path}: {e}");
                    }
                }
            }
        }
    }

    Ok(signals)
}
