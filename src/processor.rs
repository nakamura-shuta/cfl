use crate::error::CflError;
use anyhow::Result;
use glob::Pattern;
use ignore::WalkBuilder;
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

pub struct FileProcessor {
    include_patterns: Vec<Pattern>,
    exclude_patterns: Vec<Pattern>,
    processed_paths: HashSet<PathBuf>,
    target_files: Vec<FileInfo>,
    result: String,
    current_dir: PathBuf,
}

#[derive(Clone)]
pub struct FileInfo {
    pub path: String,
    pub size: usize,
    pub tokens: usize,
}

impl FileProcessor {
    pub fn new(
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

    pub fn process_path(&mut self, path: &Path) -> Result<()> {
        if !path.exists() {
            return Err(CflError::PathNotFound(path.display().to_string()).into());
        }

        let walker = WalkBuilder::new(path)
            .hidden(false) // 隠しファイルを含める
            .git_ignore(true) // .gitignoreを考慮
            .git_global(true) // グローバルな.gitignoreも考慮
            .ignore(true) // .ignoreファイルも考慮
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

    pub fn get_target_files(&self) -> &[FileInfo] {
        &self.target_files
    }

    pub fn get_result(&self) -> &str {
        &self.result
    }

    pub fn get_total_size(&self) -> usize {
        self.result.len()
    }

    pub fn get_total_tokens(&self) -> usize {
        self.target_files.iter().map(|f| f.tokens).sum()
    }

    pub fn get_directory_structure(&self) -> Result<String> {
        let mut result = String::new();
        self.build_directory_structure(&self.current_dir, 0, &mut result)?;
        Ok(result)
    }

    fn build_directory_structure(
        &self,
        path: &Path,
        depth: usize,
        output: &mut String,
    ) -> Result<()> {
        let walker = WalkBuilder::new(path)
            .hidden(false)
            .git_ignore(true)
            .git_global(true)
            .ignore(true)
            .build();

        let mut entries: Vec<_> = walker.filter_map(Result::ok).collect();
        entries.sort_by_key(|entry| entry.path().to_path_buf());

        let prefix = if depth == 0 {
            String::new()
        } else {
            "  ".repeat(depth)
        };

        for entry in entries {
            let path = entry.path();
            if let Ok(relative) = path.strip_prefix(&self.current_dir) {
                let is_dir = entry.file_type().map_or(false, |ft| ft.is_dir());
                let name = relative.file_name().and_then(|n| n.to_str()).unwrap_or("");

                if !name.is_empty() && name != ".git" && name != ".gitignore" {
                    if is_dir {
                        output.push_str(&format!("{}└── {}/\n", prefix, name));
                    } else {
                        output.push_str(&format!("{}└── {}\n", prefix, name));
                    }
                }
            }
        }

        Ok(())
    }
}
