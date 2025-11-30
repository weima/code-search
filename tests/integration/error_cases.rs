/// Integration tests for US-5: Error Handling and Guidance
///
/// These tests verify that the tool provides clear error messages and helpful
/// guidance when searches fail or encounter problems.
use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

// ============================================================================
// US-5: Error Handling and Guidance
// ============================================================================

#[test]
fn test_no_translation_files_found() {
    // Given no translation files found for search text
    // When the tool completes
    // Then I see a clear message

    let temp_dir = TempDir::new().unwrap();

    let mut cmd = Command::cargo_bin("cs").unwrap();
    cmd.arg("search text")
        .current_dir(temp_dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("No matches found"));
}

#[test]
fn test_empty_search_text_rejected() {
    // Given empty search text
    // When I run the tool
    // Then I see an error message about empty input

    let mut cmd = Command::cargo_bin("cs").unwrap();
    cmd.arg("")
        .assert()
        .failure()
        .stderr(predicate::str::contains("empty"));
}

#[test]
fn test_whitespace_only_search_rejected() {
    // Given search text with only whitespace
    // When I run the tool
    // Then I see an error message

    let mut cmd = Command::cargo_bin("cs").unwrap();
    cmd.arg("   ")
        .assert()
        .failure()
        .stderr(predicate::str::contains("empty"));
}

#[test]
fn test_malformed_yaml_shows_clear_error() {
    // Given a malformed YAML file
    // When the tool tries to parse it
    // Then I see a clear error with the file name and issue

    let temp_dir = TempDir::new().unwrap();
    let yaml_path = temp_dir.path().join("config/locales");
    fs::create_dir_all(&yaml_path).unwrap();

    let yaml_file = yaml_path.join("bad.yml");
    fs::write(&yaml_file, "key: [unclosed bracket").unwrap();

    let mut cmd = Command::cargo_bin("cs").unwrap();
    cmd.arg("test")
        .current_dir(temp_dir.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("Error:"))
        .stderr(predicate::str::contains("Failed to parse YAML file"))
        .stderr(predicate::str::contains("bad.yml"));
}

#[test]
fn test_malformed_yaml_suggests_next_steps() {
    // Given a malformed YAML file
    // When the error is shown
    // Then I see suggested next steps

    let temp_dir = TempDir::new().unwrap();
    let yaml_path = temp_dir.path().join("locales");
    fs::create_dir_all(&yaml_path).unwrap();

    fs::write(yaml_path.join("invalid.yml"), "bad: {yaml").unwrap();

    let mut cmd = Command::cargo_bin("cs").unwrap();
    cmd.arg("test")
        .current_dir(temp_dir.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("Next steps:"))
        .stderr(predicate::str::contains("YAML syntax"));
}

