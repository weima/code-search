use anyhow::{bail, Context, Result};
use chrono::Local;
use regex::Regex;
use serde_json::Value;
use std::fs;
use std::process::Command;
use toml_edit::{value, Document};

pub fn run(version: String) -> Result<()> {
    let clean_version = version.trim_start_matches('v');
    println!("=== Preparing Release {} ===", version);

    check_clean_git()?;

    let branch_name = format!("build-release-{}", version);
    create_branch(&branch_name)?;

    bump_cargo_version(clean_version)?;
    bump_npm_version(clean_version)?;
    bump_install_js_version(clean_version)?;
    update_changelog(clean_version)?;

    commit_and_push(&branch_name, &version)?;
    create_pr(&branch_name, &version)?;

    Ok(())
}

fn check_clean_git() -> Result<()> {
    let output = Command::new("git")
        .args(["status", "--porcelain"])
        .output()
        .context("Failed to run git status")?;

    if !output.stdout.is_empty() {
        bail!("Git working directory is not clean. Please commit or stash changes.");
    }
    Ok(())
}

fn create_branch(branch_name: &str) -> Result<()> {
    println!("Creating branch {}...", branch_name);
    Command::new("git")
        .args(["checkout", "-b", branch_name])
        .status()
        .context("Failed to create branch")?;
    Ok(())
}

fn bump_cargo_version(version: &str) -> Result<()> {
    println!("Bumping Cargo.toml to {}...", version);
    let cargo_toml_path = "../../Cargo.toml";
    let cargo_toml = fs::read_to_string(cargo_toml_path).context("Failed to read Cargo.toml")?;
    let mut doc = cargo_toml
        .parse::<Document>()
        .context("Failed to parse Cargo.toml")?;

    doc["package"]["version"] = value(version);

    fs::write(cargo_toml_path, doc.to_string()).context("Failed to write Cargo.toml")?;
    Ok(())
}

fn bump_npm_version(version: &str) -> Result<()> {
    println!("Bumping npm/package.json to {}...", version);
    let package_json_path = "../../npm/package.json";
    let package_json =
        fs::read_to_string(package_json_path).context("Failed to read package.json")?;
    let mut json: Value =
        serde_json::from_str(&package_json).context("Failed to parse package.json")?;

    if let Some(obj) = json.as_object_mut() {
        obj.insert("version".to_string(), Value::String(version.to_string()));
    }

    let formatted = serde_json::to_string_pretty(&json)?;
    fs::write(package_json_path, formatted).context("Failed to write package.json")?;
    Ok(())
}

fn bump_install_js_version(version: &str) -> Result<()> {
    println!("Bumping npm/install.js to {}...", version);
    let install_js_path = "../../npm/install.js";
    let content = fs::read_to_string(install_js_path).context("Failed to read install.js")?;

    let re = Regex::new(r"const VERSION = '.*';").unwrap();
    let new_content = re.replace(&content, format!("const VERSION = '{}';", version));

    fs::write(install_js_path, new_content.to_string()).context("Failed to write install.js")?;
    Ok(())
}

fn update_changelog(version: &str) -> Result<()> {
    println!("Updating CHANGELOG.md...");
    let changelog_path = "../../CHANGELOG.md";
    let content = fs::read_to_string(changelog_path).context("Failed to read CHANGELOG.md")?;

    let date = Local::now().format("%Y-%m-%d").to_string();
    let new_header = format!("## [Unreleased]\n\n## [{}] - {}", version, date);

    let new_content = content.replace("## [Unreleased]", &new_header);

    if new_content == content {
        bail!("Could not find '## [Unreleased]' in CHANGELOG.md");
    }

    fs::write(changelog_path, new_content).context("Failed to write CHANGELOG.md")?;
    Ok(())
}

fn commit_and_push(branch_name: &str, version: &str) -> Result<()> {
    println!("Committing changes...");
    Command::new("git")
        .args([
            "add",
            "Cargo.toml",
            "npm/package.json",
            "npm/install.js",
            "CHANGELOG.md",
        ])
        .current_dir("../..") // Run from root
        .status()
        .context("Failed to git add")?;

    Command::new("git")
        .args([
            "commit",
            "-m",
            &format!("chore: bump versions to {}", version),
        ])
        .current_dir("../..")
        .status()
        .context("Failed to git commit")?;

    println!("Pushing branch {}...", branch_name);
    Command::new("git")
        .args(["push", "-u", "origin", branch_name])
        .current_dir("../..")
        .status()
        .context("Failed to git push")?;
    Ok(())
}

fn create_pr(branch_name: &str, version: &str) -> Result<()> {
    // Check if gh is installed
    if Command::new("gh").arg("--version").output().is_err() {
        println!("GitHub CLI (gh) not found. Please create PR manually.");
        return Ok(());
    }

    println!("Creating Pull Request...");
    Command::new("gh")
        .args([
            "pr",
            "create",
            "--title",
            &format!("chore: bump versions to {}", version),
            "--body",
            &format!("Automated PR to bump versions for release {}.", version),
            "--base",
            "main",
            "--head",
            branch_name,
            "--assignee",
            "@me",
        ])
        .current_dir("../..")
        .status()
        .context("Failed to create PR")?;
    Ok(())
}
