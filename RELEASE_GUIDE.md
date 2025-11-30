# Release Guide

This document outlines the step-by-step process for releasing a new version of `code-search`.

## Prerequisites
- [ ] **Clean Git State**: Ensure your working directory is clean.
- [ ] **Credentials**: You must be logged in to:
    - `npm` (`npm login`)
    - `cargo` (`cargo login`)
    - GitHub CLI (`gh auth login`) (optional, for checking status)

## 1. Prepare the Release

1.  **Update Changelog**: Add entry for the new version in `CHANGELOG.md`.
2.  **Bump Rust Version**: Update `version` in `Cargo.toml`.
    ```toml
    [package]
    version = "0.2.0"
    ```
3.  **Commit & Push**:
    ```bash
    git add CHANGELOG.md Cargo.toml
    git commit -m "chore: bump version to 0.2.0"
    git push origin main
    ```

## 2. GitHub Release (Automated)

The GitHub Action `release.yml` handles the heavy lifting (building binaries, creating release).

1.  **Tag the Release**:
    ```bash
    git tag v0.2.0
    git push origin v0.2.0
    ```
2.  **Wait**: Go to the [Actions tab](https://github.com/weima/code-search/actions) and wait for the "Release" workflow to complete.
3.  **Verify**: Check the [Releases page](https://github.com/weima/code-search/releases) to ensure `v0.2.0` exists and has assets (`cs-darwin-amd64`, etc.).

## 3. Crates.io Release

Once the GitHub release is live:

1.  **Publish**:
    ```bash
    cargo publish
    ```

## 4. NPM Release

The NPM package is a wrapper. It needs to know which version of the binary to download.

1.  **Update Wrapper Version**: Edit `npm/package.json`.
    *   *Note*: This usually matches the Rust version, but can be higher if we patch the wrapper only.
    ```json
    "version": "0.2.0"
    ```
2.  **Update Binary Target**: Edit `npm/install.js`.
    ```javascript
    const VERSION = '0.2.0'; // Must match the GitHub Release tag (without 'v')
    ```
3.  **Publish**:
    ```bash
    cd npm
    npm publish
    ```

## 5. Homebrew Release

Homebrew formulas point to specific release artifacts and their SHA256 hashes.

1.  **Get SHA256**: Download the macOS binary from the new GitHub Release and hash it.
    ```bash
    # Example
    curl -L -O https://github.com/weima/code-search/releases/download/v0.2.0/cs-darwin-amd64
    shasum -a 256 cs-darwin-amd64
    ```
2.  **Update Formula**: Edit `Formula/cs.rb`.
    *   Update `url` to the new version.
    *   Update `sha256` with the hash from step 1.
    *   Update `version`.
3.  **Commit & Push**:
    ```bash
    git add Formula/cs.rb
    git commit -m "chore: update homebrew formula to v0.2.0"
    git push origin main
    ```

## Summary Checklist

- [ ] Bump `Cargo.toml` & `CHANGELOG.md`
- [ ] Push git tag `vX.Y.Z` -> **Triggers GitHub Release**
- [ ] `cargo publish` -> **Crates.io**
- [ ] Update `npm/package.json` & `npm/install.js` -> `npm publish` -> **NPM**
- [ ] Update `Formula/cs.rb` with new SHA -> Push to main -> **Homebrew**
