use thiserror::Error;

#[derive(Error, Debug)]
pub enum CflError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Pattern error: {0}")]
    Pattern(#[from] glob::PatternError),

    #[error("Clipboard error: {0}")]
    Clipboard(String),

    #[error("Path not found: {0}")]
    PathNotFound(String),
}
