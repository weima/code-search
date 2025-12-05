use anyhow::{bail, Context, Result};
use chrono::Local;
use git2::{Cred, PushOptions, RemoteCallbacks, Repository, Signature};
use regex::Regex;
use serde_json::Value;
use std::fs;
use std::path::Path;
use std::process::Command;
use toml_edit::{value, Document};

pub fn run(version: String) -> Result<()> {
    let clean_version = version.trim_start_matches('v');
    println!("=== Preparing Release {} ===", version);

    // Open repository
    let repo = Repository::open(".")?;

    check_clean_git(&repo)?;

    let branch_name = format!("build-release-{}", version);
    create_branch(&repo, &branch_name)?;

    bump_cargo_version(clean_version)?;
    bump_npm_version(clean_version)?;
    bump_install_js_version(clean_version)?;
    update_changelog(clean_version)?;

    commit_and_push(&repo, &branch_name, &version)?;
    create_pr(&branch_name, &version)?;

    Ok(())
}

fn check_clean_git(repo: &Repository) -> Result<()> {
    let mut status_opts = git2::StatusOptions::new();
    status_opts.include_untracked(true);

    let statuses = repo.statuses(Some(&mut status_opts))?;

    if !statuses.is_empty() {
        bail!("Git working directory is not clean. Please commit or stash changes.");
    }
    Ok(())
}

fn create_branch(repo: &Repository, branch_name: &str) -> Result<()> {
    println!("Creating branch {}...", branch_name);

    let head = repo.head()?;
    let commit = head.peel_to_commit()?;

    repo.branch(branch_name, &commit, false)
        .context("Failed to create branch")?;

    // Checkout the new branch
    let obj = repo.revparse_single(&format!("refs/heads/{}", branch_name))?;
    repo.checkout_tree(&obj, None)?;
    repo.set_head(&format!("refs/heads/{}", branch_name))?;

    Ok(())
}

fn bump_cargo_version(version: &str) -> Result<()> {
    println!("Bumping Cargo.toml to {}...", version);
    let cargo_toml_path = "Cargo.toml";
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
    let package_json_path = "npm/package.json";
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
    let install_js_path = "npm/install.js";
    let content = fs::read_to_string(install_js_path).context("Failed to read install.js")?;

    let re = Regex::new(r"const VERSION = '.*';").unwrap();
    let new_content = re.replace(&content, format!("const VERSION = '{}';", version));

    fs::write(install_js_path, new_content.to_string()).context("Failed to write install.js")?;
    Ok(())
}

fn update_changelog(version: &str) -> Result<()> {
    println!("Updating CHANGELOG.md...");
    let changelog_path = "CHANGELOG.md";
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

fn commit_and_push(repo: &Repository, branch_name: &str, version: &str) -> Result<()> {
    println!("Committing changes...");

    let mut index = repo.index()?;
    index.add_path(Path::new("Cargo.toml"))?;
    index.add_path(Path::new("npm/package.json"))?;
    index.add_path(Path::new("npm/install.js"))?;
    index.add_path(Path::new("CHANGELOG.md"))?;
    index.write()?;

    let oid = index.write_tree()?;
    let tree = repo.find_tree(oid)?;
    let parent = repo.head()?.peel_to_commit()?;

    let sig = Signature::now("Code Search Bot", "bot@code-search.com")?;

    repo.commit(
        Some("HEAD"),
        &sig,
        &sig,
        &format!("chore: bump versions to {}", version),
        &tree,
        &[&parent],
    )?;

    println!("Pushing branch {}...", branch_name);
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

    remote.push(
        &[&format!("refs/heads/{}", branch_name)],
        Some(&mut push_opts),
    )?;

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
        .status()
        .context("Failed to create PR")?;
    Ok(())
}
