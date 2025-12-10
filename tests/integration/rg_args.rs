#[allow(deprecated)]
use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

#[test]
fn test_ignore_case_flag() {
    let temp_dir = TempDir::new().unwrap();
    fs::write(temp_dir.path().join("test.txt"), "Hello World").unwrap();

    // Default is case sensitive - should not find "Hello World" when searching for "hello"
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin("cs"));
    cmd.env("NO_COLOR", "1")
        .arg("hello")
        .current_dir(temp_dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("No matches found"));

    // -i should work (case insensitive)
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin("cs"));
    cmd.env("NO_COLOR", "1")
        .args(["hello", "-i"])
        .current_dir(temp_dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Hello World"));

    // -s should also fail to find "hello" (explicit case sensitive)
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin("cs"));
    cmd.env("NO_COLOR", "1")
        .args(["hello", "-s"])
        .current_dir(temp_dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("No matches found"));
}

#[test]
fn test_word_match_flag() {
    let temp_dir = TempDir::new().unwrap();
    fs::write(temp_dir.path().join("test.txt"), "hello world\nhelloworld").unwrap();

    // Without -w, finds both
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin("cs"));
    cmd.env("NO_COLOR", "1")
        .arg("hello")
        .current_dir(temp_dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("hello world"))
        .stdout(predicate::str::contains("helloworld"));

    // With -w, finds only "hello world"
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin("cs"));
    cmd.env("NO_COLOR", "1")
        .args(["hello", "-w"])
        .current_dir(temp_dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("hello world"))
        .stdout(predicate::str::contains("helloworld").not());
}

#[test]
fn test_glob_flag() {
    let temp_dir = TempDir::new().unwrap();
    fs::write(temp_dir.path().join("test.rs"), "target").unwrap();
    fs::write(temp_dir.path().join("test.txt"), "target").unwrap();

    // With -g "*.rs", finds only test.rs
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin("cs"));
    cmd.env("NO_COLOR", "1")
        .args(["target", "-g", "*.rs"])
        .current_dir(temp_dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("test.rs"))
        .stdout(predicate::str::contains("test.txt").not());
}

#[test]
fn test_regex_flag() {
    let temp_dir = TempDir::new().unwrap();
    fs::write(temp_dir.path().join("test.txt"), "abc 123 xyz").unwrap();

    // Without --regex, "c \d+" is literal and won't match
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin("cs"));
    cmd.env("NO_COLOR", "1")
        .arg("c \\d+")
        .current_dir(temp_dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("No matches found"));

    // With --regex, it should match
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin("cs"));
    cmd.env("NO_COLOR", "1")
        .args(["c \\d+", "--regex"])
        .current_dir(temp_dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("abc 123 xyz"));
}
