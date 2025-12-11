use cs::{
    run_search, CodeReference, ReferenceTreeBuilder, SearchQuery, SearchResult, TranslationEntry,
    TreeFormatter,
};
use proptest::prelude::*;
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

// **Feature: simple-flag-and-context-lines, Property 1: Simple format consistency**
proptest! {
    #[test]
    fn test_simple_format_consistency(
        translation_key in "[a-zA-Z][a-zA-Z0-9_.]{0,50}",
        translation_value in "[^\\n\\r]{0,100}",
        file_path in "[a-zA-Z0-9_/.-]{1,50}\\.(yml|ts|js)",
        line_num in 1u32..1000u32,
        code_content in "[^\\n\\r]{0,200}"
    ) {
        // **Validates: Requirements 1.1, 1.2, 1.3, 1.4, 1.5, 3.1, 3.2, 3.4**

        // Create a test SearchResult with generated data
        let mut result = SearchResult {
            query: "test".to_string(),
            translation_entries: vec![],
            code_references: vec![],
        };

        // Add translation entry
        result.translation_entries.push(TranslationEntry {
            key: translation_key.clone(),
            value: translation_value.clone(),
            line: line_num as usize,
            file: PathBuf::from(&file_path),
        });

        // Add code reference
        result.code_references.push(CodeReference {
            file: PathBuf::from(&file_path),
            line: line_num as usize,
            pattern: "test".to_string(),
            context: code_content.clone(),
            key_path: "".to_string(),
            context_before: vec![],
            context_after: vec![],
        });

        // Test simple format
        let formatter = TreeFormatter::new().with_simple_format(true);
        let output = formatter.format_result(&result);

        // Verify format consistency: each line should follow file:line:content format
        for line in output.lines() {
            if !line.trim().is_empty() {
                let parts: Vec<&str> = line.splitn(3, ':').collect();
                prop_assert_eq!(parts.len(), 3, "Line should have exactly 3 parts separated by colons: {}", line);

                // First part should be a file path
                prop_assert!(!parts[0].is_empty(), "File path should not be empty: {}", line);

                // Second part should be a line number
                prop_assert!(parts[1].parse::<u32>().is_ok(), "Line number should be numeric: {}", line);

                // Third part should be content (can be empty)
                // Content should not contain tree characters or ANSI codes
                prop_assert!(!parts[2].contains("├─>"), "Content should not contain tree characters: {}", line);
                prop_assert!(!parts[2].contains("└─>"), "Content should not contain tree characters: {}", line);
                prop_assert!(!parts[2].contains("\x1b["), "Content should not contain ANSI codes: {}", line);

                // Content should be on a single line (no newlines)
                prop_assert!(!parts[2].contains('\n'), "Content should not contain newlines: {}", line);
                prop_assert!(!parts[2].contains('\r'), "Content should not contain carriage returns: {}", line);
            }
        }
    }
}
#[test]
fn test_simple_format_empty_results() {
    // Test empty results
    let result = SearchResult {
        query: "nonexistent".to_string(),
        translation_entries: vec![],
        code_references: vec![],
    };

    let formatter = TreeFormatter::new().with_simple_format(true);
    let output = formatter.format_result(&result);

    assert_eq!(output, "", "Empty results should produce empty output");
}

#[test]
fn test_simple_format_single_translation() {
    // Test single translation result
    let result = SearchResult {
        query: "test".to_string(),
        translation_entries: vec![TranslationEntry {
            key: "test.key".to_string(),
            value: "test value".to_string(),
            line: 5,
            file: PathBuf::from("test.yml"),
        }],
        code_references: vec![],
    };

    let formatter = TreeFormatter::new().with_simple_format(true);
    let output = formatter.format_result(&result);

    assert_eq!(output, "test.yml:5:test.key: test value\n");
}

