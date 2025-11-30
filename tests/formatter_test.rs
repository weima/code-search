use cs::{run_search, ReferenceTreeBuilder, SearchQuery, TreeFormatter};
use std::path::PathBuf;

#[test]
fn test_format_search_results() {
    let query = SearchQuery::new("add new".to_string())
        .with_base_dir(PathBuf::from("tests/fixtures/rails-app"));

    let result = run_search(query).expect("Search should succeed");
    let tree = ReferenceTreeBuilder::build(&result);
    let formatter = TreeFormatter::new();
    let output = formatter.format(&tree);

    println!("\n{}", output);

    // Verify output contains expected elements
    assert!(output.contains("'add new'"));
    assert!(output.contains("search query"));
    assert!(output.contains("invoice.labels.add_new"));
    assert!(output.contains("Key:"));
    assert!(output.contains("Code:"));
}

#[test]
fn test_format_with_custom_width() {
    let query = SearchQuery::new("add new".to_string())
        .with_base_dir(PathBuf::from("tests/fixtures/rails-app"));

    let result = run_search(query).expect("Search should succeed");
    let tree = ReferenceTreeBuilder::build(&result);

    let formatter = TreeFormatter::with_width(120);
    let output = formatter.format(&tree);

    println!("\n=== Wide Format (120 columns) ===\n{}", output);

    assert!(output.contains("'add new'"));
}

#[test]
fn test_format_no_results() {
    let query = SearchQuery::new("nonexistent xyz".to_string())
        .with_base_dir(PathBuf::from("tests/fixtures/rails-app"));

    let result = run_search(query).expect("Search should succeed");
    let tree = ReferenceTreeBuilder::build(&result);
    let formatter = TreeFormatter::new();
    let output = formatter.format(&tree);

    println!("\n=== No Results ===\n{}", output);

    assert!(output.contains("'nonexistent xyz'"));
    assert!(output.contains("search query"));
    // Should only have the root node
    assert_eq!(output.lines().count(), 1);
}

#[test]
fn test_format_tree_structure() {
    let query = SearchQuery::new("add new".to_string())
        .with_base_dir(PathBuf::from("tests/fixtures/rails-app"));

    let result = run_search(query).expect("Search should succeed");
    let tree = ReferenceTreeBuilder::build(&result);
    let formatter = TreeFormatter::new();
    let output = formatter.format(&tree);

    // Check for tree structure characters
    assert!(output.contains("└─>") || output.contains("├─>"));

    // Check for proper indentation (spaces or tree characters)
    let lines: Vec<&str> = output.lines().collect();
    assert!(lines.len() > 1, "Should have multiple lines");
}

#[test]
fn test_end_to_end_with_formatter() {
    println!("\n=== End-to-End with Formatter ===\n");

    let search_text = "add new";
    println!("Searching for: '{}'", search_text);

    let query = SearchQuery::new(search_text.to_string())
        .with_base_dir(PathBuf::from("tests/fixtures/rails-app"));

    println!("1. Running search...");
    let result = run_search(query).expect("Search should succeed");
    println!(
        "   Found {} translations, {} code references",
        result.translation_entries.len(),
        result.code_references.len()
    );

    println!("2. Building tree...");
    let tree = ReferenceTreeBuilder::build(&result);
    println!(
        "   Tree has {} nodes, depth {}",
        tree.node_count(),
        tree.max_depth()
    );

    println!("3. Formatting output...\n");
    let formatter = TreeFormatter::new();
    let output = formatter.format(&tree);

    println!("{}", output);

    println!("\n✅ End-to-end workflow complete!");

    assert!(!output.is_empty());
    assert!(output.contains("'add new'"));
}

#[test]
fn test_format_multiple_translations() {
    let query = SearchQuery::new("invoice".to_string())
        .with_base_dir(PathBuf::from("tests/fixtures/rails-app"));

    let result = run_search(query).expect("Search should succeed");
    let tree = ReferenceTreeBuilder::build(&result);
    let formatter = TreeFormatter::new();
    let output = formatter.format(&tree);

    println!("\n=== Multiple Translations ===\n{}", output);

    // Should have multiple translation entries
    assert!(output.contains("invoice"));

    // Count the number of translation nodes (lines with .yml)
    let yml_count = output.lines().filter(|line| line.contains(".yml")).count();
    assert!(
        yml_count > 0,
        "Should have at least one translation file reference"
    );
}

#[test]
fn test_format_shows_locations() {
    let query = SearchQuery::new("add new".to_string())
        .with_base_dir(PathBuf::from("tests/fixtures/rails-app"));

    let result = run_search(query).expect("Search should succeed");
    let tree = ReferenceTreeBuilder::build(&result);
    let formatter = TreeFormatter::new();
    let output = formatter.format(&tree);

    // Should show file locations with line numbers
    assert!(output.contains(".yml:"));
    assert!(output.contains(".ts:") || output.contains(".tsx:"));
}

#[test]
fn test_format_readable_output() {
    let query = SearchQuery::new("add new".to_string())
        .with_base_dir(PathBuf::from("tests/fixtures/rails-app"));

    let result = run_search(query).expect("Search should succeed");
    let tree = ReferenceTreeBuilder::build(&result);
    let formatter = TreeFormatter::new();
    let output = formatter.format(&tree);

    println!("\n=== Readable Output Test ===\n{}", output);

    // Verify output is human-readable
    assert!(output.contains("Key:"), "Should label key paths");
    assert!(output.contains("Code:"), "Should label code references");
    assert!(output.contains("search query"), "Should label root");

    // Verify no truncation artifacts in short content
    assert!(
        !output.contains("...") || output.len() > 500,
        "Short content should not be truncated"
    );
}

#[test]
fn test_format_comparison() {
    let query = SearchQuery::new("add new".to_string())
        .with_base_dir(PathBuf::from("tests/fixtures/rails-app"));

    let result = run_search(query).expect("Search should succeed");
    let tree = ReferenceTreeBuilder::build(&result);

    println!("\n=== Format Comparison ===\n");

    println!("--- 80 columns ---");
    let formatter_80 = TreeFormatter::with_width(80);
    let output_80 = formatter_80.format(&tree);
    println!("{}", output_80);

    println!("\n--- 120 columns ---");
    let formatter_120 = TreeFormatter::with_width(120);
    let output_120 = formatter_120.format(&tree);
    println!("{}", output_120);

    // Both should have the same structure
    assert_eq!(output_80.lines().count(), output_120.lines().count());
}
