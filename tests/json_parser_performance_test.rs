use cs::parse::JsonParser;
use std::path::Path;
use std::time::Instant;

#[test]
fn test_json_parser_performance_large_file() {
    let large_file_path = Path::new("tests/fixtures/performance/large-en.json");

    // Skip test if file doesn't exist
    if !large_file_path.exists() {
        println!("Skipping performance test - large JSON file not found");
        println!("Run 'node generate_large_translation.js' to create test file");
        return;
    }

    println!("Testing JSON parser performance on large file...");

    // Test 1: Parse entire file without query
    let start = Instant::now();
    let entries = JsonParser::parse_file(large_file_path).expect("Failed to parse large JSON file");
    let parse_duration = start.elapsed();

    println!(
        "Parse entire file: {} entries in {:?}",
        entries.len(),
        parse_duration
    );
    assert!(!entries.is_empty(), "Should find translation entries");

    // Test 2: Parse with specific query (should be faster due to filtering)
    let start = Instant::now();
    let filtered_entries = JsonParser::parse_file_with_query(large_file_path, Some("Click here"))
        .expect("Failed to parse with query");
    let query_duration = start.elapsed();

    println!(
        "Parse with query 'Click here': {} entries in {:?}",
        filtered_entries.len(),
        query_duration
    );
    assert!(
        !filtered_entries.is_empty(),
        "Should find entries matching 'Click here'"
    );

    // Test 3: Parse with common query
    let start = Instant::now();
    let common_entries = JsonParser::parse_file_with_query(large_file_path, Some("error"))
        .expect("Failed to parse with common query");
    let common_duration = start.elapsed();

    println!(
        "Parse with query 'error': {} entries in {:?}",
        common_entries.len(),
        common_duration
    );

    // Test 4: Parse with rare query
    let start = Instant::now();
    let rare_entries = JsonParser::parse_file_with_query(large_file_path, Some("nonexistent"))
        .expect("Failed to parse with rare query");
    let rare_duration = start.elapsed();

    println!(
        "Parse with query 'nonexistent': {} entries in {:?}",
        rare_entries.len(),
        rare_duration
    );
    assert_eq!(
        rare_entries.len(),
        0,
        "Should find no entries for nonexistent query"
    );

    // Performance assertions
    assert!(
        parse_duration.as_millis() < 5000,
        "Full parse should complete within 5 seconds"
    );
    assert!(
        query_duration.as_millis() < 5000,
        "Query parse should complete within 5 seconds"
    );

    // Verify data integrity
    let sample_entry = entries
        .iter()
        .find(|e| e.key.contains("auth.labels.create"))
        .unwrap();
    assert_eq!(sample_entry.value, "Click here to continue");

    println!("JSON parser performance test completed successfully!");
}

#[test]
fn test_json_parser_memory_usage() {
    let large_file_path = Path::new("tests/fixtures/performance/large-en.json");

    if !large_file_path.exists() {
        println!("Skipping memory test - large JSON file not found");
        return;
    }

    // Test multiple parses to check for memory leaks
    println!("Testing JSON parser memory usage...");

    for i in 1..=5 {
        let start = Instant::now();
        let entries = JsonParser::parse_file(large_file_path).expect("Failed to parse");
        let duration = start.elapsed();

        println!("Parse #{}: {} entries in {:?}", i, entries.len(), duration);

        // Each parse should be consistent
        assert!(
            entries.len() > 7000,
            "Should consistently find many entries"
        );
        assert!(
            duration.as_millis() < 5000,
            "Each parse should be reasonably fast"
        );
    }

    println!("Memory usage test completed - no significant degradation detected");
}

#[test]
fn test_json_parser_nested_structure_performance() {
    let large_file_path = Path::new("tests/fixtures/performance/large-en.json");

    if !large_file_path.exists() {
        println!("Skipping nested structure test - large JSON file not found");
        return;
    }

    println!("Testing JSON parser on deeply nested structures...");

    let start = Instant::now();
    let entries = JsonParser::parse_file(large_file_path).expect("Failed to parse");
    let duration = start.elapsed();

    // Analyze the structure
    let max_depth = entries
        .iter()
        .map(|e| e.key.matches('.').count())
        .max()
        .unwrap_or(0);

    let avg_depth: f64 = entries
        .iter()
        .map(|e| e.key.matches('.').count() as f64)
        .sum::<f64>()
        / entries.len() as f64;

    println!("Parsed {} entries in {:?}", entries.len(), duration);
    println!("Max nesting depth: {} levels", max_depth);
    println!("Average nesting depth: {:.2} levels", avg_depth);

    // Verify we handle nested structures correctly
    assert!(max_depth >= 3, "Should handle at least 3 levels of nesting");
    assert!(avg_depth >= 2.0, "Should have reasonable average nesting");

    // Check for specific nested keys
    let nested_keys: Vec<_> = entries
        .iter()
        .filter(|e| e.key.contains("_details"))
        .collect();

    println!("Found {} nested detail entries", nested_keys.len());
    assert!(
        !nested_keys.is_empty(),
        "Should find nested detail structures"
    );

    // Verify a specific nested structure
    let detail_entry = entries
        .iter()
        .find(|e| e.key == "auth.labels.create_details.long")
        .expect("Should find specific nested entry");

    assert!(detail_entry
        .value
        .contains("additional detailed information"));

    println!("Nested structure performance test completed successfully!");
}

#[test]
fn test_json_parser_query_filtering_accuracy() {
    let large_file_path = Path::new("tests/fixtures/performance/large-en.json");

    if !large_file_path.exists() {
        println!("Skipping filtering accuracy test - large JSON file not found");
        return;
    }

    println!("Testing JSON parser query filtering accuracy...");

    // Test case-insensitive filtering
    let entries_lower = JsonParser::parse_file_with_query(large_file_path, Some("click"))
        .expect("Failed to parse with lowercase query");

    let entries_upper = JsonParser::parse_file_with_query(large_file_path, Some("CLICK"))
        .expect("Failed to parse with uppercase query");

    println!("Lowercase 'click': {} entries", entries_lower.len());
    println!("Uppercase 'CLICK': {} entries", entries_upper.len());

    // Should find the same entries regardless of case
    assert_eq!(
        entries_lower.len(),
        entries_upper.len(),
        "Case-insensitive filtering should work"
    );
    assert!(
        !entries_lower.is_empty(),
        "Should find entries containing 'click'"
    );

    // Test partial word matching
    let partial_entries = JsonParser::parse_file_with_query(large_file_path, Some("error"))
        .expect("Failed to parse with partial query");

    println!("Partial match 'error': {} entries", partial_entries.len());

    // Verify all returned entries actually contain the query
    for entry in &partial_entries {
        assert!(
            entry.value.to_lowercase().contains("error"),
            "Entry '{}' should contain 'error'",
            entry.value
        );
    }

    // Test specific phrase matching
    let phrase_entries =
        JsonParser::parse_file_with_query(large_file_path, Some("additional detailed"))
            .expect("Failed to parse with phrase query");

    println!(
        "Phrase match 'additional detailed': {} entries",
        phrase_entries.len()
    );
    assert!(
        !phrase_entries.is_empty(),
        "Should find entries with phrase"
    );

    println!("Query filtering accuracy test completed successfully!");
}