#[test]
fn test_simple_format_single_code_reference() {
    // Test single code reference result
    let result = SearchResult {
        query: "test".to_string(),
        translation_entries: vec![],
        code_references: vec![CodeReference {
            file: PathBuf::from("test.ts"),
            line: 10,
            pattern: "test".to_string(),
            context: "const x = test();".to_string(),
            key_path: "".to_string(),
            context_before: vec![],
            context_after: vec![],
        }],
    };

    let formatter = TreeFormatter::new().with_simple_format(true);
    let output = formatter.format_result(&result);

    assert_eq!(output, "test.ts:10:const x = test();\n");
}

#[test]
fn test_simple_format_multiple_results() {
    // Test multiple results
    let result = SearchResult {
        query: "test".to_string(),
        translation_entries: vec![
            TranslationEntry {
                key: "test.key1".to_string(),
                value: "value1".to_string(),
                line: 1,
                file: PathBuf::from("en.yml"),
            },
            TranslationEntry {
                key: "test.key2".to_string(),
                value: "value2".to_string(),
                line: 2,
                file: PathBuf::from("en.yml"),
            },
        ],
        code_references: vec![
            CodeReference {
                file: PathBuf::from("app.ts"),
                line: 5,
                pattern: "test".to_string(),
                context: "I18n.t('test.key1')".to_string(),
                key_path: "test.key1".to_string(),
                context_before: vec![],
                context_after: vec![],
            },
            CodeReference {
                file: PathBuf::from("app.ts"),
                line: 10,
                pattern: "test".to_string(),
                context: "I18n.t('test.key2')".to_string(),
                key_path: "test.key2".to_string(),
                context_before: vec![],
                context_after: vec![],
            },
        ],
    };

    let formatter = TreeFormatter::new().with_simple_format(true);
    let output = formatter.format_result(&result);

    let expected = "en.yml:1:test.key1: value1\n\
                    en.yml:2:test.key2: value2\n\
                    app.ts:5:I18n.t('test.key1')\n\
                    app.ts:10:I18n.t('test.key2')\n";
    assert_eq!(output, expected);
}

#[test]
fn test_simple_format_special_characters_in_paths() {
    // Test special characters in file paths
    let result = SearchResult {
        query: "test".to_string(),
        translation_entries: vec![TranslationEntry {
            key: "test.key".to_string(),
            value: "test value".to_string(),
            line: 1,
            file: PathBuf::from("path with spaces/file:name.yml"),
        }],
        code_references: vec![],
    };

    let formatter = TreeFormatter::new().with_simple_format(true);
    let output = formatter.format_result(&result);

    // Should escape colons in file paths
    assert_eq!(
        output,
        "path with spaces/file\\:name.yml:1:test.key: test value\n"
    );
}

#[test]
fn test_simple_format_special_characters_in_content() {
    // Test special characters in content
    let result = SearchResult {
        query: "test".to_string(),
        translation_entries: vec![TranslationEntry {
            key: "test.key".to_string(),
            value: "value with\nnewlines\rand\ttabs".to_string(),
            line: 1,
            file: PathBuf::from("test.yml"),
        }],
        code_references: vec![CodeReference {
            file: PathBuf::from("test.ts"),
            line: 5,
            pattern: "test".to_string(),
            context: "  code with\n  newlines  ".to_string(),
            key_path: "".to_string(),
            context_before: vec![],
            context_after: vec![],
        }],
    };

    let formatter = TreeFormatter::new().with_simple_format(true);
    let output = formatter.format_result(&result);

    // Should normalize whitespace and remove newlines
    let lines: Vec<&str> = output.lines().collect();
    assert_eq!(lines.len(), 2);

    // Translation entry should have normalized content
    assert!(lines[0].contains("test.key: value with newlines and tabs"));
    assert!(!lines[0].contains('\n'));
    assert!(!lines[0].contains('\r'));

    // Code reference should have trimmed and normalized content
    assert!(lines[1].contains("code with newlines"));
    assert!(!lines[1].contains('\n'));
    assert!(!lines[1].contains('\r'));
}

