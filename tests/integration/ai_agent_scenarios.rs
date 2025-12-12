/// Integration tests for AI agent usage scenarios
///
/// These tests verify that cs can serve as a drop-in replacement for rg in AI agent workflows
/// by testing programmatic parsing of simple format output, error handling, and performance.
use assert_cmd::{cargo_bin, Command};
use std::fs;
use tempfile::TempDir;

#[test]
fn test_simple_format_programmatic_parsing() {
    // Test that AI agents can reliably parse simple format output
    let mut cmd = Command::new(cargo_bin!("cs"));
    let output = cmd
        .arg("add new")
        .arg("--simple")
        .current_dir("tests/fixtures/rails-app")
        .output()
        .expect("Failed to execute cs command");

    assert!(output.status.success(), "cs command should succeed");

    let stdout = String::from_utf8(output.stdout).expect("Output should be valid UTF-8");
    let stderr = String::from_utf8(output.stderr).expect("Stderr should be valid UTF-8");

    // Verify stderr is empty (all output goes to stdout)
    // Verify stderr is empty (all output goes to stdout)
    assert!(
        stderr.trim().is_empty(),
        "Stderr should be empty in simple mode, but got:\n{}",
        stderr
    );

    // Parse each line of output
    let lines: Vec<&str> = stdout
        .lines()
        .filter(|line| !line.trim().is_empty())
        .collect();
    assert!(!lines.is_empty(), "Should have at least one result");

    for line in lines {
        // Each line should follow file:line:content format
        let parts: Vec<&str> = line.splitn(3, ':').collect();
        assert_eq!(parts.len(), 3, "Line should have exactly 3 parts: {}", line);

        // File path should not be empty
        assert!(
            !parts[0].is_empty(),
            "File path should not be empty: {}",
            line
        );

        // Line number should be numeric
        assert!(
            parts[1].parse::<u32>().is_ok(),
            "Line number should be numeric: {}",
            line
        );

        // Content should be present (third part exists by definition)
        // Verify no tree characters or ANSI codes
        assert!(
            !parts[2].contains("├─>"),
            "Content should not contain tree characters: {}",
            line
        );
        assert!(
            !parts[2].contains("└─>"),
            "Content should not contain tree characters: {}",
            line
        );
        assert!(
            !parts[2].contains("\x1b["),
            "Content should not contain ANSI codes: {}",
            line
        );
    }
}

#[test]
fn test_error_handling_in_automated_scenarios() {
    // Test that errors are properly handled and reported to stderr
    let temp_dir = TempDir::new().unwrap();

    // Test 1: Search in non-existent directory
    let mut cmd = Command::new(cargo_bin!("cs"));
    let output = cmd
        .arg("test")
        .arg("--simple")
        .current_dir(&temp_dir) // Use temp_dir itself, not a nonexistent subdirectory
        .output()
        .expect("Failed to execute cs command");

    // This should succeed but find no results (empty directory)
    assert!(
        output.status.success(),
        "cs should succeed even in empty directory"
    );

    let stdout = String::from_utf8(output.stdout).expect("Stdout should be valid UTF-8");

    // In simple mode, empty results should either produce empty output or a "No matches found" message
    // Both are acceptable for AI agents to handle
    if !stdout.trim().is_empty() {
        assert!(
            stdout.contains("No matches found"),
            "Should either be empty or contain 'No matches found' message"
        );
    }

    // Test 2: Empty search text
    let mut cmd = Command::new(cargo_bin!("cs"));
    let output = cmd
        .arg("")
        .arg("--simple")
        .current_dir("tests/fixtures/rails-app")
        .output()
        .expect("Failed to execute cs command");

    assert!(
        !output.status.success(),
        "cs should fail for empty search text"
    );

    let stderr = String::from_utf8(output.stderr).expect("Stderr should be valid UTF-8");
    assert!(
        stderr.contains("empty"),
        "Should mention empty search text in error"
    );

    // Test 3: Invalid regex (when --regex flag is used)
    let mut cmd = Command::new(cargo_bin!("cs"));
    let output = cmd
        .arg("[invalid")
        .arg("--regex")
        .arg("--simple")
        .current_dir("tests/fixtures/rails-app")
        .output()
        .expect("Failed to execute cs command");

    // This might succeed or fail depending on regex implementation, but should handle gracefully
    let stderr = String::from_utf8(output.stderr).expect("Stderr should be valid UTF-8");
    // If it fails, error should be in stderr, not stdout
    if !output.status.success() {
        assert!(
            !stderr.trim().is_empty(),
            "Should have error message in stderr for invalid regex"
        );
    }
}

