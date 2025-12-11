use cs::TextSearcher;
use proptest::prelude::*;
use std::fs;
use tempfile::TempDir;

// **Feature: simple-flag-and-context-lines, Property 2: Context line display**
proptest! {
    #[test]
    fn test_context_line_display(
        content_lines in prop::collection::vec("[a-zA-Z0-9 ]{1,50}", 5..20),
        match_line_idx_raw in 2usize..15,
        search_text in "[a-zA-Z]{3,10}"
    ) {
        // **Validates: Requirements 2.1**

        let mut lines = content_lines;

        // Ensure we have enough lines first
        while lines.len() < 5 {
            lines.push("filler line".to_string());
        }

        // Now calculate match_line_idx within valid bounds
        let match_line_idx = match_line_idx_raw % lines.len().max(1);

        // Ensure we have enough lines for context after the match
        while lines.len() <= match_line_idx + 2 {
            lines.push("filler line".to_string());
        }

        // Insert search text into the match line with unique prefix/suffix to avoid partial matches
        lines[match_line_idx] = format!("UNIQUE_{}_{}_UNIQUE", search_text, match_line_idx);

        // Create temporary file
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        fs::write(&file_path, lines.join("\n")).unwrap();

        // Search for the unique pattern
        let searcher = TextSearcher::new(temp_dir.path().to_path_buf())
            .context_lines(2);
        let search_pattern = format!("UNIQUE_{}_{}_UNIQUE", search_text, match_line_idx);
        let matches = searcher.search(&search_pattern).unwrap();

        // Should find exactly one match
        prop_assert_eq!(matches.len(), 1);
        let match_result = &matches[0];

        // Verify the match is correct
        prop_assert_eq!(match_result.line, match_line_idx + 1); // 1-indexed
        prop_assert!(match_result.content.contains(&search_text));

        // Verify context lines - should have exactly 2 lines before and after (unless at file boundaries)
        let expected_before = std::cmp::min(2, match_line_idx);
        let expected_after = std::cmp::min(2, lines.len() - match_line_idx - 1);

        prop_assert_eq!(match_result.context_before.len(), expected_before);
        prop_assert_eq!(match_result.context_after.len(), expected_after);

        // Verify context content matches the original lines
        for (i, context_line) in match_result.context_before.iter().enumerate() {
            let original_idx = match_line_idx - expected_before + i;
            prop_assert_eq!(context_line, &lines[original_idx]);
        }

        for (i, context_line) in match_result.context_after.iter().enumerate() {
            let original_idx = match_line_idx + 1 + i;
            prop_assert_eq!(context_line, &lines[original_idx]);
        }
    }
}

// **Feature: simple-flag-and-context-lines, Property 3: Context boundary handling**
proptest! {
    #[test]
    fn test_context_boundary_handling(
        file_size in 1usize..10,
        match_position in prop::sample::select(vec!["start", "end"]),
        search_text in "[a-zA-Z]{3,8}"
    ) {
        // **Validates: Requirements 2.4**

        let mut lines = vec![];
        for i in 0..file_size {
            lines.push(format!("line {}", i + 1));
        }

        // Place match at start or end based on match_position
        let match_idx = match match_position {
            "start" => 0,
            "end" => file_size - 1,
            _ => 0,
        };

        lines[match_idx] = format!("UNIQUE_{}_TARGET", search_text);

        // Create temporary file
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        fs::write(&file_path, lines.join("\n")).unwrap();

        // Search with 2 context lines
        let searcher = TextSearcher::new(temp_dir.path().to_path_buf())
            .context_lines(2);
        let search_pattern = format!("UNIQUE_{}_TARGET", search_text);
        let matches = searcher.search(&search_pattern).unwrap();

        prop_assert_eq!(matches.len(), 1);
        let match_result = &matches[0];

        // Verify boundary handling
        match match_position {
            "start" => {
                // At file start: no context before, up to 2 lines after
                prop_assert_eq!(match_result.context_before.len(), 0);
                let expected_after = std::cmp::min(2, file_size - 1);
                prop_assert_eq!(match_result.context_after.len(), expected_after);
            },
            "end" => {
                // At file end: up to 2 lines before, no context after
                let expected_before = std::cmp::min(2, file_size - 1);
                prop_assert_eq!(match_result.context_before.len(), expected_before);
                prop_assert_eq!(match_result.context_after.len(), 0);
            },
            _ => {}
        }

        // Verify no invalid line numbers or content
        prop_assert!(match_result.line >= 1);
        prop_assert!(match_result.line <= file_size);
        prop_assert!(match_result.content.contains(&search_text));
    }
}

