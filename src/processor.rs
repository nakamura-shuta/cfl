use crate::error::CflError;
use anyhow::Result;
use glob::Pattern;
use ignore::WalkBuilder;
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

/// FileProcessor handles the core functionality of processing and copying files
#[derive(Debug)]
pub struct FileProcessor {
    include_patterns: Vec<Pattern>,
    exclude_patterns: Vec<Pattern>,
    processed_paths: HashSet<PathBuf>,
    target_files: Vec<FileInfo>,
    result: String,
    current_dir: PathBuf,
}

/// Information about a processed file
#[derive(Clone, Debug)]
pub struct FileInfo {
    /// Relative path of the file
    pub path: String,
    /// Size of the file in bytes
    pub size: usize,
    /// Estimated number of tokens in the file
    pub tokens: usize,
}

impl FileProcessor {
    /// Creates a new FileProcessor instance
    pub(crate) fn new(
        include: &Option<String>,
        exclude: &Option<String>,
        current_dir: &Path,
    ) -> Result<Self> {
        let include_patterns = match include {
            Some(patterns) => patterns
                .split(',')
                .map(Pattern::new)
                .collect::<Result<Vec<_>, _>>()
                .map_err(CflError::from)?,
            None => Vec::new(),
        };

        let exclude_patterns = match exclude {
            Some(patterns) => patterns
                .split(',')
                .map(Pattern::new)
                .collect::<Result<Vec<_>, _>>()
                .map_err(CflError::from)?,
            None => Vec::new(),
        };

        Ok(Self {
            include_patterns,
            exclude_patterns,
            processed_paths: HashSet::new(),
            target_files: Vec::new(),
            result: String::new(),
            current_dir: current_dir.to_path_buf(),
        })
    }

    /// Process files in the specified path
    ///
    /// # Arguments
    ///
    /// * `path` - The path to process (file or directory)
    ///
    /// # Returns
    ///
    /// * `Result<()>` - Success or error
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use cfl::CflBuilder;
    /// use std::path::Path;
    ///
    /// # fn main() -> anyhow::Result<()> {
    /// let mut processor = CflBuilder::new()
    ///     .include_patterns("*.rs")
    ///     .build()?;
    ///
    /// processor.process_path(Path::new("src/"))?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn process_path(&mut self, path: &Path) -> Result<()> {
        if !path.exists() {
            return Err(CflError::PathNotFound(path.display().to_string()).into());
        }

        let walker = WalkBuilder::new(path)
            .hidden(false)
            .git_ignore(true)
            .git_global(true)
            .ignore(true)
            .build();

        for result in walker {
            match result {
                Ok(entry) => {
                    if entry.file_type().map_or(false, |ft| ft.is_file()) {
                        self.process_file(entry.path())?;
                    }
                }
                Err(err) => {
                    eprintln!("Error walking directory: {}", err);
                }
            }
        }

