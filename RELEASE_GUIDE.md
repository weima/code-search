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

This step creates a branch, bumps versions in `Cargo.toml`, `npm/package.json`, etc., and commits them.

```bash
./scripts/release.sh prepare v0.2.0
```

**Next Steps:**
1.  Push the branch (`git push ...`).
2.  Create a Pull Request and merge it to `main`.
3.  **Tag the release** on GitHub:
    ```bash
    git checkout main
    git pull
    git tag v0.2.0
    git push origin v0.2.0
    ```

## 2. Publish Release (Distribute)

Wait for the GitHub Action to finish building the release assets. Then run:

```bash
./scripts/release.sh publish v0.2.0
```

This will:
1.  (Commented out) Publish to Crates.io
2.  (Commented out) Publish to NPM
3.  Create a branch `homebrew-v0.2.0`, update the formula with the new SHA256, and commit.

**Next Steps:**
1.  Push the homebrew branch.
2.  Create a Pull Request to merge the formula update.

