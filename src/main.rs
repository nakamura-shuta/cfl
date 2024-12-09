use std::path::{Path, PathBuf};
use std::fs;
use clap::Parser;
use clipboard::{ClipboardContext, ClipboardProvider};
use glob::Pattern;

#[derive(Parser)]
#[command(name = "cfl")]
#[command(about = "Copy file contents for LLM")]
struct Cli {
    /// Paths to copy (comma-separated)
    paths: String,

    /// Include patterns (comma-separated)
    #[arg(short, long)]
    include: Option<String>,

    /// Exclude patterns (comma-separated)
    #[arg(short, long)]
    exclude: Option<String>,
}

fn main() {
    let cli = Cli::parse();
    let paths: Vec<&str> = cli.paths.split(',').collect();
    
    // Convert include/exclude patterns
    let include_patterns: Vec<Pattern> = cli.include
        .map(|s| s.split(',').map(|p| Pattern::new(p).unwrap()).collect())
        .unwrap_or_default();
    
    let exclude_patterns: Vec<Pattern> = cli.exclude
        .map(|s| s.split(',').map(|p| Pattern::new(p).unwrap()).collect())
        .unwrap_or_default();

    let mut processed_paths = std::collections::HashSet::new();
    let mut result = String::new();

    for path in paths {
        process_path(
            Path::new(path),
            &mut processed_paths,
            &mut result,
            &include_patterns,
            &exclude_patterns,
        );
    }

    // Copy to clipboard
    let mut ctx: ClipboardContext = ClipboardProvider::new().unwrap();
    ctx.set_contents(result).unwrap();
}

fn process_path(
    path: &Path,
    processed_paths: &mut std::collections::HashSet<PathBuf>,
    result: &mut String,
    include_patterns: &[Pattern],
    exclude_patterns: &[Pattern],
) {
    if !path.exists() {
        eprintln!("Path does not exist: {}", path.display());
        return;
    }

    if path.is_file() {
        process_file(path, processed_paths, result, include_patterns, exclude_patterns);
    } else if path.is_dir() {
        if let Ok(entries) = fs::read_dir(path) {
            for entry in entries.flatten() {
                process_path(
                    &entry.path(),
                    processed_paths,
                    result,
                    include_patterns,
                    exclude_patterns,
                );
            }
        }
    }
}

fn process_file(
    path: &Path,
    processed_paths: &mut std::collections::HashSet<PathBuf>,
    result: &mut String,
    include_patterns: &[Pattern],
    exclude_patterns: &[Pattern],
) {
    let canonical_path = fs::canonicalize(path).unwrap();
    if processed_paths.contains(&canonical_path) {
        return;
    }

    let file_name = path.file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("");

    // Check exclude patterns first
    if exclude_patterns.iter().any(|pattern| pattern.matches(file_name)) {
        return;
    }

    // Then check include patterns if they exist
    if !include_patterns.is_empty() && 
       !include_patterns.iter().any(|pattern| pattern.matches(file_name)) {
        return;
    }

    if let Ok(content) = fs::read_to_string(path) {
        // 修正: ```で囲む形式に変更
        result.push_str(&format!("```{}\n{}\n```\n", path.display(), content));
        processed_paths.insert(canonical_path);
    }
}