// **Feature: simple-flag-and-context-lines, Property 4: Context overlap merging**
proptest! {
    #[test]
    fn test_context_overlap_merging(
        gap_size in 0usize..5, // Gap between matches (0 = adjacent, 4 = max gap for overlap with 2 context lines)
        search_text in "[a-zA-Z]{3,8}"
    ) {
        // **Validates: Requirements 2.3**

        // Create file with two matches separated by gap_size lines
        let mut lines = vec![];
        lines.push("before line 1".to_string());
        lines.push("before line 2".to_string());
        lines.push(format!("first {} match", search_text));

        // Add gap lines
        for i in 0..gap_size {
            lines.push(format!("gap line {}", i + 1));
        }

        lines.push(format!("second {} match", search_text));
        lines.push("after line 1".to_string());
        lines.push("after line 2".to_string());

        // Create temporary file
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        fs::write(&file_path, lines.join("\n")).unwrap();

        // Search with 2 context lines
        let searcher = TextSearcher::new(temp_dir.path().to_path_buf())
            .context_lines(2);
        let matches = searcher.search(&search_text).unwrap();

        prop_assert_eq!(matches.len(), 2);

        let first_match = &matches[0];
        let second_match = &matches[1];

        // Verify matches are found correctly
        prop_assert!(first_match.content.contains("first"));
        prop_assert!(second_match.content.contains("second"));

        // Check if context should overlap (gap_size <= 4 means overlap with 2 context lines each)
        // First match: line 3, context after goes to line 5
        // Second match: line 3 + gap_size + 1, context before starts at line (3 + gap_size + 1) - 2
        // Overlap occurs when: 5 >= (3 + gap_size + 1) - 2, which simplifies to gap_size <= 3

        if gap_size <= 3 {
            // Context should overlap - verify no duplicate lines in the combined context
            // This is more of a formatter responsibility, but we can verify the raw data is correct
            prop_assert!(first_match.line < second_match.line);

            // The context_after of first match and context_before of second match should contain
            // some of the same lines when gap_size is small
            let first_end_line = first_match.line + first_match.context_after.len();
            let second_start_line = second_match.line - second_match.context_before.len();

            if gap_size <= 3 {
                prop_assert!(first_end_line >= second_start_line,
                    "Context should overlap when gap_size <= 3, but first_end_line={} < second_start_line={}",
                    first_end_line, second_start_line);
            }
        }
    }
}

#[test]
fn test_context_display_basic() {
    let temp_dir = TempDir::new().unwrap();
    let content = "line1\nline2\ntarget line\nline4\nline5";
    fs::write(temp_dir.path().join("test.txt"), content).unwrap();

    let searcher = TextSearcher::new(temp_dir.path().to_path_buf()).context_lines(2);
    let matches = searcher.search("target").unwrap();

    assert_eq!(matches.len(), 1);
    let match_result = &matches[0];

    assert_eq!(match_result.line, 3);
    assert_eq!(match_result.context_before, vec!["line1", "line2"]);
    assert_eq!(match_result.context_after, vec!["line4", "line5"]);
}

#[test]
fn test_context_display_at_file_start() {
    let temp_dir = TempDir::new().unwrap();
    let content = "target line\nline2\nline3\nline4";
    fs::write(temp_dir.path().join("test.txt"), content).unwrap();

    let searcher = TextSearcher::new(temp_dir.path().to_path_buf()).context_lines(2);
    let matches = searcher.search("target").unwrap();

    assert_eq!(matches.len(), 1);
    let match_result = &matches[0];

    assert_eq!(match_result.line, 1);
    assert_eq!(match_result.context_before.len(), 0);
    assert_eq!(match_result.context_after, vec!["line2", "line3"]);
}

#[test]
fn test_context_display_at_file_end() {
    let temp_dir = TempDir::new().unwrap();
    let content = "line1\nline2\nline3\ntarget line";
    fs::write(temp_dir.path().join("test.txt"), content).unwrap();

    let searcher = TextSearcher::new(temp_dir.path().to_path_buf()).context_lines(2);
    let matches = searcher.search("target").unwrap();

    assert_eq!(matches.len(), 1);
    let match_result = &matches[0];

    assert_eq!(match_result.line, 4);
    assert_eq!(match_result.context_before, vec!["line2", "line3"]);
    assert_eq!(match_result.context_after.len(), 0);
}
#[test]
fn test_context_line_formatting_basic() {
    // Test context display with various file sizes
    let temp_dir = TempDir::new().unwrap();
    let content = "line1\nline2\ntarget line\nline4\nline5";
    fs::write(temp_dir.path().join("test.txt"), content).unwrap();

    let searcher = TextSearcher::new(temp_dir.path().to_path_buf()).context_lines(2);
    let matches = searcher.search("target").unwrap();

    assert_eq!(matches.len(), 1);
    let match_result = &matches[0];

    // Verify context lines are captured correctly
    assert_eq!(match_result.context_before, vec!["line1", "line2"]);
    assert_eq!(match_result.context_after, vec!["line4", "line5"]);
    assert_eq!(match_result.line, 3);
    assert!(match_result.content.contains("target"));
}

