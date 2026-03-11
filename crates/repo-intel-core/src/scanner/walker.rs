use crate::{config::Config, error::Error};
use std::collections::HashMap;
use std::path::Path;
use walkdir::WalkDir;

/// Directory names always skipped during traversal.
pub const ALWAYS_SKIP: &[&str] = &[".git", ".svn", ".hg"];

/// Build a map of `relative_dir_path → [filenames]` for the given root,
/// respecting the `skip_dirs` configuration.
pub fn build_folder_map(
    root: &Path,
    cfg: &Config,
) -> Result<HashMap<String, Vec<String>>, Error> {
    let mut folder_map: HashMap<String, Vec<String>> = HashMap::new();
    let skip_dirs = &cfg.scan.skip_dirs;

    let walker = WalkDir::new(root).follow_links(false).into_iter();

    for entry in walker.filter_entry(|e| should_visit(e, skip_dirs)) {
        let entry = entry?;

        if entry.file_type().is_file() {
            let path = entry.path();
            let rel = path.strip_prefix(root).unwrap_or(path);
            let parent = rel
                .parent()
                .map(|p| p.to_string_lossy().replace('\\', "/"))
                .unwrap_or_default();
            let filename = path
                .file_name()
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_default();

            folder_map.entry(parent).or_default().push(filename);
        }
    }

    Ok(folder_map)
}

fn should_visit(entry: &walkdir::DirEntry, skip_dirs: &[String]) -> bool {
    if entry.file_type().is_dir() {
        let name = entry.file_name().to_string_lossy();
        if ALWAYS_SKIP.iter().any(|s| *s == name.as_ref()) {
            return false;
        }
        if skip_dirs.iter().any(|s| s == name.as_ref()) {
            return false;
        }
    }
    true
}
