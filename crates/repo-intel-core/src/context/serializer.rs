use crate::{error::Error, types::RepoContext};

/// Serialize a `RepoContext` to a pretty-printed JSON string.
pub fn to_json(ctx: &RepoContext) -> Result<String, Error> {
    let json = serde_json::to_string_pretty(ctx)?;
    Ok(json)
}

/// Serialize a `RepoContext` to compact JSON (for bandwidth-sensitive uses).
pub fn to_json_compact(ctx: &RepoContext) -> Result<String, Error> {
    let json = serde_json::to_string(ctx)?;
    Ok(json)
}

/// Write context JSON to a file path.
pub fn write_to_file(ctx: &RepoContext, path: &std::path::Path) -> Result<(), Error> {
    let json = to_json(ctx)?;
    std::fs::write(path, json)
        .map_err(|e| Error::ScanIo(path.display().to_string(), e))?;
    Ok(())
}
