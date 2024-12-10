use clap::{command, Parser};

#[derive(Parser)]
#[command(
    author = "Shuta Nakamura",
    version,
    about = "Copy file contents for LLM processing",
    long_about = None,
    after_help = "Examples:
    # Copy all files in the current directory
    cfl .

    # Copy specific files or directories
    cfl src/main.rs
    cfl src/,tests/

    # Copy only Rust files
    cfl . -i \"*.rs\"

    # Copy both Rust and TOML files
    cfl . -i \"*.rs,*.toml\"

    # Copy all files except JSON files
    cfl . -e \"*.json\"

    # Copy all files except test files
    cfl . -e \"*_test.rs,test_*.rs\"

    # Copy only Rust files, but exclude test files
    cfl . -i \"*.rs\" -e \"*_test.rs\"

    # Show which files would be copied without copying
    cfl -s .
    
Note: .gitignore rules are automatically respected"
)]
pub struct Cli {
    /// Paths to copy (comma-separated)
    #[arg(name = "PATHS", help = "Paths to copy (comma-separated)")]
    pub paths: String,

    /// Include patterns (comma-separated)
    #[arg(
        short,
        long,
        help = "Include only files matching these patterns (comma-separated)",
        value_name = "PATTERNS"
    )]
    pub include: Option<String>,

    /// Exclude patterns (comma-separated)
    #[arg(
        short,
        long,
        help = "Exclude files matching these patterns (comma-separated)",
        value_name = "PATTERNS"
    )]
    pub exclude: Option<String>,

    /// Show target files (relative paths)
    #[arg(short, long, help = "Show which files would be copied without copying")]
    pub show: bool,
}
