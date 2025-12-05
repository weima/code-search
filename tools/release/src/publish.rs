use anyhow::{bail, Context, Result};
use git2::{Cred, PushOptions, RemoteCallbacks, Repository, Signature};
use std::fs;
use std::path::Path;
use std::process::Command;
use std::thread;
use std::time::Duration;
use tempfile::NamedTempFile;

pub fn run(version: String) -> Result<()> {
    println!("=== Publishing Release {} ===", version);
    let clean_version = version.trim_start_matches('v');

    let repo = Repository::open(".")?;

    check_uncommitted_changes(&repo);
    create_and_push_tag(&repo, &version)?;
    wait_for_github_assets(&version)?;
    publish_to_crates_io()?;
    publish_to_npm()?;
    update_homebrew(&repo, &version, clean_version)?;

    println!("\nDone! Release {} published.", version);
    Ok(())
}

fn check_uncommitted_changes(repo: &Repository) {
    let mut status_opts = git2::StatusOptions::new();
    status_opts.include_untracked(true);

    if let Ok(statuses) = repo.statuses(Some(&mut status_opts)) {
        if !statuses.is_empty() {
            println!("Warning: Git working directory has uncommitted changes.");
            println!("Continuing anyway since this might be expected for release branches...");
        }
    }
}

fn create_and_push_tag(repo: &Repository, version: &str) -> Result<()> {
    // Check if tag exists
    if repo.revparse_single(version).is_ok() {
        println!("Tag {} already exists.", version);
        return Ok(());
    }

    println!("Creating tag {}...", version);
    let head = repo.head()?;
    let commit = head.peel_to_commit()?;
    let obj = commit.into_object();

    let sig = Signature::now("Code Search Bot", "bot@code-search.com")?;

    repo.tag(version, &obj, &sig, &format!("Release {}", version), false)
        .context("Failed to create tag")?;

    println!("Pushing tag {}...", version);
    push_ref(repo, &format!("refs/tags/{}", version))?;

    Ok(())
}

fn push_ref(repo: &Repository, ref_spec: &str) -> Result<()> {
    let mut remote = repo.find_remote("origin")?;

    let mut callbacks = RemoteCallbacks::new();
    callbacks.credentials(|url, username_from_url, allowed_types| {
        if allowed_types.contains(git2::CredentialType::USER_PASS_PLAINTEXT) {
            let config = git2::Config::open_default()?;
            return Cred::credential_helper(&config, url, username_from_url);
        }
        if allowed_types.contains(git2::CredentialType::SSH_KEY) {
            return Cred::ssh_key_from_agent(username_from_url.unwrap_or("git"));
        }
        Err(git2::Error::from_str("no valid credential type"))
    });

    let mut push_opts = PushOptions::new();
    push_opts.remote_callbacks(callbacks);

    remote.push(&[ref_spec], Some(&mut push_opts))?;
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
        .current_dir("npm")
        .status()
        .context("Failed to run npm publish")?;

    if status.success() {
        println!("✓ Published to NPM");
    } else {
        println!("⚠ NPM publish failed (might already be published)");
    }
    Ok(())
}

fn update_homebrew(repo: &Repository, version: &str, clean_version: &str) -> Result<()> {
    println!("Updating Homebrew Formula...");
    let branch_name = format!("homebrew-{}", version);
    let url = format!(
        "https://github.com/weima/code-search/releases/download/{}/cs-darwin-amd64",
        version
    );

    // Download asset to calculate SHA
    let temp_file = NamedTempFile::new()?;
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
    if let Ok(mut branch) = repo.find_branch(&branch_name, git2::BranchType::Local) {
        branch.delete()?;
    }

    // Create branch
    let head = repo.head()?;
    let commit = head.peel_to_commit()?;
    repo.branch(&branch_name, &commit, false)?;

    // Checkout branch
    let obj = repo.revparse_single(&format!("refs/heads/{}", branch_name))?;
    repo.checkout_tree(&obj, None)?;
    repo.set_head(&format!("refs/heads/{}", branch_name))?;

    // Update Formula
    let formula_path = "Formula/cs.rb";
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
    let mut index = repo.index()?;
    index.add_path(Path::new("Formula/cs.rb"))?;
    index.write()?;

    let oid = index.write_tree()?;
    let tree = repo.find_tree(oid)?;
    let parent = repo.head()?.peel_to_commit()?;

    let sig = Signature::now("Code Search Bot", "bot@code-search.com")?;

    repo.commit(
        Some("HEAD"),
        &sig,
        &sig,
        &format!("chore: update homebrew formula to {}", version),
        &tree,
        &[&parent],
    )?;

    println!("Pushing branch {}...", branch_name);
    push_ref(repo, &format!("refs/heads/{}", branch_name))?;

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
            .status()?;
    } else {
        println!("GitHub CLI (gh) not found. Please create PR manually.");
    }

    Ok(())
}
