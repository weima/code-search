use cs::search::PatternMatcher;
use std::path::PathBuf;

#[test]
fn test_pattern_matcher_finds_ruby_i18n_usage() {
    let matcher = PatternMatcher::new();
    
    // Search for a key that exists in the fixtures
    let result = matcher.find_usages("invoice.labels.add_new");
    
    assert!(result.is_ok(), "Pattern matching should succeed");
    let code_refs = result.unwrap();
    
    // Should find at least one reference
    assert!(!code_refs.is_empty(), "Should find at least one code reference");
    
    // Find the reference in invoices.ts
    let invoices_refs: Vec<_> = code_refs.iter()
        .filter(|r| r.file.to_string_lossy().contains("invoices.ts"))
        .collect();
    
    assert!(!invoices_refs.is_empty(), "Should find reference in invoices.ts");
    
    // Verify the reference details
    let first_ref = invoices_refs[0];
    assert_eq!(first_ref.key_path, "invoice.labels.add_new");
    assert!(first_ref.context.contains("I18n.t"));
}

#[test]
fn test_pattern_matcher_finds_multiple_usages() {
    let matcher = PatternMatcher::new();
    
    // Search for keys that are used multiple times
    let result = matcher.find_usages("invoice.labels.delete");
    
    assert!(result.is_ok());
    let code_refs = result.unwrap();
    
    // Should find the reference in invoices.ts
    assert!(!code_refs.is_empty(), "Should find code references for delete");
    
    for code_ref in &code_refs {
        assert_eq!(code_ref.key_path, "invoice.labels.delete");
        assert!(code_ref.context.contains("invoice.labels.delete"));
    }
}

#[test]
fn test_pattern_matcher_finds_all_invoice_keys() {
    let matcher = PatternMatcher::new();
    
    // Test all the invoice label keys
    let keys = vec![
        "invoice.labels.add_new",
        "invoice.labels.edit",
        "invoice.labels.delete",
    ];
    
    for key in keys {
        let result = matcher.find_usages(key);
        assert!(result.is_ok(), "Should find usage for key: {}", key);
        
        let code_refs = result.unwrap();
        assert!(!code_refs.is_empty(), "Should have at least one reference for key: {}", key);
        
        // Verify all references have the correct key
        for code_ref in &code_refs {
            assert_eq!(code_ref.key_path, key);
        }
    }
}

#[test]
fn test_pattern_matcher_finds_message_keys() {
    let matcher = PatternMatcher::new();
    
    // Test message keys
    let message_keys = vec![
        "invoice.messages.created",
        "invoice.messages.updated",
        "invoice.messages.deleted",
    ];
    
    for key in message_keys {
        let result = matcher.find_usages(key);
        assert!(result.is_ok(), "Should find usage for message key: {}", key);
        
        let code_refs = result.unwrap();
        assert!(!code_refs.is_empty(), "Should have reference for message: {}", key);
    }
}

#[test]
fn test_pattern_matcher_finds_error_keys() {
    let matcher = PatternMatcher::new();
    
    let result = matcher.find_usages("invoice.errors.not_found");
    assert!(result.is_ok());
    
    let code_refs = result.unwrap();
    assert!(!code_refs.is_empty(), "Should find error key usage");
    
    let error_ref = &code_refs[0];
    assert!(error_ref.context.contains("invoice.errors.not_found"));
}

#[test]
fn test_pattern_matcher_finds_user_keys() {
    let matcher = PatternMatcher::new();
    
    // Test user-related keys
    let user_keys = vec![
        "user.labels.login",
        "user.labels.logout",
        "user.messages.welcome",
    ];
    
    for key in user_keys {
        let result = matcher.find_usages(key);
        assert!(result.is_ok(), "Should find usage for user key: {}", key);
        
        let code_refs = result.unwrap();
        assert!(!code_refs.is_empty(), "Should have reference for user key: {}", key);
    }
}

#[test]
fn test_pattern_matcher_no_matches_for_nonexistent_key() {
    let matcher = PatternMatcher::new();
    
    let result = matcher.find_usages("nonexistent.key.path");
    assert!(result.is_ok());
    
    let code_refs = result.unwrap();
    assert!(code_refs.is_empty(), "Should not find references for nonexistent key");
}

#[test]
fn test_pattern_matcher_includes_line_numbers() {
    let matcher = PatternMatcher::new();
    
    let result = matcher.find_usages("invoice.labels.add_new");
    assert!(result.is_ok());
    
    let code_refs = result.unwrap();
    assert!(!code_refs.is_empty());
    
    // Verify line numbers are present and reasonable
    for code_ref in &code_refs {
        assert!(code_ref.line > 0, "Line number should be positive");
        assert!(code_ref.line < 1000, "Line number should be reasonable");
    }
}

