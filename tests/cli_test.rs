use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn test_help_flag() {
    let mut cmd = Command::cargo_bin("cs").unwrap();
    cmd.arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("code search tool"))
        .stdout(predicate::str::contains("USAGE:"));
}

#[test]
fn test_version_flag() {
    let mut cmd = Command::cargo_bin("cs").unwrap();
    cmd.arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("0.1.0"));
}

#[test]
fn test_empty_search_text_fails() {
    let mut cmd = Command::cargo_bin("cs").unwrap();
    cmd.arg("")
        .assert()
        .failure()
        .stderr(predicate::str::contains("search text cannot be empty"));
}

#[test]
fn test_requires_search_text() {
    let mut cmd = Command::cargo_bin("cs").unwrap();
    cmd.assert().failure().stderr(predicate::str::contains(
        "required arguments were not provided",
    ));
}

#[test]
fn test_depth_validation_too_low() {
    let mut cmd = Command::cargo_bin("cs").unwrap();
    cmd.args(["test", "--trace", "--depth", "0"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("depth must be between 1 and 10"));
}

#[test]
fn test_depth_validation_too_high() {
    let mut cmd = Command::cargo_bin("cs").unwrap();
    cmd.args(["test", "--trace", "--depth", "15"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("depth must be between 1 and 10"));
}

#[test]
fn test_trace_and_traceback_conflict() {
    let mut cmd = Command::cargo_bin("cs").unwrap();
    cmd.args(["test", "--trace", "--traceback"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("cannot be used with"));
}

#[test]
fn test_depth_flag_accepted() {
    let mut cmd = Command::cargo_bin("cs").unwrap();
    cmd.args(["test", "--trace", "--depth", "5"])
        .assert()
        .success();
}

#[test]
fn test_trace_mode_forward() {
    let mut cmd = Command::cargo_bin("cs").unwrap();
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
    let mut cmd = Command::cargo_bin("cs").unwrap();
    cmd.args(["processPayment", "--traceback"])
        .current_dir("tests/fixtures/code-examples")
        .assert()
        .success()
        .stdout(predicate::str::contains("processPayment"));
}

#[test]
fn test_case_sensitive_flag_accepted() {
    let mut cmd = Command::cargo_bin("cs").unwrap();
    cmd.args(["Test", "--case-sensitive"]).assert().success();
}
