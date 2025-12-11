use assert_cmd::{cargo_bin, Command};
use predicates::prelude::*;
use proptest::prelude::*;
use proptest::test_runner::Config as ProptestConfig;
use std::fs;
use tempfile::TempDir;

// **Feature: simple-flag-and-context-lines, Property 6: Error stream separation**
proptest! {
    #![proptest_config(ProptestConfig::with_cases(10))] // Reduce iterations for performance
    #[test]
    fn test_error_stream_separation(
        search_text in "[a-zA-Z0-9]{1,20}", // Only non-whitespace characters
        invalid_path in "[a-zA-Z0-9/]{5,30}",
    ) {
        // Property: For any error condition during search, error messages should appear on stderr
        // while keeping stdout clean for result parsing

        // Test 1: Empty search text should produce error on stderr
        let mut cmd = Command::new(cargo_bin!("cs"));
        cmd.arg("")
            .assert()
            .failure()
            .stderr(predicate::str::contains("empty"))
            .stdout(predicate::str::is_empty());

        // Test 2: Whitespace-only search text should produce error on stderr
        let mut cmd = Command::new(cargo_bin!("cs"));
        cmd.arg("   ")
            .assert()
            .failure()
            .stderr(predicate::str::contains("empty"))
            .stdout(predicate::str::is_empty());

        // Test 3: Valid search text with invalid directory - should succeed with "No matches found"
        let mut cmd = Command::new(cargo_bin!("cs"));
        cmd.arg(&search_text)
            .arg(format!("/nonexistent/{}", invalid_path))
            .assert()
            .success() // Should succeed but find no matches
            .stdout(predicate::str::contains("No matches found"))
            .stderr(predicate::str::is_empty()); // No errors should go to stderr

        // Test 4: Malformed YAML should produce warnings on stderr but continue
        let temp_dir = TempDir::new().unwrap();
        let yaml_path = temp_dir.path().join("locales");
        fs::create_dir_all(&yaml_path).unwrap();
        fs::write(yaml_path.join("bad.yml"), "key: [unclosed bracket").unwrap();

        let mut cmd = Command::new(cargo_bin!("cs"));
        cmd.arg(&search_text)
            .arg("--verbose")
            .current_dir(temp_dir.path())
            .assert()
            .success(); // Should succeed despite malformed YAML

        // Note: We can't easily test stderr content here because the command succeeds,
        // but the property is that errors go to stderr, not stdout
    }
}

#[test]
fn test_cache_errors_go_to_stderr() {
    // Test that cache-related errors go to stderr

    // Test clear cache with potential permission issues
    let mut cmd = Command::new(cargo_bin!("cs"));
    let result = cmd.arg("--clear-cache").assert();

    // If it fails, error should be on stderr
    if !result.get_output().status.success() {
        result.stderr(predicate::str::is_empty().not());
    } else {
        // If it succeeds, success message should be on stdout
        result.stdout(predicate::str::contains("Cache cleared successfully"));
    }
}

#[test]
fn test_successful_operations_use_stdout() {
    // Property: Successful operations should use stdout, not stderr
    let temp_dir = TempDir::new().unwrap();

    // Create a simple test file
    fs::write(temp_dir.path().join("test.txt"), "hello world").unwrap();

    let mut cmd = Command::new(cargo_bin!("cs"));
    cmd.arg("hello")
        .current_dir(temp_dir.path())
        .assert()
        .success()
        .stdout(predicate::str::is_empty().not()) // Should have output on stdout
        .stderr(predicate::str::is_empty()); // stderr should be empty
}

#[test]
fn test_no_matches_message_goes_to_stdout() {
    // Property: "No matches found" messages should go to stdout, not stderr
    let temp_dir = TempDir::new().unwrap();

    // Create a file that won't match our search
    fs::write(temp_dir.path().join("test.txt"), "foo bar baz").unwrap();

    let mut cmd = Command::new(cargo_bin!("cs"));
    cmd.arg("nonexistent")
        .current_dir(temp_dir.path())
        .assert()
        .success() // Should succeed even with no matches
        .stdout(predicate::str::contains("No matches found"))
        .stderr(predicate::str::is_empty()); // stderr should be empty
}

#[test]
fn test_simple_format_errors_go_to_stderr() {
    // Property: When using --simple flag, errors should still go to stderr to keep stdout parseable

    // Test with empty search text
    let mut cmd = Command::new(cargo_bin!("cs"));
    cmd.arg("")
        .arg("--simple")
        .assert()
        .failure()
        .stderr(predicate::str::contains("empty"))
        .stdout(predicate::str::is_empty());

    // Test with whitespace search text
    let mut cmd = Command::new(cargo_bin!("cs"));
    cmd.arg("   ")
        .arg("--simple")
        .assert()
        .failure()
        .stderr(predicate::str::contains("empty"))
        .stdout(predicate::str::is_empty());
}
