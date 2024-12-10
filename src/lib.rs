//! # cfl
//!
//! A library for copying and formatting file contents for LLM processing.
//!
//! ## Features
//!
//! - Copy multiple files with proper markdown code block formatting
//! - Support for include/exclude patterns
//! - Respect .gitignore rules
//! - Display file statistics
//!
//! ## Example
//!
//! ```rust,no_run
//! use cfl::{CflBuilder, Result};
//! use std::path::Path;
//!
//! fn main() -> Result<()> {
//!     let mut processor = CflBuilder::new()
//!         .include_patterns("*.rs")
//!         .build()?;
//!     
//!     processor.process_path(Path::new("src/"))?;
//!     let content = processor.get_result();
//!     println!("Copied content:\n{}", content);
//!     
//!     Ok(())
//! }
//! ```

pub mod cli;
pub mod error;
pub mod processor;

pub use anyhow::Result;
pub use error::CflError;
pub use processor::{FileInfo, FileProcessor};

use std::path::{Path, PathBuf};

/// Builder pattern for FileProcessor configuration
pub struct CflBuilder {
    include_patterns: Option<String>,
    exclude_patterns: Option<String>,
    current_dir: PathBuf,
}

impl Default for CflBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl CflBuilder {
    pub fn new() -> Self {
        Self {
            include_patterns: None,
            exclude_patterns: None,
            current_dir: std::env::current_dir().unwrap_or_default(),
        }
    }

    pub fn include_patterns<S: Into<String>>(mut self, patterns: S) -> Self {
        self.include_patterns = Some(patterns.into());
        self
    }

    pub fn exclude_patterns<S: Into<String>>(mut self, patterns: S) -> Self {
        self.exclude_patterns = Some(patterns.into());
        self
    }

    pub fn current_dir<P: AsRef<Path>>(mut self, path: P) -> Self {
        self.current_dir = path.as_ref().to_path_buf();
        self
    }

    pub fn build(self) -> Result<FileProcessor> {
        FileProcessor::new(
            &self.include_patterns,
            &self.exclude_patterns,
            &self.current_dir,
        )
    }
}

/// High-level convenience functions
pub fn copy_files<P: AsRef<Path>>(path: P) -> Result<String> {
    let mut processor = CflBuilder::new().current_dir(path.as_ref()).build()?;

    processor.process_path(path.as_ref())?;
    Ok(processor.get_result().to_string())
}

pub fn copy_files_with_patterns<P: AsRef<Path>>(
    path: P,
    include: Option<String>,
    exclude: Option<String>,
) -> Result<String> {
    let mut processor = CflBuilder::new()
        .current_dir(path.as_ref())
        .include_patterns(include.unwrap_or_default())
        .exclude_patterns(exclude.unwrap_or_default())
        .build()?;

    processor.process_path(path.as_ref())?;
    Ok(processor.get_result().to_string())
}