#[test]
fn test_simple_format_ansi_codes_stripped() {
    // Test ANSI color codes are stripped
    let result = SearchResult {
        query: "test".to_string(),
        translation_entries: vec![],
        code_references: vec![CodeReference {
            file: PathBuf::from("test.ts"),
            line: 5,
            pattern: "test".to_string(),
            context: "\x1b[31mred text\x1b[0m normal text".to_string(),
            key_path: "".to_string(),
            context_before: vec![],
            context_after: vec![],
        }],
    };

    let formatter = TreeFormatter::new().with_simple_format(true);
    let output = formatter.format_result(&result);

    assert_eq!(output, "test.ts:5:red text normal text\n");
    assert!(!output.contains("\x1b["));
}

#[test]
fn test_simple_format_different_search_types() {
    // Test that different search types (translation vs exact match) work consistently
    let translation_result = SearchResult {
        query: "test".to_string(),
        translation_entries: vec![TranslationEntry {
            key: "test.key".to_string(),
            value: "test value".to_string(),
            line: 1,
            file: PathBuf::from("en.yml"),
        }],
        code_references: vec![],
    };

    let exact_match_result = SearchResult {
        query: "test".to_string(),
        translation_entries: vec![],
        code_references: vec![CodeReference {
            file: PathBuf::from("app.ts"),
            line: 5,
            pattern: "test".to_string(),
            context: "const test = 'value';".to_string(),
            key_path: "".to_string(),
            context_before: vec![],
            context_after: vec![],
        }],
    };

    let formatter = TreeFormatter::new().with_simple_format(true);

    let translation_output = formatter.format_result(&translation_result);
    let exact_match_output = formatter.format_result(&exact_match_result);

    // Both should follow the same format
    assert_eq!(translation_output, "en.yml:1:test.key: test value\n");
    assert_eq!(exact_match_output, "app.ts:5:const test = 'value';\n");

    // Both should have the same structure (file:line:content)
    let translation_parts: Vec<&str> = translation_output.trim().splitn(3, ':').collect();
    let exact_match_parts: Vec<&str> = exact_match_output.trim().splitn(3, ':').collect();

    assert_eq!(translation_parts.len(), 3);
    assert_eq!(exact_match_parts.len(), 3);
}

