use anyhow::{bail, Context, Result};
use std::fs;
use std::process::Command;
use std::thread;
use std::time::Duration;
use tempfile::NamedTempFile;

pub fn run(version: String) -> Result<()> {
    println!("=== Publishing Release {} ===", version);
    let clean_version = version.trim_start_matches('v');

    check_uncommitted_changes();
    create_and_push_tag(&version)?;
    wait_for_github_assets(&version)?;
    publish_to_crates_io()?;
    publish_to_npm()?;
    update_homebrew(&version, clean_version)?;

    println!("\nDone! Release {} published.", version);
    Ok(())
}

fn check_uncommitted_changes() {
    let output = Command::new("git")
        .args(["status", "--porcelain"])
        .output()
        .expect("Failed to run git status");

    if !output.stdout.is_empty() {
        println!("Warning: Git working directory has uncommitted changes.");
        println!("Continuing anyway since this might be expected for release branches...");
    }
}

fn create_and_push_tag(version: &str) -> Result<()> {
    // Check if tag exists
    let output = Command::new("git").args(["rev-parse", version]).output()?;

    if output.status.success() {
        println!("Tag {} already exists.", version);
        return Ok(());
    }

    println!("Creating tag {}...", version);
    Command::new("git")
        .args(["tag", version])
        .status()
        .context("Failed to create tag")?;

    println!("Pushing tag {}...", version);
    Command::new("git")
        .args(["push", "origin", version])
        .status()
        .context("Failed to push tag")?;

    Ok(())
}

fn wait_for_github_assets(version: &str) -> Result<()> {
    let url = format!(
        "https://github.com/weima/code-search/releases/download/{}/cs-darwin-amd64",
        version
    );
    println!("Waiting for GitHub Release asset to be available...");
    println!("Target: {}", url);

    let max_retries = 30; // 5 minutes
    let mut count = 0;

    while count < max_retries {
        let status = Command::new("curl")
            .args(["-L", "-o", "/dev/null", "-w", "%{http_code}", &url])
            .output()
            .context("Failed to check asset URL")?;

        let status_code = String::from_utf8_lossy(&status.stdout);
        if status_code.trim() == "200" {
            println!("Asset downloaded successfully!");
            return Ok(());
        }

        println!(
            "Asset not ready yet (HTTP {}). Waiting 10s... ({}/{})",
            status_code.trim(),
            count + 1,
            max_retries
        );
        thread::sleep(Duration::from_secs(10));
        count += 1;
    }

    bail!("Timed out waiting for release asset.");
}

fn publish_to_crates_io() -> Result<()> {
    println!("Publishing to Crates.io...");
    // Run from root
    let status = Command::new("cargo")
        .arg("publish")
        .current_dir("../..")
        .status()
        .context("Failed to run cargo publish")?;

    if status.success() {
        println!("✓ Published to Crates.io");
    } else {
        println!("⚠ Cargo publish failed (might already be published)");
    }
    Ok(())
}

fn publish_to_npm() -> Result<()> {
    println!("Publishing to NPM...");
    let status = Command::new("npm")
        .arg("publish")
        .current_dir("../../npm")
        .status()
        .context("Failed to run npm publish")?;

    if status.success() {
        println!("✓ Published to NPM");
    } else {
        println!("⚠ NPM publish failed (might already be published)");
    }
    Ok(())
}

fn update_homebrew(version: &str, clean_version: &str) -> Result<()> {
    println!("Updating Homebrew Formula...");
    let branch_name = format!("homebrew-{}", version);
    let url = format!(
        "https://github.com/weima/code-search/releases/download/{}/cs-darwin-amd64",
        version
    );

    // Download asset to calculate SHA
    let mut temp_file = NamedTempFile::new()?;
    let status = Command::new("curl")
        .args(["-L", "-o", temp_file.path().to_str().unwrap(), &url])
        .status()
        .context("Failed to download asset for SHA calculation")?;

    if !status.success() {
        bail!("Failed to download asset");
    }

    let sha_output = Command::new("shasum")
        .args(["-a", "256", temp_file.path().to_str().unwrap()])
        .output()
        .context("Failed to calculate SHA256")?;

    let sha_line = String::from_utf8_lossy(&sha_output.stdout);
    let sha = sha_line
        .split_whitespace()
        .next()
        .context("Failed to parse SHA256 output")?;
    println!("SHA256: {}", sha);

    // Git operations
    // Check if branch exists
    let _ = Command::new("git")
        .args(["branch", "-D", &branch_name])
        .current_dir("../..")
        .output();

    Command::new("git")
        .args(["checkout", "-b", &branch_name])
        .current_dir("../..")
        .status()
        .context("Failed to create branch")?;

    // Update Formula
    let formula_path = "../../Formula/cs.rb";
    let content = fs::read_to_string(formula_path).context("Failed to read Formula")?;

    // Use regex for replacement
    let url_re = regex::Regex::new(r#"url ".*""#).unwrap();
    let sha_re = regex::Regex::new(r#"sha256 ".*""#).unwrap();
    let ver_re = regex::Regex::new(r#"version ".*""#).unwrap();

    let c1 = url_re.replace(&content, format!(r#"url "{}""#, url));
    let c2 = sha_re.replace(&c1, format!(r#"sha256 "{}""#, sha));
    let c3 = ver_re.replace(&c2, format!(r#"version "{}""#, clean_version));

    fs::write(formula_path, c3.to_string()).context("Failed to write Formula")?;

    // Commit and Push
    Command::new("git")
        .args(["add", "Formula/cs.rb"])
        .current_dir("../..")
        .status()?;

    Command::new("git")
        .args([
            "commit",
            "-m",
            &format!("chore: update homebrew formula to {}", version),
        ])
        .current_dir("../..")
        .status()?;

    println!("Pushing branch {}...", branch_name);
    Command::new("git")
        .args(["push", "-u", "origin", &branch_name])
        .current_dir("../..")
        .status()?;

    // Create PR
    if Command::new("gh").arg("--version").output().is_ok() {
        println!("Creating Pull Request...");
        Command::new("gh")
            .args([
                "pr",
                "create",
                "--title",
                &format!("chore: update homebrew formula to {}", version),
                "--body",
                &format!(
                    "Automated PR to update Homebrew formula SHA256 for release {}.",
                    version
                ),
                "--base",
                "main",
                "--head",
                &branch_name,
                "--assignee",
                "@me",
            ])
            .current_dir("../..")
            .status()?;
    } else {
        println!("GitHub CLI (gh) not found. Please create PR manually.");
    }

    // Return to original branch (simplified, assumes main or we can just stay)
    // For now, let's stay on the branch to let user verify

    Ok(())
}
