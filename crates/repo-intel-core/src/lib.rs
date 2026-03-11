/// repo-intel-core — library root
///
/// Public API re-exports for use by the CLI binary and integration tests.
pub mod config;
pub mod context;
pub mod detector;
pub mod error;
pub mod scanner;
pub mod types;

// Convenience re-exports
pub use error::Error;
pub use types::{RepoContext, ScanResult, Skill, StackResult};