#[test]
fn test_malformed_yaml_mentions_common_issues() {
    // Given a malformed YAML file
    // When the error is shown
    // Then common YAML issues are mentioned

    let temp_dir = TempDir::new().unwrap();
    fs::create_dir_all(temp_dir.path().join("locales")).unwrap();
    fs::write(
        temp_dir.path().join("locales/test.yml"),
        "key: [unclosed bracket",
    )
    .unwrap();

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

// ============================================================================
// Edge Cases
// ============================================================================

#[test]
fn test_empty_yaml_file_handled_gracefully() {
    // Given an empty YAML file
    // When I search for text
    // Then the tool handles it without crashing

    let temp_dir = TempDir::new().unwrap();
    let yaml_path = temp_dir.path().join("locales");
    fs::create_dir_all(&yaml_path).unwrap();

    fs::write(yaml_path.join("empty.yml"), "").unwrap();

    let mut cmd = Command::cargo_bin("cs").unwrap();
    cmd.arg("test")
        .current_dir(temp_dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("No matches found"));
}

#[test]
fn test_yaml_with_only_comments() {
    // Given a YAML file with only comments
    // When I search for text
    // Then the tool handles it gracefully

    let temp_dir = TempDir::new().unwrap();
    fs::create_dir_all(temp_dir.path().join("locales")).unwrap();

    fs::write(
        temp_dir.path().join("locales/comments.yml"),
        "# This is a comment\n# Another comment\n",
    )
    .unwrap();

    let mut cmd = Command::cargo_bin("cs").unwrap();
    cmd.arg("test")
        .current_dir(temp_dir.path())
        .assert()
        .success();
}

#[test]
fn test_yaml_with_null_values() {
    // Given a YAML file with null values
    // When I search for text
    // Then null values are handled correctly

    let temp_dir = TempDir::new().unwrap();
    fs::create_dir_all(temp_dir.path().join("locales")).unwrap();

    fs::write(
        temp_dir.path().join("locales/nulls.yml"),
        "en:\n  key1: null\n  key2: \"value\"",
    )
    .unwrap();

    let mut cmd = Command::cargo_bin("cs").unwrap();
    cmd.arg("value")
        .current_dir(temp_dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("key2"));
}

#[test]
fn test_very_long_translation_keys() {
    // Given extremely long translation key paths
    // When I search for them
    // Then the tool handles them without issues

    let temp_dir = TempDir::new().unwrap();
    fs::create_dir_all(temp_dir.path().join("locales")).unwrap();

    let long_key = "a:\n  b:\n    c:\n      d:\n        e:\n          f:\n            g:\n              h: \"deep value\"";
    fs::write(temp_dir.path().join("locales/deep.yml"), long_key).unwrap();

    let mut cmd = Command::cargo_bin("cs").unwrap();
    cmd.arg("deep value")
        .current_dir(temp_dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("a.b.c.d.e.f.g.h"));
}

#[test]
fn test_special_yaml_characters_in_values() {
    // Given YAML with special characters in values
    // When I search for them
    // Then they are found correctly

    let temp_dir = TempDir::new().unwrap();
    fs::create_dir_all(temp_dir.path().join("locales")).unwrap();

    fs::write(
        temp_dir.path().join("locales/special.yml"),
        "en:\n  key: \"value: with: colons\"\n  key2: \"value with 'quotes'\"",
    )
    .unwrap();

    let mut cmd = Command::cargo_bin("cs").unwrap();
    cmd.arg("value: with: colons")
        .current_dir(temp_dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("key"));
}

#[test]
fn test_unicode_in_translations() {
    // Given translations with unicode characters
    // When I search for them
    // Then they are handled correctly

    let temp_dir = TempDir::new().unwrap();
    fs::create_dir_all(temp_dir.path().join("locales")).unwrap();

    fs::write(
        temp_dir.path().join("locales/unicode.yml"),
        "en:\n  greeting: \"Hello 世界 🌍\"",
    )
    .unwrap();

    let mut cmd = Command::cargo_bin("cs").unwrap();
    cmd.arg("Hello 世界")
        .current_dir(temp_dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("greeting"));
}

#[test]
fn test_yaml_with_array_values() {
    // Given YAML with array values
    // When I search for text in arrays
    // Then the tool handles it appropriately

    let temp_dir = TempDir::new().unwrap();
    fs::create_dir_all(temp_dir.path().join("locales")).unwrap();

    fs::write(
        temp_dir.path().join("locales/arrays.yml"),
        "en:\n  items:\n    - \"first\"\n    - \"second\"\n    - \"third\"",
    )
    .unwrap();

    let mut cmd = Command::cargo_bin("cs").unwrap();
    cmd.arg("second")
        .current_dir(temp_dir.path())
        .assert()
        .success();
}

#[test]
fn test_file_permission_errors_handled() {
    // This test would require creating files with restricted permissions
    // which may not work reliably on all systems, so we skip it
    // but document the expectation: permission errors should show clear messages
}

#[test]
fn test_very_large_yaml_file() {
    // Given a very large YAML file
    // When I search for text in it
    // Then the tool completes without hanging or crashing

    let temp_dir = TempDir::new().unwrap();
    fs::create_dir_all(temp_dir.path().join("locales")).unwrap();

    // Create a large YAML file with many keys
    let mut content = String::from("en:\n");
    for i in 0..1000 {
        content.push_str(&format!("  key{}: \"value{}\"\n", i, i));
    }
    content.push_str("  target: \"find me\"\n");

    fs::write(temp_dir.path().join("locales/large.yml"), content).unwrap();

    let mut cmd = Command::cargo_bin("cs").unwrap();
    cmd.arg("find me")
        .current_dir(temp_dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("target"));
}

#[test]
fn test_mixed_yaml_and_json_files() {
    // Given a project with both YAML and JSON translation files
    // When I search for text
    // Then both file types are searched

    let temp_dir = TempDir::new().unwrap();
    fs::create_dir_all(temp_dir.path().join("locales")).unwrap();

    fs::write(
        temp_dir.path().join("locales/en.yml"),
        "en:\n  yaml_key: \"yaml value\"",
    )
    .unwrap();

    fs::write(
        temp_dir.path().join("locales/en.json"),
        r#"{"en": {"json_key": "json value"}}"#,
    )
    .unwrap();

    let mut cmd = Command::cargo_bin("cs").unwrap();
    cmd.arg("yaml value")
        .current_dir(temp_dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("yaml_key"));

    let mut cmd2 = Command::cargo_bin("cs").unwrap();
    cmd2.arg("json value")
        .current_dir(temp_dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("json_key"));
}

#[test]
fn test_nested_directory_structure() {
    // Given translation files in nested directories
    // When I search for text
    // Then files in subdirectories are found

    let temp_dir = TempDir::new().unwrap();
    let nested_path = temp_dir.path().join("config/locales/en");
    fs::create_dir_all(&nested_path).unwrap();

    fs::write(
        nested_path.join("models.yml"),
        "en:\n  models:\n    user: \"User\"",
    )
    .unwrap();

    let mut cmd = Command::cargo_bin("cs").unwrap();
    cmd.arg("User")
        .current_dir(temp_dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("models.user"));
}

#[test]
fn test_symlinks_handled_correctly() {
    // On Unix-like systems, verify symlinks are followed correctly
    // This is platform-specific behavior
    #[cfg(unix)]
    {
        use std::os::unix::fs as unix_fs;

        let temp_dir = TempDir::new().unwrap();
        let real_dir = temp_dir.path().join("real_locales");
        fs::create_dir_all(&real_dir).unwrap();

        fs::write(real_dir.join("en.yml"), "en:\n  key: \"value\"").unwrap();

        let link_path = temp_dir.path().join("locales");
        unix_fs::symlink(&real_dir, &link_path).unwrap();

        let mut cmd = Command::cargo_bin("cs").unwrap();
        cmd.arg("value")
            .current_dir(temp_dir.path())
            .assert()
            .success()
            .stdout(predicate::str::contains("key"));
    }
}