        Ok(())
    }

    /// Process a single file
    fn process_file(&mut self, path: &Path) -> Result<()> {
        let canonical_path = fs::canonicalize(path)?;
        if self.processed_paths.contains(&canonical_path) {
            return Ok(());
        }

        let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");

        if self
            .exclude_patterns
            .iter()
            .any(|pattern| pattern.matches(file_name))
        {
            return Ok(());
        }

        if !self.include_patterns.is_empty()
            && !self
                .include_patterns
                .iter()
                .any(|pattern| pattern.matches(file_name))
        {
            return Ok(());
        }

        let content = fs::read_to_string(path)?;
        let relative_path = path
            .strip_prefix(&self.current_dir)
            .unwrap_or(path)
            .to_string_lossy()
            .to_string();

        let size = content.len();
        let tokens = self.estimate_tokens(&content);

        self.target_files.push(FileInfo {
            path: relative_path.clone(),
            size,
            tokens,
        });

        self.result
            .push_str(&format!("```{}\n{}\n```\n", relative_path, content));
        self.processed_paths.insert(canonical_path);

        Ok(())
    }

    /// Estimate the number of tokens in a string
    fn estimate_tokens(&self, content: &str) -> usize {
        content
            .split(|c: char| {
                c.is_whitespace()
                    || c.is_ascii_punctuation()
                    || matches!(c, '(' | ')' | '[' | ']' | '{' | '}')
                    || matches!(
                        c,
                        '+' | '-'
                            | '*'
                            | '/'
                            | '='
                            | '<'
                            | '>'
                            | '&'
                            | '|'
                            | '!'
                            | '@'
                            | '#'
                            | '$'
                            | '%'
                            | '^'
                    )
            })
            .filter(|s| !s.is_empty())
            .count()
    }

    /// Get information about all processed files
    ///
    /// # Returns
    ///
    /// A slice containing information about each processed file
    pub fn get_target_files(&self) -> &[FileInfo] {
        &self.target_files
    }

    /// Get the formatted result string containing all file contents
    ///
    /// # Returns
    ///
    /// A string containing all file contents formatted with markdown code blocks
    pub fn get_result(&self) -> &str {
        &self.result
    }

    /// Get the total size of all processed files in bytes
    ///
    /// # Returns
    ///
    /// The total size in bytes
    pub fn get_total_size(&self) -> usize {
        self.result.len()
    }

    /// Get the total number of tokens across all processed files
    ///
    /// # Returns
    ///
    /// The total number of tokens
    pub fn get_total_tokens(&self) -> usize {
        self.target_files.iter().map(|f| f.tokens).sum()
    }

    /// Get a string representation of the directory structure
    ///
    /// # Returns
    ///
    /// A formatted string showing the directory structure
    pub fn get_directory_structure(&self) -> Result<String> {
        let mut result = String::new();
        self.build_directory_structure(&self.current_dir, 0, &mut result)?;
        Ok(result)
    }

    fn build_directory_structure(
        &self,
        path: &Path,
        _depth: usize,
        output: &mut String,
    ) -> Result<()> {
        let walker = WalkBuilder::new(path)
            .hidden(false)
            .git_ignore(true)
            .git_global(true)
            .ignore(true)
            .build();

        // エントリを収集
        let entries: Vec<_> = walker
            .filter_map(Result::ok)
            .filter(|entry| {
                let path = entry.path();
                !path.to_string_lossy().contains("/.git/")
                    && path
                        .file_name()
                        .and_then(|n| n.to_str())
                        .map(|n| n != ".git" && n != ".gitignore")
                        .unwrap_or(false)
            })
            .collect();

        // ディレクトリ構造をツリー形式で構築
        let mut tree: std::collections::BTreeMap<PathBuf, bool> = std::collections::BTreeMap::new();

        for entry in entries {
            if let Ok(relative) = entry.path().strip_prefix(path) {
                if relative.as_os_str().is_empty() {
                    continue;
                }

                // 親ディレクトリをすべて追加
                let mut current = PathBuf::new();
                for component in relative.components() {
                    current.push(component);
                    if !tree.contains_key(&current) {
                        let is_dir = if current == entry.path().strip_prefix(path).unwrap() {
                            entry.file_type().map_or(false, |ft| ft.is_dir())
                        } else {
                            true
                        };
                        tree.insert(current.clone(), is_dir);
                    }
                }
            }
        }

        // ツリーを表示
        for (path, is_dir) in tree {
            let depth = path.components().count();
            let indent = "  ".repeat(depth.saturating_sub(1));
            let name = path.file_name().unwrap_or_default().to_string_lossy();

            if is_dir {
                output.push_str(&format!("{}└── {}/\n", indent, name));
            } else {
                output.push_str(&format!("{}└── {}\n", indent, name));
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn setup_test_dir() -> TempDir {
        let temp_dir = TempDir::new().unwrap();
        fs::write(
            temp_dir.path().join("test.rs"),
            "fn test() { println!(\"test\"); }",
        )
        .unwrap();
        temp_dir
    }

    #[test]
    fn test_file_processing() {
        let temp_dir = setup_test_dir();
        let mut processor = FileProcessor::new(&None, &None, temp_dir.path()).unwrap();

        processor.process_path(temp_dir.path()).unwrap();
        assert!(!processor.get_result().is_empty());
    }
}
