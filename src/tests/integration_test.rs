// src/tests/integration_test.rs
use cfl::{copy_files, copy_files_with_patterns};
use std::fs;
use tempfile::TempDir;

fn create_test_files() -> TempDir {
    let temp_dir = TempDir::new().unwrap();
    
    fs::write(
        temp_dir.path().join("main.rs"),
        "fn main() {}"
    ).unwrap();
    
    fs::write(
        temp_dir.path().join("lib.rs"),
        "pub fn test() {}"
    ).unwrap();
    
    fs::write(
        temp_dir.path().join("config.json"),
        "{}"
    ).unwrap();

    temp_dir
}

#[test]
fn test_copy_files() {
    let temp_dir = create_test_files();
    let result = copy_files(temp_dir.path()).unwrap();
    
    assert!(result.contains("main.rs"));
    assert!(result.contains("lib.rs"));
    assert!(result.contains("config.json"));
}

#[test]
fn test_copy_files_with_patterns() {
    let temp_dir = create_test_files();
    let result = copy_files_with_patterns(
        temp_dir.path(),
        Some("*.rs".to_string()),
        None,
    ).unwrap();
    
    assert!(result.contains("main.rs"));
    assert!(result.contains("lib.rs"));
    assert!(!result.contains("config.json"));
}