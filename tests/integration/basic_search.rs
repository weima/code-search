/// Integration tests for US-1: Basic Text-to-Code Trace
///
/// These tests verify that the tool can trace UI text directly to implementation code
/// by searching translation files and finding code references.
#[allow(deprecated)]
use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

#[test]
fn test_basic_search_shows_complete_chain() {
    // Given a project with i18n translation files
    // When I run `cs "add new"`
    // Then I see a tree showing:
    //   - The search text
    //   - Translation file location and line number
    //   - Full translation key path
    //   - All code files that use that key
    //   - Line numbers for each usage

    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin("cs"));
    cmd.arg("add new")
        .current_dir("tests/fixtures/rails-app")
        .assert()
        .success()
        .stdout(predicate::str::contains("'add new'"))
        .stdout(predicate::str::contains("search query"))
        .stdout(predicate::str::contains("invoice.labels.add_new"))
        .stdout(predicate::str::contains("en.yml"))
        .stdout(predicate::str::contains("Key:"))
        .stdout(predicate::str::contains("Code:"))
        .stdout(predicate::str::contains("invoices.ts"))
        .stdout(predicate::str::contains(":14)")) // Line number verification
        .stdout(predicate::str::contains("├─>").or(predicate::str::contains("└─>")));
}

#[test]
fn test_search_with_case_insensitive_default() {
    // Given search text with different casing
    // When I search without --case-sensitive flag
    // Then I find matches regardless of case

    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin("cs"));
    cmd.arg("Add New") // Mixed case
        .current_dir("tests/fixtures/rails-app")
        .assert()
        .success()
        .stdout(predicate::str::contains("add new"))
        .stdout(predicate::str::contains("invoice.labels.add_new"));
}

#[test]
fn test_search_with_case_sensitive_flag() {
    // Given search text
    // When I search with --case-sensitive flag
    // Then I only find exact case matches

    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin("cs"));
    cmd.args(["add new", "--case-sensitive"])
        .current_dir("tests/fixtures/rails-app")
        .assert()
        .success()
        .stdout(predicate::str::contains("add new"));
}

#[test]
fn test_search_finds_yaml_translation_files() {
    // Given a project with YAML translation files
    // When I search for translation text
    // Then the tool identifies YAML files correctly

    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin("cs"));
    cmd.arg("add new")
        .current_dir("tests/fixtures/rails-app")
        .assert()
        .success()
        .stdout(predicate::str::contains(".yml"));
}

#[test]
fn test_search_shows_full_key_path() {
    // Given nested translation keys
    // When I search for a translation
    // Then the full dot-notation key path is shown

    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin("cs"));
    cmd.arg("add new")
        .current_dir("tests/fixtures/rails-app")
        .assert()
        .success()
        .stdout(predicate::str::contains("invoice.labels.add_new"))
        .stdout(predicate::str::contains("en.invoice.labels.add_new"));
}

#[test]
fn test_search_finds_code_references() {
    // Given code files that use translation keys
    // When I search for translation text
    // Then all code references are found with line numbers

    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin("cs"));
    cmd.arg("add new")
        .current_dir("tests/fixtures/rails-app")
        .assert()
        .success()
        .stdout(predicate::str::contains("invoice_list.ts:"))
        .stdout(predicate::str::contains("invoices.ts:"));
}

#[test]
fn test_search_performance_reasonable() {
    // Given a typical project
    // When I run a search
    // Then results are displayed within reasonable time (< 5 seconds for tests)

    let start = std::time::Instant::now();

    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin("cs"));
    cmd.arg("add new")
        .current_dir("tests/fixtures/rails-app")
        .assert()
        .success();

    let duration = start.elapsed();
    assert!(
        duration.as_secs() < 5,
        "Search took too long: {:?}",
        duration
    );
}

#[test]
fn test_no_matches_returns_success() {
    // Given search text that doesn't exist
    // When I run the search
    // Then the tool returns success (not an error) with a clear message

    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin("cs"));
    cmd.arg("nonexistent xyz abc 123")
        .current_dir("tests/fixtures/rails-app")
        .assert()
        .success() // Should exit 0, not 1
        .stdout(predicate::str::contains("No matches found"));
}

#[test]
fn test_search_with_special_characters() {
    // Given search text with special characters
    // When I search for it
    // Then the tool handles it correctly

    let temp_dir = TempDir::new().unwrap();
    let yaml_path = temp_dir.path().join("config/locales");
    fs::create_dir_all(&yaml_path).unwrap();

    let yaml_file = yaml_path.join("en.yml");
    fs::write(&yaml_file, "en:\n  special: \"hello-world_test.123\"").unwrap();

    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin("cs"));
    cmd.arg("hello-world_test.123")
        .current_dir(temp_dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("special"));
}

#[test]
fn test_search_in_react_project() {
    // Given a React project with i18n
    // When I search for translation text
    // Then the tool finds react-i18next patterns

    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin("cs"));
    cmd.arg("home")
        .current_dir("tests/fixtures/react-app")
        .assert()
        .success();
}

#[test]
fn test_search_in_vue_project() {
    // Given a Vue project with i18n
    // When I search for translation text
    // Then the tool finds vue-i18n patterns

    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin("cs"));
    cmd.arg("welcome")
        .current_dir("tests/fixtures/vue-app")
        .assert()
        .success();
}