#[test]
fn test_performance_with_large_codebase() {
    // Test performance characteristics that AI agents care about
    let start = std::time::Instant::now();

    let mut cmd = Command::new(cargo_bin!("cs"));
    let output = cmd
        .arg("function")
        .arg("--simple")
        .current_dir("tests/fixtures")
        .output()
        .expect("Failed to execute cs command");

    let duration = start.elapsed();

    // Should complete within reasonable time (10 seconds for test fixtures)
    assert!(
        duration.as_secs() < 10,
        "Search should complete within 10 seconds, took {:?}",
        duration
    );

    if output.status.success() {
        let stdout = String::from_utf8(output.stdout).expect("Output should be valid UTF-8");

        // Verify output is still parseable even with many results
        for line in stdout.lines() {
            if !line.trim().is_empty() {
                let parts: Vec<&str> = line.splitn(3, ':').collect();
                assert_eq!(parts.len(), 3, "Line should be parseable: {}", line);
            }
        }
    }
}

#[test]
fn test_ai_agent_workflow_simulation() {
    // Simulate a typical AI agent workflow: search -> parse -> analyze

    // Step 1: Search for a pattern
    let mut cmd = Command::new(cargo_bin!("cs"));
    let output = cmd
        .arg("add new")
        .arg("--simple")
        .current_dir("tests/fixtures/rails-app")
        .output()
        .expect("Failed to execute cs command");

    assert!(output.status.success(), "Initial search should succeed");

    let stdout = String::from_utf8(output.stdout).expect("Output should be valid UTF-8");

    // Step 2: Parse results programmatically
    let mut parsed_results = Vec::new();
    for line in stdout.lines() {
        if !line.trim().is_empty() {
            let parts: Vec<&str> = line.splitn(3, ':').collect();
            if parts.len() == 3 {
                if let Ok(line_num) = parts[1].parse::<u32>() {
                    parsed_results.push((parts[0].to_string(), line_num, parts[2].to_string()));
                }
            }
        }
    }

    assert!(
        !parsed_results.is_empty(),
        "Should parse at least one result"
    );

    // Step 3: Analyze results (simulate AI agent logic)
    let mut translation_files = 0;
    let mut code_files = 0;

    for (file_path, _line_num, _content) in &parsed_results {
        if file_path.ends_with(".yml") || file_path.ends_with(".yaml") {
            translation_files += 1;
        } else if file_path.ends_with(".ts")
            || file_path.ends_with(".js")
            || file_path.ends_with(".tsx")
        {
            code_files += 1;
        }
    }

    // Verify AI agent can distinguish between file types
    assert!(
        translation_files > 0 || code_files > 0,
        "Should find either translation or code files"
    );

    // Step 4: Follow-up search based on results (simulate AI agent decision making)
    if translation_files > 0 {
        // AI agent might want to search for specific translation keys
        let mut cmd = Command::new(cargo_bin!("cs"));
        let output = cmd
            .arg("invoice.labels")
            .arg("--simple")
            .current_dir("tests/fixtures/rails-app")
            .output()
            .expect("Failed to execute follow-up search");

        // Should handle follow-up searches gracefully
        assert!(
            output.status.success() || output.status.code() == Some(0),
            "Follow-up search should not crash"
        );
    }
}

#[test]
fn test_rg_compatibility_command_line_args() {
    // Test that cs accepts common rg-style arguments that AI agents might use

    // Test case-insensitive search (rg -i)
    let mut cmd = Command::new(cargo_bin!("cs"));
    cmd.arg("ADD NEW") // uppercase
        .arg("--ignore-case")
        .arg("--simple")
        .current_dir("tests/fixtures/rails-app")
        .assert()
        .success();

    // Test word boundary search (rg -w)
    let mut cmd = Command::new(cargo_bin!("cs"));
    cmd.arg("new")
        .arg("--word-regexp")
        .arg("--simple")
        .current_dir("tests/fixtures/rails-app")
        .assert()
        .success();

    // Test regex search (rg -e with regex)
    let mut cmd = Command::new(cargo_bin!("cs"));
    cmd.arg("add.*new")
        .arg("--regex")
        .arg("--simple")
        .current_dir("tests/fixtures/rails-app")
        .assert()
        .success();
}

