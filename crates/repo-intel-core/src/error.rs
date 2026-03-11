use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("failed to read config file '{0}': {1}")]
    ConfigRead(String, #[source] std::io::Error),

    #[error("failed to parse config file '{0}': {1}")]
    ConfigParse(String, String),

    #[error("failed to scan directory '{0}': {1}")]
    ScanIo(String, #[source] std::io::Error),

    #[error("failed to read signal file '{0}': {1}")]
    SignalRead(String, #[source] std::io::Error),

    #[error("failed to serialize context to JSON: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("walkdir error: {0}")]
    WalkDir(#[from] walkdir::Error),
}