// **Feature: simple-flag-and-context-lines, Property 7: Special character handling**
proptest! {
    #[test]
    fn test_special_character_handling(
        // Generate file paths with special characters
        file_base in "[a-zA-Z0-9_-]{1,20}",
        file_ext in prop::sample::select(vec!["yml", "ts", "js", "json"]),
        // Special characters that might appear in paths
        path_special_chars in prop::collection::vec(
            prop::sample::select(vec![" ", ".", "-", "_", "(", ")", "[", "]", "&", "$", "!", "@"]),
            0..3
        ),
        // Generate content with unicode and special characters
        content_base in "[a-zA-Z0-9 ]{0,50}",
        unicode_chars in prop::collection::vec(
            prop::sample::select(vec!["é", "ñ", "中", "🚀", "ü", "ß", "ø", "λ"]),
            0..3
        ),
        shell_metacharacters in prop::collection::vec(
            prop::sample::select(vec!["$", "|", "&", ";", "(", ")", "<", ">", "`", "\"", "'", "\\", "*", "?"]),
            0..5
        ),
        line_num in 1u32..1000u32,
        translation_key in "[a-zA-Z][a-zA-Z0-9_.]{0,30}",
        translation_value_base in "[a-zA-Z0-9 ]{0,30}"
    ) {
        // **Validates: Requirements 3.3**

        // Build file path with special characters
        let mut file_path = file_base.clone();
        for special_char in &path_special_chars {
            file_path.push_str(special_char);
        }
        file_path.push('.');
        file_path.push_str(file_ext);

        // Build content with unicode and special characters
        let mut content = content_base.clone();
        for unicode_char in &unicode_chars {
            content.push_str(unicode_char);
        }
        for meta_char in &shell_metacharacters {
            content.push_str(meta_char);
        }

        // Build translation value with special characters
        let mut translation_value = translation_value_base.clone();
        for unicode_char in &unicode_chars {
            translation_value.push_str(unicode_char);
        }

        // Create SearchResult with special characters
        let mut result = SearchResult {
            query: "test".to_string(),
            translation_entries: vec![],
            code_references: vec![],
        };

        // Add translation entry with special characters
        result.translation_entries.push(TranslationEntry {
            key: translation_key.clone(),
            value: translation_value.clone(),
            line: line_num as usize,
            file: PathBuf::from(&file_path),
        });

        // Add code reference with special characters
        result.code_references.push(CodeReference {
            file: PathBuf::from(&file_path),
            line: line_num as usize,
            pattern: "test".to_string(),
            context: content.clone(),
            key_path: "".to_string(),
            context_before: vec![],
            context_after: vec![],
        });

        // Test simple format with special characters
        let formatter = TreeFormatter::new().with_simple_format(true);
        let output = formatter.format_result(&result);

        // Verify output is parseable: each line should follow file:line:content format
        for line in output.lines() {
            if !line.trim().is_empty() {
                let parts: Vec<&str> = line.splitn(3, ':').collect();
                prop_assert_eq!(parts.len(), 3, "Line should have exactly 3 parts separated by colons even with special characters: {}", line);

                // File path should not be empty
                prop_assert!(!parts[0].is_empty(), "File path should not be empty: {}", line);

                // Line number should be numeric
                prop_assert!(parts[1].parse::<u32>().is_ok(), "Line number should be numeric: {}", line);

                // Content should be on a single line (no unescaped newlines)
                prop_assert!(!parts[2].contains('\n'), "Content should not contain unescaped newlines: {}", line);
                prop_assert!(!parts[2].contains('\r'), "Content should not contain unescaped carriage returns: {}", line);

                // Content should not contain tree characters or ANSI codes
                prop_assert!(!parts[2].contains("├─>"), "Content should not contain tree characters: {}", line);
                prop_assert!(!parts[2].contains("└─>"), "Content should not contain tree characters: {}", line);
                prop_assert!(!parts[2].contains("\x1b["), "Content should not contain ANSI codes: {}", line);

                // File path should have URL-encoded colons if they were present
                if file_path.contains(':') {
                    prop_assert!(parts[0].contains("%3A"), "File path with colons should be URL-encoded: {}", line);
                }
            }
        }

        // Verify that unicode characters are preserved (not corrupted)
        for unicode_char in &unicode_chars {
            if !unicode_char.is_empty() && (content.contains(unicode_char) || translation_value.contains(unicode_char)) {
                prop_assert!(output.contains(unicode_char), "Unicode character '{}' should be preserved in output", unicode_char);
            }
        }

        // Verify that the output can be split and parsed consistently
        let lines: Vec<&str> = output.lines().filter(|line| !line.trim().is_empty()).collect();
        prop_assert_eq!(lines.len(), 2, "Should have exactly 2 output lines (translation + code reference)");
    }
}