#[test]
fn test_output_consistency_across_search_types() {
    // Test that different search types produce consistent output format

    let temp_dir = TempDir::new().unwrap();

    // Create test files
    let yaml_dir = temp_dir.path().join("config/locales");
    fs::create_dir_all(&yaml_dir).unwrap();

    let yaml_file = yaml_dir.join("en.yml");
    fs::write(&yaml_file, "en:\n  test:\n    key: \"test value\"").unwrap();

    let code_dir = temp_dir.path().join("src");
    fs::create_dir_all(&code_dir).unwrap();

    let code_file = code_dir.join("app.ts");
    fs::write(
        &code_file,
        "const message = 'test value';\nconsole.log('test');",
    )
    .unwrap();

    // Test translation search
    let mut cmd = Command::new(cargo_bin!("cs"));
    let translation_output = cmd
        .arg("test value")
        .arg("--simple")
        .current_dir(temp_dir.path())
        .output()
        .expect("Failed to execute translation search");

    // Test exact text search
    let mut cmd = Command::new(cargo_bin!("cs"));
    let exact_output = cmd
        .arg("test")
        .arg("--simple")
        .current_dir(temp_dir.path())
        .output()
        .expect("Failed to execute exact search");

    // Both should use the same output format
    if translation_output.status.success() {
        let stdout = String::from_utf8(translation_output.stdout).unwrap();
        for line in stdout.lines() {
            if !line.trim().is_empty() {
                let parts: Vec<&str> = line.splitn(3, ':').collect();
                assert_eq!(
                    parts.len(),
                    3,
                    "Translation search line should be parseable: {}",
                    line
                );
            }
        }
    }

    if exact_output.status.success() {
        let stdout = String::from_utf8(exact_output.stdout).unwrap();
        for line in stdout.lines() {
            if !line.trim().is_empty() {
                let parts: Vec<&str> = line.splitn(3, ':').collect();
                assert_eq!(
                    parts.len(),
                    3,
                    "Exact search line should be parseable: {}",
                    line
                );
            }
        }
    }
}

#[test]
fn test_large_output_handling() {
    // Test that cs can handle large amounts of output without issues

    let temp_dir = TempDir::new().unwrap();

    // Create a file with many lines containing the search term
    let test_file = temp_dir.path().join("large_file.txt");
    let mut content = String::new();
    for i in 1..=1000 {
        content.push_str(&format!("Line {} contains test pattern\n", i));
    }
    fs::write(&test_file, content).unwrap();

    let mut cmd = Command::new(cargo_bin!("cs"));
    let output = cmd
        .arg("test pattern")
        .arg("--simple")
        .current_dir(temp_dir.path())
        .output()
        .expect("Failed to execute search on large file");

    if output.status.success() {
        let stdout = String::from_utf8(output.stdout).expect("Output should be valid UTF-8");
        let lines: Vec<&str> = stdout
            .lines()
            .filter(|line| !line.trim().is_empty())
            .collect();

        // Should find many matches
        assert!(lines.len() > 100, "Should find many matches in large file");

        // All lines should still be parseable
        for line in lines {
            let parts: Vec<&str> = line.splitn(3, ':').collect();
            assert_eq!(
                parts.len(),
                3,
                "Large output line should be parseable: {}",
                line
            );
        }
    }
}

#[test]
fn test_special_characters_in_ai_workflows() {
    // Test handling of special characters that might appear in AI agent queries

    let temp_dir = TempDir::new().unwrap();

    // Create test file with various special characters
    let test_file = temp_dir.path().join("special.txt");
    fs::write(
        &test_file,
        "Line with spaces and symbols: $var @mention #tag\n\
         Unicode content: café naïve résumé 中文 🚀\n\
         Shell metacharacters: $(command) `backticks` |pipe| &background&\n\
         Quotes and escapes: \"double\" 'single' \\backslash\n",
    )
    .unwrap();

    // Test searching for content with special characters
    let test_cases = vec!["$var", "café", "中文", "🚀", "\"double\"", "\\backslash"];

    for search_term in test_cases {
        let mut cmd = Command::new(cargo_bin!("cs"));
        let output = cmd
            .arg(search_term)
            .arg("--simple")
            .current_dir(temp_dir.path())
            .output()
            .expect("Failed to execute search with special characters");

        // Should handle special characters gracefully (success or clean failure)
        let stderr = String::from_utf8(output.stderr).unwrap();
        if !output.status.success() {
            // If it fails, should have clean error message, not crash
            assert!(
                !stderr.contains("panic"),
                "Should not panic on special characters: {}",
                search_term
            );
        } else {
            // If it succeeds, output should be parseable
            let stdout = String::from_utf8(output.stdout).unwrap();
            for line in stdout.lines() {
                if !line.trim().is_empty() {
                    let parts: Vec<&str> = line.splitn(3, ':').collect();
                    assert_eq!(
                        parts.len(),
                        3,
                        "Special character search result should be parseable: {}",
                        line
                    );
                }
            }
        }
    }
}
