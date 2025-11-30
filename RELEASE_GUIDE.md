# Release Guide

This document outlines the step-by-step process for releasing a new version of `code-search`.

## Prerequisites
- [ ] **Clean Git State**: Ensure your working directory is clean.
- [ ] **Credentials**: You must be logged in to:
    - `npm` (`npm login`)
    - `cargo` (`cargo login`)
    - GitHub CLI (`gh auth login`) (optional, for checking status)

# Release Guide

This document outlines the automated process for releasing a new version of `code-search`.

## Prerequisites
- Clean git directory
- Logged in to `npm` and `cargo`

## 1. Prepare Release (Version Bump)

```bash
./scripts/release.sh prepare v0.2.0
```
*   Creates branch `build-release-v0.2.0`
*   Bumps versions
*   Commits changes
*   **Auto-creates Pull Request** (if `gh` CLI is installed)

**Action**: Merge the PR to `main`.

## 2. Publish Release (Distribute)

Once the prepare PR is merged:

```bash
./scripts/release.sh publish v0.2.0
```

*   **Checks/Creates Tag**: If `v0.2.0` doesn't exist, it tags and pushes it.
*   **Waits for CI**: Polls GitHub until release assets are built (can take ~2-5 mins).
*   **Updates Homebrew**: Downloads asset, calculates SHA, updates formula.
*   **Auto-creates Pull Request** for the formula update.

**Action**: Merge the Homebrew PR.

