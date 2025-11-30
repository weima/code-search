#!/bin/bash
set -e

# Usage: ./scripts/release.sh [prepare|publish] [version]
# Example: ./scripts/release.sh prepare v0.2.0
# Example: ./scripts/release.sh publish v0.2.0

COMMAND=$1
VERSION=$2

if [ -z "$COMMAND" ] || [ -z "$VERSION" ]; then
    echo "Usage: $0 [prepare|publish] <version-tag>"
    exit 1
fi

CLEAN_VERSION=${VERSION#v}

# --- Helper Functions ---

check_clean_git() {
    if [ -n "$(git status --porcelain)" ]; then
        echo "Error: Git working directory is not clean."
        exit 1
    fi
}

# --- Prepare Stage ---
if [ "$COMMAND" == "prepare" ]; then
    echo "=== Preparing Release $VERSION ==="
    check_clean_git

    BRANCH_NAME="build-release-$VERSION"
    
    echo "Creating branch $BRANCH_NAME..."
    git checkout -b "$BRANCH_NAME"

    echo "Bumping Cargo.toml to $CLEAN_VERSION..."
    # naive sed replacement for package version
    sed -i '' "s/^version = \".*\"/version = \"$CLEAN_VERSION\"/" Cargo.toml

    echo "Bumping npm/package.json to $CLEAN_VERSION..."
    sed -i '' "s/\"version\": \".*\"/\"version\": \"$CLEAN_VERSION\"/" npm/package.json

    echo "Bumping npm/install.js target to $CLEAN_VERSION..."
    sed -i '' "s/const VERSION = '.*';/const VERSION = '$CLEAN_VERSION';/" npm/install.js

    echo "Updating CHANGELOG.md..."
    # Prepend new version header (simplified)
    DATE=$(date +%Y-%m-%d)
    sed -i '' "129i\\
## [$CLEAN_VERSION] - $DATE\\
" CHANGELOG.md

    echo "Committing changes..."
    git add Cargo.toml npm/package.json npm/install.js CHANGELOG.md
    git commit -m "chore: bump versions to $VERSION"

    echo "Done! Now push this branch and create a PR:"
    echo "  git push -u origin $BRANCH_NAME"
    exit 0
fi

# --- Publish Stage ---
if [ "$COMMAND" == "publish" ]; then
    echo "=== Publishing Release $VERSION ==="
    check_clean_git

    # 1. Cargo
    echo "Publishing to Crates.io..."
    # cargo publish --dry-run
    # cargo publish

    # 2. NPM
    echo "Publishing to NPM..."
    # cd npm && npm publish

    # 3. Homebrew
    echo "Updating Homebrew Formula..."
    BRANCH_NAME="homebrew-$VERSION"
    git checkout -b "$BRANCH_NAME"

    URL="https://github.com/weima/code-search/releases/download/${VERSION}/cs-darwin-amd64"
    TEMP_FILE="cs-darwin-amd64-temp"

    echo "Downloading $URL..."
    curl -L -o "$TEMP_FILE" "$URL"
    
    if [ ! -f "$TEMP_FILE" ]; then
        echo "Error: Failed to download file. Did the GitHub Release finish building?"
        exit 1
    fi

    SHA=$(shasum -a 256 "$TEMP_FILE" | awk '{print $1}')
    rm "$TEMP_FILE"
    echo "SHA256: $SHA"

    sed -i '' "s|url \".*\"|url \"$URL\"|" Formula/cs.rb
    sed -i '' "s|sha256 \".*\"|sha256 \"$SHA\"|" Formula/cs.rb
    sed -i '' "s|version \".*\"|version \"$CLEAN_VERSION\"|" Formula/cs.rb

    git add Formula/cs.rb
    git commit -m "chore: update homebrew formula to $VERSION"

    echo "Done! Now push this branch and create a PR:"
    echo "  git push -u origin $BRANCH_NAME"
    exit 0
fi

echo "Unknown command: $COMMAND"
exit 1
