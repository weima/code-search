use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

#[test]
fn test_no_matches_shows_simple_message() {
    // When no translation files are found, show a simple "no matches" message
    // This is NOT an error - it's a valid state
    let temp_dir = TempDir::new().unwrap();

    let mut cmd = Command::cargo_bin("cs").unwrap();
    cmd.arg("nonexistent text")
        .current_dir(temp_dir.path())
        .assert()
        .success() // Exit code 0
        .stdout(predicate::str::contains("No matches found"));
}

#[test]
fn test_yaml_parse_error_message() {
    // Create invalid YAML file
    let temp_dir = TempDir::new().unwrap();
    let yaml_path = temp_dir.path().join("invalid.yml");
    fs::write(&yaml_path, "key: [invalid yaml structure").unwrap();

    let mut cmd = Command::cargo_bin("cs").unwrap();
    cmd.arg("test")
        .current_dir(temp_dir.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("Error:"))
        .stderr(predicate::str::contains("Failed to parse YAML file"))
        .stderr(predicate::str::contains("Reason:"))
        .stderr(predicate::str::contains("Next steps:"))
        .stderr(predicate::str::contains("YAML syntax"))
        .stderr(predicate::str::contains("indentation"));
}

#[test]
fn test_yaml_parse_error_has_helpful_suggestions() {
    // Test that YAML parse errors include helpful next steps
    let temp_dir = TempDir::new().unwrap();
    let yaml_path = temp_dir.path().join("bad.yml");
    fs::write(&yaml_path, "key: [unclosed").unwrap();

    let mut cmd = Command::cargo_bin("cs").unwrap();
    cmd.arg("test")
        .current_dir(temp_dir.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("Next steps:"));
}

#[test]
fn test_yaml_parse_error_includes_reason() {
    // Test that YAML parse errors include the specific reason
    let temp_dir = TempDir::new().unwrap();
    let yaml_path = temp_dir.path().join("invalid.yml");
    fs::write(&yaml_path, "key: {broken").unwrap();

    let mut cmd = Command::cargo_bin("cs").unwrap();
    cmd.arg("test")
        .current_dir(temp_dir.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("Reason:"));
}

#[test]
fn test_yaml_parse_error_shows_common_issues() {
    // Test that YAML errors mention common problems
    let temp_dir = TempDir::new().unwrap();
    let yaml_path = temp_dir.path().join("test.yml");
    fs::write(&yaml_path, "bad: [yaml").unwrap();

    let mut cmd = Command::cargo_bin("cs").unwrap();
    cmd.arg("test")
        .current_dir(temp_dir.path())
        .assert()
        .failure()
        .stderr(
            predicate::str::contains("indentation")
                .or(predicate::str::contains("quotes"))
                .or(predicate::str::contains("brackets")),
        );
}
