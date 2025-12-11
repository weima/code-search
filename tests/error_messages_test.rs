use assert_cmd::{cargo_bin, Command};
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

#[test]
fn test_no_matches_shows_simple_message() {
    // When no translation files are found, show a simple "no matches" message
    // This is NOT an error - it's a valid state
    let temp_dir = TempDir::new().unwrap();

    let mut cmd = Command::new(cargo_bin!("cs"));
    cmd.arg("nonexistent text")
        .current_dir(temp_dir.path())
        .assert()
        .success() // Exit code 0
        .stdout(predicate::str::contains("No matches found"));
}

#[test]
fn test_yaml_parse_error_is_handled_gracefully() {
    // Create invalid YAML file
    let temp_dir = TempDir::new().unwrap();
    let yaml_path = temp_dir.path().join("invalid.yml");
    fs::write(&yaml_path, "key: [invalid yaml structure test").unwrap();

    let mut cmd = Command::new(cargo_bin!("cs"));
    cmd.arg("--verbose") // Enable verbose mode
        .arg("test")
        .current_dir(temp_dir.path())
        .assert()
        .success() // Should NOT fail
        .stdout(predicate::str::contains("test")); // Should find the match despite invalid structure
}

#[test]
fn test_json_parse_error_is_handled_gracefully() {
    // Create invalid JSON file
    let temp_dir = TempDir::new().unwrap();
    let json_path = temp_dir.path().join("invalid.json");
    fs::write(&json_path, "{ key: 'invalid json test' }").unwrap(); // Invalid JSON

    let mut cmd = Command::new(cargo_bin!("cs"));
    cmd.arg("--verbose") // Enable verbose mode
        .arg("test")
        .current_dir(temp_dir.path())
        .assert()
        .success() // Should NOT fail
        .stdout(predicate::str::contains("test")); // Should find the match despite invalid structure
}
