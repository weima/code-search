#[allow(deprecated)]
use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn test_cli_displays_formatted_tree() {
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin("cs"));
    cmd.arg("add new")
        .current_dir("tests/fixtures/rails-app")
        .assert()
        .success()
        .stdout(predicate::str::contains("'add new'"))
        .stdout(predicate::str::contains("search query"))
        .stdout(predicate::str::contains("invoice.labels.add_new"))
        .stdout(predicate::str::contains("Key:"))
        .stdout(predicate::str::contains("en.yml"))
        .stdout(predicate::str::contains("invoice_list.ts"))
        .stdout(predicate::str::contains("invoices.ts"));
}

#[test]
fn test_cli_no_matches() {
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin("cs"));
    cmd.arg("nonexistent xyz abc")
        .current_dir("tests/fixtures/rails-app")
        .assert()
        .success()
        .stdout(predicate::str::contains("No matches found"));
}

#[test]
fn test_cli_case_sensitive() {
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin("cs"));
    cmd.args(["add new", "--case-sensitive"])
        .current_dir("tests/fixtures/rails-app")
        .assert()
        .success();
}

#[test]
fn test_cli_shows_tree_structure() {
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin("cs"));
    cmd.arg("add new")
        .current_dir("tests/fixtures/rails-app")
        .assert()
        .success()
        .stdout(predicate::str::contains("├─>").or(predicate::str::contains("└─>")));
}

#[test]
fn test_cli_shows_locations() {
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin("cs"));
    cmd.arg("add new")
        .current_dir("tests/fixtures/rails-app")
        .assert()
        .success()
        .stdout(predicate::str::contains(":4)")) // Line number from YAML
        .stdout(predicate::str::contains(":14)")); // Line number from code
}

#[test]
fn test_cli_multiple_matches() {
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin("cs"));
    cmd.arg("invoice")
        .current_dir("tests/fixtures/rails-app")
        .assert()
        .success()
        .stdout(predicate::str::contains("invoice"));
}

#[test]
fn test_cli_empty_search_text() {
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin("cs"));
    cmd.arg("")
        .assert()
        .failure()
        .stderr(predicate::str::contains("empty"));
}

#[test]
fn test_cli_help() {
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin("cs"));
    cmd.arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Code Search"))
        .stdout(predicate::str::contains("SEARCH_TEXT"));
}

#[test]
fn test_cli_version() {
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin("cs"));
    cmd.arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("cs"));
}

#[test]
fn test_cli_multiple_translation_files() {
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin("cs"));
    cmd.arg("ajouter nouveau")
        .current_dir("tests/fixtures/rails-app")
        .assert()
        .success()
        .stdout(predicate::str::contains("ajouter nouveau"))
        .stdout(predicate::str::contains("fr.yml"))
        .stdout(predicate::str::contains("invoice.labels.add_new"));
}

#[test]
fn test_cli_multiple_code_references() {
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin("cs"));
    cmd.arg("add new")
        .current_dir("tests/fixtures/rails-app")
        .assert()
        .success()
        .stdout(predicate::str::contains("invoice_list.ts"))
        .stdout(predicate::str::contains("invoices.ts"))
        // Should show multiple code references (translation matches + direct matches)
        // 5 occurrences in invoice_list.ts (lines 12,22,29,34,43) + 2 in invoices.ts (12,14)
        .stdout(predicate::str::contains("invoice_list.ts").count(5))
        .stdout(predicate::str::contains("invoices.ts").count(2));
}

#[test]
fn test_cli_exclude_flag() {
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin("cs"));
    cmd.args(["add new", "--exclude", "invoice_list"])
        .current_dir("tests/fixtures/rails-app")
        .assert()
        .success()
        .stdout(predicate::str::contains("invoices.ts"))
        .stdout(predicate::str::contains("invoice_list.ts").not());
}

#[test]
fn test_cli_multiple_exclusions() {
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin("cs"));
    cmd.args(["add new", "--exclude", "invoice_list,invoices"])
        .current_dir("tests/fixtures/rails-app")
        .assert()
        .success()
        .stdout(predicate::str::contains("invoice_list.ts").not())
        .stdout(predicate::str::contains("invoices.ts").not());
}
