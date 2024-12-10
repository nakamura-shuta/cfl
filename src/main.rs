mod cli;
mod error;
mod processor;

#[cfg(test)]
mod tests;

use anyhow::{Context, Result};
use clap::Parser;
use cli::Cli;
use clipboard::{ClipboardContext, ClipboardProvider};
use error::CflError;

fn format_number(num: usize) -> String {
    num.to_string()
        .chars()
        .rev()
        .collect::<Vec<_>>()
        .chunks(3)
        .map(|chunk| chunk.iter().collect::<String>())
        .collect::<Vec<_>>()
        .join(",")
        .chars()
        .rev()
        .collect()
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let current_dir = std::env::current_dir().context("Failed to get current directory")?;

    let mut processor = processor::FileProcessor::new(&cli.include, &cli.exclude, &current_dir)?;

    for path in cli.paths.split(',') {
        processor
            .process_path(std::path::Path::new(path))
            .with_context(|| format!("Failed to process path: {}", path))?;
    }

    let target_files = processor.get_target_files();
    let files_count = target_files.len();

    if cli.show {
        println!("Target files:");
        for file in target_files {
            println!(
                "  {} ({} bytes, {} tokens)",
                file.path,
                format_number(file.size),
                format_number(file.tokens)
            );
        }
        println!("\nTotal: {} files", format_number(files_count));
    } else {
        let mut ctx: ClipboardContext =
            ClipboardProvider::new().map_err(|e| CflError::Clipboard(e.to_string()))?;

        ctx.set_contents(processor.get_result().to_string())
            .map_err(|e| CflError::Clipboard(e.to_string()))?;

        println!(
            "\n✨ Successfully copied {} files to clipboard:",
            files_count
        );
        println!("📁 Files:");
        for file in target_files {
            println!(
                "  • {} ({} bytes, {} tokens)",
                file.path,
                format_number(file.size),
                format_number(file.tokens)
            );
        }

        let total_size = processor.get_total_size();
        let total_tokens = processor.get_total_tokens();

        println!("\n📊 Summary:");
        println!("  • Total files: {}", format_number(files_count));
        println!("  • Total size: {} bytes", format_number(total_size));
        println!("  • Total tokens: {}", format_number(total_tokens));

        println!("\n📁 Directory Structure:");
        let structure = processor.get_directory_structure()?;
        println!("{}", structure);

        if let Some(include) = &cli.include {
            println!("  • Include patterns: {}", include);
        }
        if let Some(exclude) = &cli.exclude {
            println!("  • Exclude patterns: {}", exclude);
        }

        if files_count == 0 {
            println!("\n⚠️  No files were copied. Check your include/exclude patterns.");
        }
    }

    Ok(())
}
