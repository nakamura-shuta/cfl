use thiserror::Error;

/// Errors that can occur during file processing
#[derive(Error, Debug)]
pub enum CflError {
    /// IO-related errors
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Pattern parsing errors
    #[error("Pattern error: {0}")]
    Pattern(#[from] glob::PatternError),

    /// Clipboard-related errors
    #[error("Clipboard error: {0}")]
    Clipboard(String),

    /// Path not found errors
    #[error("Path not found: {0}")]
    PathNotFound(String),
}
