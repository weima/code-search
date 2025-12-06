#[allow(deprecated)]
use assert_cmd::Command;
use predicates::prelude::*;

fn cs_cmd() -> Command {
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin("cs"));
    cmd.env("NO_COLOR", "1");
    cmd
}

#[test]
fn test_help_flag() {
    let mut cmd = cs_cmd();
    cmd.arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("code search tool"))
        .stdout(predicate::str::contains("USAGE:"));
}

#[test]
fn test_version_flag() {
    let mut cmd = cs_cmd();
    cmd.arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains(env!("CARGO_PKG_VERSION")));
}

#[test]
fn test_empty_search_text_fails() {
    let mut cmd = cs_cmd();
    cmd.arg("")
        .assert()
        .failure()
        .stderr(predicate::str::contains("search text cannot be empty"));
}

#[test]
fn test_requires_search_text() {
    let mut cmd = cs_cmd();
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("search text cannot be empty"));
}

#[test]
fn test_depth_validation_too_low() {
    let mut cmd = cs_cmd();
    cmd.args(["test", "--trace", "--depth", "0"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("depth must be between 1 and 10"));
}

#[test]
fn test_depth_validation_too_high() {
    let mut cmd = cs_cmd();
    cmd.args(["test", "--trace", "--depth", "15"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("depth must be between 1 and 10"));
}

#[test]
fn test_trace_and_traceback_conflict() {
    let mut cmd = cs_cmd();
    cmd.args(["test", "--trace", "--traceback"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("cannot be used with"));
}

#[test]
fn test_depth_flag_accepted() {
    let mut cmd = cs_cmd();
    cmd.args(["checkout", "--trace", "--depth", "5"])
        .current_dir("tests/fixtures/code-examples")
        .assert()
        .success();
}

#[test]
fn test_trace_mode_forward() {
    let mut cmd = cs_cmd();
    cmd.args(["checkout", "--trace"])
        .current_dir("tests/fixtures/code-examples")
        .assert()
        .success()
        .stdout(predicate::str::contains("checkout"))
        .stdout(predicate::str::contains("calculateTotal"))
        .stdout(predicate::str::contains("processPayment"));
}

#[test]
fn test_trace_mode_backward() {
    let mut cmd = cs_cmd();
    cmd.args(["processPayment", "--traceback"])
        .current_dir("tests/fixtures/code-examples")
        .assert()
        .success()
        .stdout(predicate::str::contains("processPayment"));
}

#[test]
fn test_case_sensitive_flag_accepted() {
    let mut cmd = cs_cmd();
    cmd.args(["Test", "--case-sensitive"]).assert().success();
}

#[test]
fn test_trace_function_not_found() {
    let mut cmd = cs_cmd();
    cmd.args(["nonExistentFunction123", "--trace"])
        .current_dir("tests/fixtures/code-examples")
        .assert()
        .failure()
        .stderr(predicate::str::contains("nonExistentFunction123"))
        .stderr(predicate::str::contains("not found in codebase"))
        .stderr(predicate::str::contains("Possible reasons:"))
        .stderr(predicate::str::contains("Next steps:"));
}

#[test]
fn test_trace_function_not_found_shows_helpful_tips() {
    let mut cmd = cs_cmd();
    cmd.args(["invalidFunc", "--traceback"])
        .current_dir("tests/fixtures/code-examples")
        .assert()
        .failure()
        .stderr(predicate::str::contains("The function doesn't exist"))
        .stderr(predicate::str::contains("Verify function name"))
        .stderr(predicate::str::contains(
            "Check if you're in the right directory",
        ));
}

#[test]
fn test_default_mode_without_flags_does_i18n_search() {
    // This test verifies that default mode (no --trace flags) still performs i18n search
    let mut cmd = cs_cmd();
    cmd.arg("Add New")
        .current_dir("tests/fixtures/rails-app")
        .assert()
        .success()
        .stdout(predicate::str::contains("invoice.labels.add_new"));
}
