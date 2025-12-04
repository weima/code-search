use cs::TextSearcher;

#[test]
fn test_search_finds_text_in_fixtures() {
    let searcher = TextSearcher::new(std::env::current_dir().unwrap());
    let matches = searcher.search("add new").unwrap();

    // Filter to only fixtures directory to avoid matches in other files
    let fixture_matches: Vec<_> = matches
        .iter()
        .filter(|m| m.file.to_str().unwrap().contains("fixtures"))
        .collect();

    // Should find "add new" in multiple files
    assert!(
        !fixture_matches.is_empty(),
        "Should find at least one match for 'add new' in fixtures"
    );

    // Should find in YAML file
    assert!(
        fixture_matches
            .iter()
            .any(|m| m.file.to_str().unwrap().contains("en.yml")),
        "Should find match in YAML translation file"
    );

    // Should find in JSON files
    assert!(
        fixture_matches
            .iter()
            .any(|m| m.file.to_str().unwrap().contains("en.json")),
        "Should find match in JSON translation files"
    );
}

#[test]
fn test_search_case_insensitive() {
    let searcher = TextSearcher::new(std::env::current_dir().unwrap());
    let matches = searcher.search("ADD NEW").unwrap();

    let fixture_matches: Vec<_> = matches
        .iter()
        .filter(|m| m.file.to_str().unwrap().contains("fixtures"))
        .collect();

    // Case-insensitive should find matches
    assert!(
        !fixture_matches.is_empty(),
        "Case-insensitive search should find matches"
    );
}

#[test]
fn test_search_case_sensitive() {
    let searcher = TextSearcher::new(std::env::current_dir().unwrap()).case_sensitive(true);
    let uppercase_matches = searcher.search("ADD NEW").unwrap();
    let lowercase_matches = searcher.search("add new").unwrap();

    let uppercase_fixtures: Vec<_> = uppercase_matches
        .iter()
        .filter(|m| m.file.to_str().unwrap().contains("fixtures"))
        .collect();

    let lowercase_fixtures: Vec<_> = lowercase_matches
        .iter()
        .filter(|m| m.file.to_str().unwrap().contains("fixtures"))
        .collect();

    // Case-sensitive should not find uppercase version
    assert_eq!(
        uppercase_fixtures.len(),
        0,
        "Should not find 'ADD NEW' with case-sensitive search"
    );
    assert!(
        !lowercase_fixtures.is_empty(),
        "Should find 'add new' with case-sensitive search"
    );
}

#[test]
fn test_search_finds_function_names() {
    let searcher = TextSearcher::new(std::env::current_dir().unwrap());
    let matches = searcher.search("processPayment").unwrap();

    let code_matches: Vec<_> = matches
        .iter()
        .filter(|m| m.file.to_str().unwrap().contains("code-examples"))
        .collect();

    assert!(
        code_matches.len() >= 2,
        "Should find processPayment in multiple files"
    );

    // Should find definition in utils.ts
    assert!(
        code_matches
            .iter()
            .any(|m| m.file.to_str().unwrap().contains("utils.ts")),
        "Should find processPayment in utils.ts"
    );

    // Should find usage in checkout.ts
    assert!(
        code_matches
            .iter()
            .any(|m| m.file.to_str().unwrap().contains("checkout.ts")),
        "Should find processPayment in checkout.ts"
    );
}

#[test]
fn test_search_finds_error_messages() {
    let searcher = TextSearcher::new(std::env::current_dir().unwrap());
    let matches = searcher.search("Invalid payment amount").unwrap();

    let code_matches: Vec<_> = matches
        .iter()
        .filter(|m| m.file.to_str().unwrap().contains("code-examples"))
        .collect();

    assert!(
        code_matches.len() >= 2,
        "Should find error message in multiple locations"
    );

    // Should find in utils.ts
    assert!(
        code_matches
            .iter()
            .any(|m| m.file.to_str().unwrap().contains("utils.ts")),
        "Should find error message in utils.ts"
    );
}

#[test]
fn test_search_finds_variables() {
    let searcher = TextSearcher::new(std::env::current_dir().unwrap());
    let matches = searcher.search("userId").unwrap();

    let code_matches: Vec<_> = matches
        .iter()
        .filter(|m| m.file.to_str().unwrap().contains("code-examples"))
        .collect();

    // userId is used extensively across all three files
    assert!(
        code_matches.len() > 10,
        "Should find userId many times across files"
    );

    // Should find in all three code files
    assert!(
        code_matches
            .iter()
            .any(|m| m.file.to_str().unwrap().contains("utils.ts")),
        "Should find userId in utils.ts"
    );
    assert!(
        code_matches
            .iter()
            .any(|m| m.file.to_str().unwrap().contains("checkout.ts")),
        "Should find userId in checkout.ts"
    );
    assert!(
        code_matches
            .iter()
            .any(|m| m.file.to_str().unwrap().contains("api.ts")),
        "Should find userId in api.ts"
    );
}

#[test]
fn test_search_no_matches() {
    let searcher = TextSearcher::new(std::env::current_dir().unwrap());
    // Use a unique random string that won't appear anywhere
    let matches = searcher
        .search("xYzAbC987654321ThisShouldNeverExist")
        .unwrap();

    // Filter out the test file itself (which contains this search term)
    let non_test_matches: Vec<_> = matches
        .iter()
        .filter(|m| !m.file.to_str().unwrap().contains("search_test.rs"))
        .collect();

    assert_eq!(
        non_test_matches.len(),
        0,
        "Should return empty vec when no matches found (excluding test file itself)"
    );
}

#[test]
fn test_match_includes_line_numbers() {
    let searcher = TextSearcher::new(std::env::current_dir().unwrap());
    let matches = searcher.search("processPayment").unwrap();

    let code_matches: Vec<_> = matches
        .iter()
        .filter(|m| m.file.to_str().unwrap().contains("code-examples"))
        .collect();

    // All matches should have valid line numbers
    for m in code_matches {
        assert!(m.line > 0, "Line number should be greater than 0");
    }
}

#[test]
fn test_match_includes_content() {
    let searcher = TextSearcher::new(std::env::current_dir().unwrap());
    let matches = searcher.search("processPayment").unwrap();

    let code_matches: Vec<_> = matches
        .iter()
        .filter(|m| m.file.to_str().unwrap().contains("code-examples"))
        .collect();

    // All matches should have non-empty content
    for m in code_matches {
        assert!(!m.content.is_empty(), "Content should not be empty");
        assert!(
            m.content.contains("processPayment"),
            "Content should contain search text"
        );
    }
}

#[test]
#[ignore] // Can't test ripgrep not installed without mocking
fn test_ripgrep_not_installed_error() {
    // This test would only work if we could mock the PATH
    // For now, we assume ripgrep is installed
    // In a real scenario, you'd use dependency injection or env manipulation
}
