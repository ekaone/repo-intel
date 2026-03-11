use crate::{config::Config, error::Error, types::ScanResult};
use std::path::Path;

pub mod signal;
pub mod walker;

/// Scan the repository at `root` using the provided `cfg` and return a `ScanResult`.
pub fn scan(root: &Path, cfg: &Config) -> Result<ScanResult, Error> {
    let folder_map = walker::build_folder_map(root, cfg)?;
    let signal_files = signal::detect_signals(root, &folder_map)?;
    let extensions = collect_extensions(&folder_map);

    Ok(ScanResult {
        root: root.to_path_buf(),
        folder_map,
        signal_files,
        extensions,
    })
}

fn collect_extensions(folder_map: &std::collections::HashMap<String, Vec<String>>) -> Vec<String> {
    let mut exts: std::collections::HashSet<String> = std::collections::HashSet::new();
    for files in folder_map.values() {
        for file in files {
            if let Some(ext) = std::path::Path::new(file).extension() {
                exts.insert(ext.to_string_lossy().to_lowercase());
            }
        }
    }
    let mut result: Vec<String> = exts.into_iter().collect();
    result.sort();
    result
}
