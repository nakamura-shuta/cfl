use std::fs;
use tempfile::TempDir;

use crate::processor::FileProcessor;

fn setup_test_directory() -> TempDir {
    let temp_dir = TempDir::new().unwrap();
    
    // テスト用のディレクトリ構造を作成
    fs::create_dir(temp_dir.path().join("src")).unwrap();
    
    // テスト用のファイルを作成
    fs::write(
        temp_dir.path().join("src").join("main.rs"),
        "fn main() { println!(\"Hello\"); }"
    ).unwrap();
    
    fs::write(
        temp_dir.path().join("src").join("test.rs"),
        "#[test] fn test() { assert!(true); }"
    ).unwrap();
    
    fs::write(
        temp_dir.path().join("config.json"),
        "{\"key\": \"value\"}"
    ).unwrap();

    // .gitignoreファイルを作成
    fs::write(
        temp_dir.path().join(".gitignore"),
        "*.json\n"
    ).unwrap();

    // .git ディレクトリを作成（gitリポジトリをシミュレート）
    fs::create_dir(temp_dir.path().join(".git")).unwrap();

    temp_dir
}

#[test]
fn test_basic_file_processing() {
    let temp_dir = setup_test_directory();
    let processor = FileProcessor::new(
        &None,
        &None,
        temp_dir.path(),
    ).unwrap();

    assert!(processor.get_result().is_empty());
}

#[test]
fn test_include_pattern() {
    let temp_dir = setup_test_directory();
    let mut processor = FileProcessor::new(
        &Some("*.rs".to_string()),
        &None,
        temp_dir.path(),
    ).unwrap();

    processor.process_path(temp_dir.path()).unwrap();
    let files = processor.get_target_files();

    assert_eq!(files.len(), 2);
    assert!(files.iter().all(|f| f.path.ends_with(".rs")));
}

#[test]
fn test_exclude_pattern() {
    let temp_dir = setup_test_directory();
    let mut processor = FileProcessor::new(
        &None,
        &Some("test.rs".to_string()),
        temp_dir.path(),
    ).unwrap();

    processor.process_path(temp_dir.path()).unwrap();
    let files = processor.get_target_files();

    assert!(files.iter().any(|f| f.path == "src/main.rs"));
    assert!(!files.iter().any(|f| f.path == "src/test.rs"));
}

#[test]
fn test_gitignore_respect() {
    let temp_dir = setup_test_directory();
    let mut processor = FileProcessor::new(
        &None,
        &None,
        temp_dir.path(),
    ).unwrap();

    processor.process_path(temp_dir.path()).unwrap();
    let files = processor.get_target_files();

    // ファイルパスをデバッグ出力
    println!("Found files:");
    for file in files {
        println!("  {}", file.path);
    }

    assert!(!files.iter().any(|f| f.path.ends_with(".json")));
    assert!(files.iter().any(|f| f.path.contains("main.rs")));
}

#[test]
fn test_token_counting() {
    let temp_dir = setup_test_directory();
    let mut processor = FileProcessor::new(
        &Some("**/main.rs".to_string()),
        &None,
        temp_dir.path(),
    ).unwrap();

    processor.process_path(temp_dir.path()).unwrap();
    let files = processor.get_target_files();

    assert_eq!(files.len(), 1);
    let tokens = files[0].tokens;
    assert!(tokens > 0, "Expected non-zero tokens, got {}", tokens);
}

#[test]
fn test_directory_structure() {
    let temp_dir = setup_test_directory();
    let processor = FileProcessor::new(
        &None,
        &None,
        temp_dir.path(),
    ).unwrap();

    let structure = processor.get_directory_structure().unwrap();
    println!("Directory structure:\n{}", structure);
    
    assert!(structure.contains("src"));
    assert!(structure.contains("main.rs"));
    assert!(structure.contains("test.rs"));
    assert!(!structure.contains("config.json"));
    assert!(!structure.contains(".git"));
    assert!(!structure.contains(".gitignore"));
}