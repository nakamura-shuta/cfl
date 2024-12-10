use cfl::{CflBuilder, FileProcessor, Result};
use std::path::Path;

fn process_directory(processor: &mut FileProcessor, path: &str) -> Result<()> {
    processor.process_path(Path::new(path))?;

    println!("📊 Statistics for {}:", path);
    println!("Files processed:");
    for file in processor.get_target_files() {
        println!(
            "  • {} ({} bytes, {} tokens)",
            file.path, file.size, file.tokens
        );
    }

    println!("\nTotal size: {} bytes", processor.get_total_size());
    println!("Total tokens: {}", processor.get_total_tokens());

    println!("\nDirectory structure:");
    println!("{}", processor.get_directory_structure()?);

    Ok(())
}

fn main() -> Result<()> {
    // Create a processor for Rust files, excluding tests
    let mut rust_processor = CflBuilder::new()
        .include_patterns("*.rs")
        .exclude_patterns("*_test.rs")
        .build()?;

    // Process source directory
    process_directory(&mut rust_processor, "src/")?;

    // Create another processor for configuration files
    let mut config_processor = CflBuilder::new()
        .include_patterns("*.toml,*.json")
        .build()?;

    // Process project root
    process_directory(&mut config_processor, ".")?;

    Ok(())
}
