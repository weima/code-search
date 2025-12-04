/// Integration tests for US-3: Multiple Match Handling
///
/// These tests verify that the tool can find and display all locations where
/// a translation key is used across multiple files.
#[allow(deprecated)]
use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

#[test]
fn test_multiple_code_references_all_shown() {
    // Given a translation key used in multiple files
    // When I search for the associated text
    // Then I see all usage locations in the tree

    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin("cs"));
    cmd.arg("add new")
        .current_dir("tests/fixtures/rails-app")
        .assert()
        .success()
        .stdout(predicate::str::contains("invoice_list.ts"))
        .stdout(predicate::str::contains("invoices.ts"))
        .stdout(predicate::str::contains("Code:").count(7)); // Multiple usages (4 translation + 3 direct)
}

#[test]
fn test_multiple_usages_show_line_numbers() {
    // Given a key used multiple times
    // When I search for it
    // Then each usage shows full file path and line number

    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin("cs"));
    cmd.arg("add new")
        .current_dir("tests/fixtures/rails-app")
        .assert()
        .success()
        .stdout(predicate::str::contains(":12)"))
        .stdout(predicate::str::contains(":14)"))
        .stdout(predicate::str::contains(":22)"));
}

#[test]
fn test_multiple_translation_files() {
    // Given translation text exists in multiple language files
    // When I search for it
    // Then all translation files are shown

    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin("cs"));
    cmd.arg("ajouter nouveau") // French translation for "add new"
        .current_dir("tests/fixtures/rails-app")
        .assert()
        .success()
        .stdout(predicate::str::contains("fr.yml"));
}

#[test]
fn test_partial_key_matching() {
    // Given code using partial keys (namespace caching pattern)
    // When I search for translation text
    // Then the tool finds both full and partial key usages

    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin("cs"));
    cmd.arg("add new")
        .current_dir("tests/fixtures/rails-app")
        .assert()
        .success()
        // Full key: invoice.labels.add_new
        .stdout(predicate::str::contains("invoice.labels.add_new"))
        // Partial key without first segment: labels.add_new
        .stdout(predicate::str::contains("labels.add_new"))
        // Partial key without last segment: invoice.labels
        .stdout(predicate::str::contains("invoice.labels"));
}

#[test]
fn test_multiple_patterns_detected() {
    // Given code using different i18n patterns
    // When I search for translation text
    // Then all patterns are detected (t(), I18n.t(), etc.)

    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin("cs"));
    cmd.arg("add new")
        .current_dir("tests/fixtures/rails-app")
        .assert()
        .success()
        .stdout(predicate::str::contains("I18n.t("));
}

#[test]
fn test_related_usages_grouped() {
    // Given multiple usages of same key
    // When I display results
    // Then related usages are shown in tree format

    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin("cs"));
    cmd.arg("add new")
        .current_dir("tests/fixtures/rails-app")
        .assert()
        .success()
        .stdout(predicate::str::contains("Code:"))
        .stdout(predicate::str::contains("├─>").or(predicate::str::contains("└─>")));
}

#[test]
fn test_multiple_keys_with_same_value() {
    // Given multiple translation keys with the same value
    // When I search for the value
    // Then all keys are shown

    let temp_dir = TempDir::new().unwrap();
    let yaml_path = temp_dir.path().join("config/locales");
    fs::create_dir_all(&yaml_path).unwrap();

    let yaml_file = yaml_path.join("en.yml");
    fs::write(
        &yaml_file,
        "en:\n  key1: \"submit\"\n  key2: \"submit\"\n  nested:\n    key3: \"submit\"",
    )
    .unwrap();

    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin("cs"));
    cmd.arg("submit")
        .current_dir(temp_dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("key1"))
        .stdout(predicate::str::contains("key2"))
        .stdout(predicate::str::contains("nested.key3"));
}

#[test]
fn test_deeply_nested_keys() {
    // Given deeply nested translation keys
    // When I search for a deeply nested value
    // Then the full key path is correctly shown

    let temp_dir = TempDir::new().unwrap();
    let yaml_path = temp_dir.path().join("config/locales");
    fs::create_dir_all(&yaml_path).unwrap();

    let yaml_file = yaml_path.join("en.yml");
    fs::write(&yaml_file,
        "en:\n  app:\n    views:\n      invoice:\n        form:\n          labels:\n            add_new: \"Add New\"").unwrap();

    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin("cs"));
    cmd.arg("Add New")
        .current_dir(temp_dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "app.views.invoice.form.labels.add_new",
        ));
}

#[test]
fn test_cross_file_references() {
    // Given a key used across multiple code files
    // When I search for it
    // Then all files are listed

    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin("cs"));
    cmd.arg("add new")
        .current_dir("tests/fixtures/rails-app")
        .assert()
        .success()
        // Should appear in both invoice_list.ts and invoices.ts
        .stdout(predicate::str::contains("invoice_list.ts"))
        .stdout(predicate::str::contains("invoices.ts"));
}

#[test]
fn test_no_duplicate_results() {
    // Given a key that might match multiple patterns
    // When I search for it
    // Then each unique location is shown only once (no duplicates)

    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin("cs"));
    let output = cmd
        .arg("add new")
        .current_dir("tests/fixtures/rails-app")
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Count occurrences of specific line references
    // We should not see the same file:line combination multiple times
    let lines: Vec<&str> = stdout.lines().collect();
    let mut seen = std::collections::HashSet::new();

    for line in lines {
        if line.contains(".ts:") {
            // Each file:line combo should appear only once
            if seen.contains(line) {
                panic!("Duplicate line found: {}", line);
            }
            seen.insert(line);
        }
    }
}

#[test]
fn test_multiple_frameworks_in_one_project() {
    // Given a project mixing multiple frameworks
    // When I search for translation text
    // Then all framework patterns are detected

    let temp_dir = TempDir::new().unwrap();

    // Create YAML translation
    let yaml_path = temp_dir.path().join("locales");
    fs::create_dir_all(&yaml_path).unwrap();
    fs::write(yaml_path.join("en.yml"), "en:\n  greeting: \"Hello\"").unwrap();

    // Create code files with different patterns
    fs::write(temp_dir.path().join("ruby.rb"), "I18n.t('greeting')").unwrap();
    fs::write(temp_dir.path().join("react.tsx"), "t('greeting')").unwrap();
    fs::write(temp_dir.path().join("vue.vue"), "$t('greeting')").unwrap();

    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin("cs"));
    cmd.arg("Hello")
        .current_dir(temp_dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("ruby.rb"))
        .stdout(predicate::str::contains("react.tsx"))
        .stdout(predicate::str::contains("vue.vue"));
}
