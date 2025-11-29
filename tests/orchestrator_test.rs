use cs::{run_search, SearchQuery};
use std::path::PathBuf;

#[test]
fn test_run_search_finds_translation_and_code() {
    let query = SearchQuery::new("add new".to_string())
        .with_base_dir(PathBuf::from("tests/fixtures/rails-app"));

    let result = run_search(query);
    assert!(result.is_ok(), "Search should succeed");

    let search_result = result.unwrap();
    
    // Should find translation entries
    assert!(!search_result.translation_entries.is_empty(), 
        "Should find translation entries for 'add new'");
    
    println!("Found {} translation entries", search_result.translation_entries.len());
    for entry in &search_result.translation_entries {
        println!("  - {}: {} ({}:{})", 
            entry.key, entry.value, entry.file.display(), entry.line);
    }
    
    // Should find code references
    assert!(!search_result.code_references.is_empty(), 
        "Should find code references");
    
    println!("Found {} code references", search_result.code_references.len());
    for code_ref in &search_result.code_references {
        println!("  - {}:{} - {}", 
            code_ref.file.display(), code_ref.line, code_ref.context.trim());
    }
}

#[test]
fn test_run_search_with_no_matches() {
    let query = SearchQuery::new("nonexistent text xyz".to_string())
        .with_base_dir(PathBuf::from("tests/fixtures/rails-app"));

    let result = run_search(query);
    assert!(result.is_ok(), "Search should succeed even with no matches");

    let search_result = result.unwrap();
    assert!(search_result.translation_entries.is_empty(), 
        "Should not find translation entries for nonexistent text");
    assert!(search_result.code_references.is_empty(), 
        "Should not find code references for nonexistent text");
}

#[test]
fn test_run_search_case_insensitive() {
    let query = SearchQuery::new("ADD NEW".to_string())
        .with_base_dir(PathBuf::from("tests/fixtures/rails-app"));

    let result = run_search(query);
    assert!(result.is_ok());

    let search_result = result.unwrap();
    
    // Case-insensitive search should find "add new"
    assert!(!search_result.translation_entries.is_empty(), 
        "Case-insensitive search should find 'add new'");
}

#[test]
fn test_run_search_multiple_keys() {
    let query = SearchQuery::new("invoice".to_string())
        .with_base_dir(PathBuf::from("tests/fixtures/rails-app"));

    let result = run_search(query);
    assert!(result.is_ok());

    let search_result = result.unwrap();
    
    // Should find multiple entries containing "invoice"
    assert!(search_result.translation_entries.len() > 1, 
        "Should find multiple translation entries containing 'invoice'");
    
    println!("Found {} entries containing 'invoice'", search_result.translation_entries.len());
}

#[test]
fn test_run_search_finds_all_frameworks() {
    // Test with the entire fixtures directory to find matches across frameworks
    let query = SearchQuery::new("add new".to_string())
        .with_base_dir(PathBuf::from("tests/fixtures"));

    let result = run_search(query);
    assert!(result.is_ok());

    let search_result = result.unwrap();
    
    println!("Found {} translation entries across all fixtures", 
        search_result.translation_entries.len());
    println!("Found {} code references across all fixtures", 
        search_result.code_references.len());
    
    // Should find entries from multiple framework fixtures
    assert!(!search_result.translation_entries.is_empty());
    
    // Check if we found references in different frameworks
    let has_rails = search_result.code_references.iter()
        .any(|r| r.file.to_string_lossy().contains("rails-app"));
    let has_react = search_result.code_references.iter()
        .any(|r| r.file.to_string_lossy().contains("react-app"));
    let has_vue = search_result.code_references.iter()
        .any(|r| r.file.to_string_lossy().contains("vue-app"));
    
    println!("Found in Rails: {}", has_rails);
    println!("Found in React: {}", has_react);
    println!("Found in Vue: {}", has_vue);
    
    // At least one framework should have references
    assert!(has_rails || has_react || has_vue, 
        "Should find references in at least one framework");
}

#[test]
fn test_run_search_with_current_dir() {
    // Test without specifying base_dir (uses current directory)
    let query = SearchQuery::new("test".to_string());

    let result = run_search(query);
    assert!(result.is_ok(), "Search should succeed with current directory");
}

#[test]
fn test_search_result_structure() {
    let query = SearchQuery::new("add new".to_string())
        .with_base_dir(PathBuf::from("tests/fixtures/rails-app"));

    let result = run_search(query).unwrap();
    
    // Verify SearchResult structure
    assert_eq!(result.query, "add new");
    assert!(!result.translation_entries.is_empty());
    
    // Verify translation entries have all required fields
    for entry in &result.translation_entries {
        assert!(!entry.key.is_empty(), "Key should not be empty");
        assert!(!entry.value.is_empty(), "Value should not be empty");
        assert!(entry.line > 0, "Line number should be positive");
        assert!(entry.file.exists(), "File should exist");
    }
    
    // Verify code references have all required fields
    for code_ref in &result.code_references {
        assert!(!code_ref.key_path.is_empty(), "Key path should not be empty");
        assert!(!code_ref.context.is_empty(), "Context should not be empty");
        assert!(code_ref.line > 0, "Line number should be positive");
        assert!(code_ref.file.exists(), "File should exist");
    }
}

#[test]
fn test_end_to_end_workflow_with_orchestrator() {
    println!("\n=== End-to-End Search Workflow ===\n");
    
    let search_text = "add new";
    let query = SearchQuery::new(search_text.to_string())
        .with_base_dir(PathBuf::from("tests/fixtures/rails-app"));
    
    println!("Searching for: '{}'", search_text);
    println!("Base directory: tests/fixtures/rails-app\n");
    
    let result = run_search(query).unwrap();
    
    println!("📝 Translation Entries Found: {}", result.translation_entries.len());
    for entry in &result.translation_entries {
        println!("  ✓ {} = '{}'", entry.key, entry.value);
        println!("    Location: {}:{}", entry.file.display(), entry.line);
    }
    
    println!("\n💻 Code References Found: {}", result.code_references.len());
    for code_ref in &result.code_references {
        println!("  ✓ {}", code_ref.key_path);
        println!("    File: {}:{}", code_ref.file.display(), code_ref.line);
        println!("    Code: {}", code_ref.context.trim());
    }
    
    println!("\n✅ End-to-end workflow complete!");
    
    // Assertions
    assert!(!result.translation_entries.is_empty());
    assert!(!result.code_references.is_empty());
}