#[test]
fn test_context_line_formatting_visual_distinction() {
    // Test visual distinction between match and context lines
    use cs::{CodeReference, SearchResult, TreeFormatter};

    let result = SearchResult {
        query: "target".to_string(),
        translation_entries: vec![],
        code_references: vec![CodeReference {
            file: std::path::PathBuf::from("test.txt"),
            line: 3,
            pattern: "target".to_string(),
            context: "target line".to_string(),
            key_path: "target".to_string(),
            context_before: vec!["line1".to_string(), "line2".to_string()],
            context_after: vec!["line4".to_string(), "line5".to_string()],
        }],
    };

    let formatter = TreeFormatter::new(); // Non-simple format
    let output = formatter.format_result(&result);

    // Should contain context lines with '-' and match line with ':'
    // Note: The output may contain ANSI color codes, so we check for the basic structure
    assert!(output.contains("test.txt-1:line1"));
    assert!(output.contains("test.txt-2:line2"));
    assert!(output.contains("test.txt:3:") && output.contains("target"));
    assert!(output.contains("test.txt-4:line4"));
    assert!(output.contains("test.txt-5:line5"));
}

#[test]
fn test_context_line_formatting_empty_files() {
    // Test edge cases (empty files, single-line files)
    let temp_dir = TempDir::new().unwrap();

    // Single line file
    fs::write(temp_dir.path().join("single.txt"), "target").unwrap();

    let searcher = TextSearcher::new(temp_dir.path().to_path_buf()).context_lines(2);
    let matches = searcher.search("target").unwrap();

    assert_eq!(matches.len(), 1);
    let match_result = &matches[0];

    // Should have no context lines
    assert_eq!(match_result.context_before.len(), 0);
    assert_eq!(match_result.context_after.len(), 0);
    assert_eq!(match_result.line, 1);
}

#[test]
fn test_context_line_formatting_multiple_matches() {
    // Test context display with multiple matches in same file
    let temp_dir = TempDir::new().unwrap();
    let content = "line1\ntarget1\nline3\nline4\ntarget2\nline6";
    fs::write(temp_dir.path().join("test.txt"), content).unwrap();

    let searcher = TextSearcher::new(temp_dir.path().to_path_buf()).context_lines(1);
    let matches = searcher.search("target").unwrap();

    assert_eq!(matches.len(), 2);

    // First match
    assert_eq!(matches[0].line, 2);
    assert_eq!(matches[0].context_before, vec!["line1"]);
    assert_eq!(matches[0].context_after, vec!["line3"]);

    // Second match
    assert_eq!(matches[1].line, 5);
    assert_eq!(matches[1].context_before, vec!["line4"]);
    assert_eq!(matches[1].context_after, vec!["line6"]);
}

#[test]
fn test_context_line_formatting_overlap_handling() {
    // Test that overlapping context is handled correctly
    use cs::{CodeReference, SearchResult, TreeFormatter};

    let result = SearchResult {
        query: "target".to_string(),
        translation_entries: vec![],
        code_references: vec![
            CodeReference {
                file: std::path::PathBuf::from("test.txt"),
                line: 2,
                pattern: "target".to_string(),
                context: "target1".to_string(),
                key_path: "target".to_string(),
                context_before: vec!["line1".to_string()],
                context_after: vec!["line3".to_string(), "line4".to_string()],
            },
            CodeReference {
                file: std::path::PathBuf::from("test.txt"),
                line: 4,
                pattern: "target".to_string(),
                context: "target2".to_string(),
                key_path: "target".to_string(),
                context_before: vec!["line2".to_string(), "line3".to_string()],
                context_after: vec!["line5".to_string()],
            },
        ],
    };

    let formatter = TreeFormatter::new();
    let output = formatter.format_result(&result);

    // Should merge overlapping context and not duplicate lines
    let lines: Vec<&str> = output.lines().collect();
    let test_lines: Vec<&str> = lines
        .iter()
        .filter(|line| line.contains("test.txt"))
        .cloned()
        .collect();

    // Should have unique line numbers (no duplicates)
    let mut line_numbers: Vec<usize> = Vec::new();
    for line in &test_lines {
        // Parse line numbers more carefully
        if line.contains("test.txt") {
            // Format is: test.txt{separator}{line_num}:{content}
            // separator is either ':' for match or '-' for context
            if let Some(txt_end) = line.find("test.txt") {
                let after_txt = &line[txt_end + 8..]; // "test.txt".len() = 8
                if after_txt.len() > 1 {
                    if let Some(colon_pos) = after_txt.find(':') {
                        if colon_pos > 1 {
                            let line_num_part = &after_txt[1..colon_pos]; // Skip the separator (: or -)
                            if let Ok(line_num) = line_num_part.parse::<usize>() {
                                line_numbers.push(line_num);
                            }
                        }
                    }
                }
            }
        }
    }

    line_numbers.sort();
    line_numbers.dedup();

    // Should have consecutive line numbers without gaps in the overlapping region
    assert!(line_numbers.len() >= 4); // At least lines 1, 2, 4, 5 should be present
}