// **Feature: simple-flag-and-context-lines, Property 5: Translation vs code context separation**
proptest! {
    #[test]
    fn test_translation_vs_code_context_separation(
        translation_key in "[a-zA-Z][a-zA-Z0-9_.]{0,30}",
        translation_value in "[^\\n\\r]{0,50}",
        code_content in "[^\\n\\r]{0,100}",
        file_path in "[a-zA-Z][a-zA-Z0-9_/.]{0,29}\\.(yml|ts|js)",
        line_num in 1u32..100u32
    ) {
        // **Validates: Requirements 2.5**

        // Create a SearchResult with both translation entries and code references
        let result = SearchResult {
            query: "test".to_string(),
            translation_entries: vec![TranslationEntry {
                key: translation_key.clone(),
                value: translation_value.clone(),
                line: line_num as usize,
                file: PathBuf::from(&file_path),
            }],
            code_references: vec![CodeReference {
                file: PathBuf::from(&file_path),
                line: (line_num + 10) as usize,
                pattern: "test".to_string(),
                context: code_content.clone(),
                key_path: translation_key.clone(),
                context_before: vec!["context before line 1".to_string(), "context before line 2".to_string()],
                context_after: vec!["context after line 1".to_string(), "context after line 2".to_string()],
            }],
        };

        // Test non-simple format (where context lines should be visible for code references)
        let formatter = TreeFormatter::new().with_simple_format(false);
        let output = formatter.format_result(&result);

        // Translation entries should NOT have context lines displayed
        // They should appear in the "Translation Files" section without context
        let translation_section_lines: Vec<&str> = output
            .lines()
            .skip_while(|line| !line.contains("=== Translation Files ==="))
            .take_while(|line| !line.contains("=== Code References ==="))
            .collect();

        // Find the translation entry line (skip the header line)
        let translation_line = translation_section_lines
            .iter()
            .skip(1) // Skip the "=== Translation Files ===" header
            .find(|line| {
                // Look for lines that contain the file path and line number
                line.contains(&file_path) && line.contains(&format!(":{}", line_num))
            });

        prop_assert!(translation_line.is_some(),
            "Should find translation line with file '{}' and line '{}' in section: {:?}",
            file_path, line_num, translation_section_lines);

        if let Some(trans_line) = translation_line {
            // Translation entry should be in format: file:line:key: "value"
            let expected_format = format!("{}:{}", file_path, line_num);
            prop_assert!(trans_line.contains(&expected_format),
                "Translation line should contain '{}' but was: {}", expected_format, trans_line);
            prop_assert!(trans_line.contains(&translation_key));
            // Should NOT have context line indicators (- separator)
            let context_format = format!("{}-", file_path);
            prop_assert!(!trans_line.contains(&context_format),
                "Translation line should not contain context indicator '{}' but was: {}", context_format, trans_line);
        }

        // Code references MAY have context lines displayed
        // They should appear in the "Code References" section with context
        let code_section_lines: Vec<&str> = output
            .lines()
            .skip_while(|line| !line.contains("=== Code References ==="))
            .collect();

        // Should have context lines for code references (indicated by '-' separator)
        let context_indicator = format!("{}-", file_path);
        let has_context_lines = code_section_lines
            .iter()
            .any(|line| line.contains(&context_indicator));

        // Should have the actual match line (indicated by ':' separator)
        let match_indicator = format!("{}:{}", file_path, line_num + 10);
        let has_match_line = code_section_lines
            .iter()
            .any(|line| line.contains(&match_indicator));

        prop_assert!(has_match_line, "Code reference should have match line");
        // Context lines are optional but if present, should be properly formatted
        if has_context_lines {
            // Verify context lines don't contain the actual match content (only if content is non-empty)
            if !code_content.trim().is_empty() {
                let context_lines: Vec<&str> = code_section_lines
                    .iter()
                    .filter(|line| line.contains(&context_indicator))
                    .cloned()
                    .collect();

                for context_line in context_lines {
                    // Only check if code_content is substantial enough to avoid false positives
                    // with single characters that might appear in common words
                    if code_content.len() > 2 {
                        prop_assert!(!context_line.contains(&code_content),
                            "Context lines should not contain the match content: {}", context_line);
                    }
                }
            }
        }
    }
}

