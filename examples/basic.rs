use cfl::{CflBuilder, Result};
use std::path::Path;

fn main() -> Result<()> {
    // Basic usage
    let content = cfl::copy_files("src/")?;
    println!("Basic usage result:\n{}", content);

    // Using builder pattern
    let mut processor = CflBuilder::new()
        .include_patterns("*.rs")
        .exclude_patterns("*_test.rs")
        .current_dir(Path::new("src/"))
        .build()?;

    processor.process_path(Path::new("src/"))?;

    println!("\nDetailed information:");
    for file in processor.get_target_files() {
        println!(
            "File: {} (size: {}, tokens: {})",
            file.path, file.size, file.tokens
        );
    }

    println!("\nDirectory structure:");
    println!("{}", processor.get_directory_structure()?);

    Ok(())
}
