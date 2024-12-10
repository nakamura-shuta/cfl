// src/tests/builder_test.rs
use cfl::CflBuilder;
use std::fs;
use tempfile::TempDir;

fn setup_test_directory() -> TempDir {
    let temp_dir = TempDir::new().unwrap();
    
    // Create test files
    fs::create_dir_all(temp_dir.path().join("src")).unwrap();
    fs::create_dir_all(temp_dir.path().join("tests")).unwrap();
    
    fs::write(
        temp_dir.path().join("src/main.rs"),
        "fn main() { println!(\"Hello\"); }"
    ).unwrap();
    
    fs::write(
        temp_dir.path().join("src/lib.rs"),
        "pub fn add(a: i32, b: i32) -> i32 { a + b }"
    ).unwrap();
    
    fs::write(
        temp_dir.path().join("tests/test.rs"),
        "#[test] fn test_add() { assert_eq!(2 + 2, 4); }"
    ).unwrap();
    
    fs::write(
        temp_dir.path().join("Cargo.toml"),
        r#"[package]
name = "test"
version = "0.1.0"
"#
    ).unwrap();

    temp_dir
}

#[test]
fn test_builder_basic() {
    let temp_dir = setup_test_directory();
    let processor = CflBuilder::new()
        .current_dir(temp_dir.path())
        .build()
        .unwrap();

    assert!(processor.get_result().is_empty());
}

#[test]
fn test_builder_with_patterns() {
    let temp_dir = setup_test_directory();
    let mut processor = CflBuilder::new()
        .include_patterns("*.rs")
        .exclude_patterns("test.rs")
        .current_dir(temp_dir.path())
        .build()
        .unwrap();

    processor.process_path(temp_dir.path()).unwrap();
    let files = processor.get_target_files();

    assert_eq!(files.len(), 2);
    assert!(files.iter().any(|f| f.path.contains("main.rs")));
    assert!(files.iter().any(|f| f.path.contains("lib.rs")));
    assert!(!files.iter().any(|f| f.path.contains("test.rs")));
}

#[test]
fn test_builder_directory_structure() {
    let temp_dir = setup_test_directory();
    let processor = CflBuilder::new()
        .current_dir(temp_dir.path())
        .build()
        .unwrap();

    let structure = processor.get_directory_structure().unwrap();
    
    assert!(structure.contains("src/"));
    assert!(structure.contains("tests/"));
    assert!(structure.contains("Cargo.toml"));
}