#[test]
fn test_pattern_matcher_includes_file_paths() {
    let matcher = PatternMatcher::new();
    
    let result = matcher.find_usages("invoice.labels.add_new");
    assert!(result.is_ok());
    
    let code_refs = result.unwrap();
    assert!(!code_refs.is_empty());
    
    // Verify file paths are present
    for code_ref in &code_refs {
        assert!(!code_ref.file.as_os_str().is_empty(), "File path should not be empty");
        assert!(code_ref.file.exists(), "File should exist: {:?}", code_ref.file);
    }
}

#[test]
fn test_pattern_matcher_context_includes_full_line() {
    let matcher = PatternMatcher::new();
    
    let result = matcher.find_usages("invoice.labels.add_new");
    assert!(result.is_ok());
    
    let code_refs = result.unwrap();
    assert!(!code_refs.is_empty());
    
    // Find references in actual code files (not test files or docs)
    let code_file_refs: Vec<_> = code_refs.iter()
        .filter(|r| {
            let path = r.file.to_string_lossy();
            path.contains("fixtures") && 
            (path.ends_with(".ts") || path.ends_with(".tsx") || path.ends_with(".vue"))
        })
        .collect();
    
    assert!(!code_file_refs.is_empty(), "Should find references in code files");
    
    // Verify context includes the translation call
    for code_ref in code_file_refs {
        assert!(code_ref.context.contains("invoice.labels.add_new"), "Context should include the key");
    }
}

#[test]
fn test_pattern_matcher_batch_processing() {
    use cs::parse::KeyExtractor;
    
    let matcher = PatternMatcher::new();
    let extractor = KeyExtractor::new();
    
    // Extract entries matching "add new" from the fixtures directory
    let base_dir = PathBuf::from("tests/fixtures/rails-app/config/locales");
    let matching_entries = extractor.extract(&base_dir, "add new").expect("Should extract entries");
    assert!(!matching_entries.is_empty(), "Should find 'add new' in translations");
    
    // Find code references for all matching entries
    let result = matcher.find_usages_batch(&matching_entries);
    assert!(result.is_ok());
    
    let code_refs = result.unwrap();
    assert!(!code_refs.is_empty(), "Should find code references via batch processing");
}

#[test]
fn test_end_to_end_pattern_matching_workflow() {
    use cs::parse::{YamlParser, KeyExtractor};
    
    // Step 1: Parse YAML translation file
    let yaml_path = PathBuf::from("tests/fixtures/rails-app/config/locales/en.yml");
    let entries = YamlParser::parse_file(&yaml_path).expect("Should parse YAML");
    
    println!("Found {} translation entries", entries.len());
    
    // Step 2: Extract entries matching search text using KeyExtractor
    let extractor = KeyExtractor::new();
    let search_text = "add new";
    let base_dir = PathBuf::from("tests/fixtures/rails-app/config/locales");
    let matching_entries = extractor.extract(&base_dir, search_text).expect("Should extract entries");
    
    println!("Found {} entries matching '{}'", matching_entries.len(), search_text);
    assert!(!matching_entries.is_empty(), "Should find matching entries");
    
    // Step 3: Find code references for each matching entry
    let matcher = PatternMatcher::new();
    let mut all_code_refs = Vec::new();
    
    for entry in &matching_entries {
        println!("Searching for key: {}", entry.key);
        let code_refs = matcher.find_usages(&entry.key).expect("Should find usages");
        println!("  Found {} code references", code_refs.len());
        
        for code_ref in &code_refs {
            println!("    - {}:{} - {}", 
                code_ref.file.display(), 
                code_ref.line, 
                code_ref.context.trim()
            );
        }
        
        all_code_refs.extend(code_refs);
    }
    
    // Verify we found code references
    assert!(!all_code_refs.is_empty(), "Should find code references in end-to-end workflow");
    
    // Find references in the fixtures directory
    let fixture_refs: Vec<_> = all_code_refs.iter()
        .filter(|r| r.file.to_string_lossy().contains("fixtures"))
        .collect();
    
    assert!(!fixture_refs.is_empty(), "Should find references in fixtures");
    
    // Verify at least one reference is correct
    let has_valid_ref = fixture_refs.iter().any(|code_ref| {
        code_ref.key_path == "invoice.labels.add_new" ||
        code_ref.key_path == "en.invoice.labels.add_new"
    });
    assert!(has_valid_ref, "Should have at least one valid reference");
    
    println!("\n✅ End-to-end workflow complete!");
    println!("   Search text: '{}'", search_text);
    println!("   Translation entries found: {}", matching_entries.len());
    println!("   Code references found: {}", all_code_refs.len());
}
