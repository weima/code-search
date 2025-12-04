#[allow(deprecated)]
use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

#[test]
fn test_no_matches_shows_simple_message() {
    // When no translation files are found, show a simple "no matches" message
    // This is NOT an error - it's a valid state
    let temp_dir = TempDir::new().unwrap();

    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin("cs"));
    cmd.arg("nonexistent text")
        .current_dir(temp_dir.path())
        .assert()
        .success() // Exit code 0
        .stdout(predicate::str::contains("No matches found"));
}

#[test]
fn test_yaml_parse_error_is_warning() {
    // Create invalid YAML file
    let temp_dir = TempDir::new().unwrap();
    let yaml_path = temp_dir.path().join("invalid.yml");
    fs::write(&yaml_path, "key: [invalid yaml structure").unwrap();

    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin("cs"));
    cmd.arg("test")
        .arg("--verbose") // Enable verbose mode to see detailed warnings
        .current_dir(temp_dir.path())
        .assert()
        .success() // Should NOT fail
        .stderr(predicate::str::contains(
            "Warning: Failed to parse YAML file",
        ));
}

#[test]
fn test_json_parse_error_is_warning() {
    // Create invalid JSON file
    let temp_dir = TempDir::new().unwrap();
    let json_path = temp_dir.path().join("invalid.json");
    fs::write(&json_path, "{ key: 'invalid json' }").unwrap(); // Invalid JSON (single quotes, no quotes on key)

    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin("cs"));
    cmd.arg("test")
        .arg("--verbose") // Enable verbose mode to see detailed warnings
        .current_dir(temp_dir.path())
        .assert()
        .success() // Should NOT fail
        .stderr(predicate::str::contains(
            "Warning: Failed to parse JSON file",
        ));
}