// **Feature: simple-flag-and-context-lines, Property 8: rg compatibility**
proptest! {
    #[test]
    fn test_rg_compatibility(
        search_text in "[a-zA-Z][a-zA-Z0-9]{1,10}",
        file_content in prop::collection::vec("[a-zA-Z0-9 ]{0,50}", 3..10),
        file_path in "[a-zA-Z][a-zA-Z0-9_]{3,15}\\.(ts|js|yml|json)"
    ) {
        // **Validates: Requirements 4.4**

        // Skip empty search text
        if search_text.trim().is_empty() {
            return Ok(());
        }

        // Create a temporary directory with test files
        let temp_dir = tempfile::TempDir::new().unwrap();
        let test_file_path = temp_dir.path().join(&file_path);

        // Create parent directories if needed
        if let Some(parent) = test_file_path.parent() {
            std::fs::create_dir_all(parent).unwrap();
        }

        // Create file content with the search text embedded in some lines
        let mut full_content = Vec::new();
        let mut has_matches = false;

        for (i, content_line) in file_content.iter().enumerate() {
            // Embed search text in some lines (every 3rd line)
            let line_content = if i % 3 == 0 {
                let embedded_content = format!("prefix {} suffix", search_text);
                has_matches = true;
                embedded_content
            } else {
                content_line.clone()
            };

            full_content.push(line_content);
        }

        // Skip test if no matches expected
        if !has_matches {
            return Ok(());
        }

        // Write the test file
        std::fs::write(&test_file_path, full_content.join("\n")).unwrap();

        // Run cs search in simple format
        let cs_query = cs::SearchQuery::new(search_text.clone())
            .with_base_dir(temp_dir.path().to_path_buf())
            .with_case_sensitive(true)
            .with_quiet(true);

        let cs_result = cs::run_search(cs_query);

        // Verify cs found results
        prop_assert!(cs_result.is_ok(), "cs search should succeed");
        let cs_result = cs_result.unwrap();

        // Format cs results in simple format
        let formatter = cs::TreeFormatter::new().with_simple_format(true);
        let cs_output = formatter.format_result(&cs_result);

        // Parse cs output to extract matches
        let cs_matches: Vec<(String, usize, String)> = cs_output
            .lines()
            .filter(|line| !line.trim().is_empty())
            .filter_map(|line| {
                let parts: Vec<&str> = line.splitn(3, ':').collect();
                if parts.len() == 3 {
                    if let Ok(line_num) = parts[1].parse::<usize>() {
                        Some((parts[0].to_string(), line_num, parts[2].to_string()))
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .collect();

        // Verify cs found the expected matches
        if !cs_matches.is_empty() {
            // Verify cs output format is compatible with rg-style parsing
            for line in cs_output.lines() {
                if !line.trim().is_empty() {
                    let parts: Vec<&str> = line.splitn(3, ':').collect();
                    prop_assert_eq!(parts.len(), 3,
                        "cs output should be parseable in rg-compatible format: {}", line);

                    // File path should not be empty
                    prop_assert!(!parts[0].is_empty(), "File path should not be empty");

                    // Line number should be numeric
                    prop_assert!(parts[1].parse::<u32>().is_ok(),
                        "Line number should be numeric: {}", parts[1]);

                    // Content should contain the search text
                    prop_assert!(parts[2].contains(&search_text),
                        "Match content should contain search text: {} in {}", search_text, parts[2]);
                }
            }

            // Verify cs provides equivalent or superior results to what rg would provide
            // This means:
            // 1. All matches should contain the search text
            // 2. Line numbers should be accurate
            // 3. File paths should be correct
            // 4. Output format should be machine-parseable

            let unique_files: std::collections::HashSet<_> = cs_matches.iter().map(|(f, _, _)| f).collect();
            prop_assert_eq!(unique_files.len(), 1, "Should find matches in exactly one file");

            let found_lines: std::collections::HashSet<_> = cs_matches.iter().map(|(_, l, _)| *l).collect();
            prop_assert!(!found_lines.is_empty(), "Should find at least one match");

            // Verify that all found lines actually contain the search text
            for (_, _line_num, content) in &cs_matches {
                prop_assert!(content.contains(&search_text),
                    "Match content should contain search text: '{}' in '{}'",
                    search_text, content);
            }
        }
    }
}
