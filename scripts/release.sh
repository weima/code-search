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

    echo "Pushing branch $BRANCH_NAME..."
    git push -u origin "$BRANCH_NAME"

    if command -v gh &> /dev/null; then
        echo "Creating Pull Request..."
        gh pr create --title "chore: bump versions to $VERSION" --body "Automated PR to bump versions for release $VERSION." --base main --head "$BRANCH_NAME"
    else
        echo "GitHub CLI (gh) not found. Please create PR manually."
    fi
    
    exit 0
fi

# --- Publish Stage ---
if [ "$COMMAND" == "publish" ]; then
    echo "=== Publishing Release $VERSION ==="
    
    # Store current branch to return to it later
    ORIGINAL_BRANCH=$(git branch --show-current)
    echo "Current branch: $ORIGINAL_BRANCH"
    
    # Check for uncommitted changes (but allow continuing if on a release branch)
    if [ -n "$(git status --porcelain)" ]; then
        echo "Warning: Git working directory has uncommitted changes."
        echo "Continuing anyway since this might be expected for release branches..."
    fi

    # 1. Check/Create Tag
    if git rev-parse "$VERSION" >/dev/null 2>&1; then
        echo "Tag $VERSION already exists."
    else
        echo "Creating tag $VERSION..."
        git tag "$VERSION"
        git push origin "$VERSION"
        echo "Pushed tag $VERSION. GitHub Action should be running..."
    fi

    # 2. Wait for GitHub Release Asset (Homebrew needs this)
    URL="https://github.com/weima/code-search/releases/download/${VERSION}/cs-darwin-amd64"
    TEMP_FILE=$(mktemp)
    
    echo "Waiting for GitHub Release asset to be available..."
    echo "Target: $URL"
    
    MAX_RETRIES=30 # 5 minutes (30 * 10s)
    COUNT=0
    
    while [ $COUNT -lt $MAX_RETRIES ]; do
        HTTP_CODE=$(curl -L -o "$TEMP_FILE" -w "%{http_code}" "$URL")
        if [ "$HTTP_CODE" == "200" ]; then
            echo "Asset downloaded successfully!"
            break
        fi
        
        echo "Asset not ready yet (HTTP $HTTP_CODE). Waiting 10s... ($((COUNT+1))/$MAX_RETRIES)"
        sleep 10
        COUNT=$((COUNT+1))
    done

    if [ ! -s "$TEMP_FILE" ] || [ "$HTTP_CODE" != "200" ]; then
        echo "Error: Timed out waiting for release asset."
        rm -f "$TEMP_FILE"
        exit 1
    fi

    # 3. Cargo
    echo "Publishing to Crates.io..."
    cargo publish

    # 4. NPM
    echo "Publishing to NPM..."
    cd npm && npm publish && cd ..

    # 5. Homebrew Update
    echo "Updating Homebrew Formula..."
    BRANCH_NAME="homebrew-$VERSION"
    
    # Calculate SHA before switching branches
    SHA=$(shasum -a 256 "$TEMP_FILE" | awk '{print $1}')
    rm "$TEMP_FILE"
    echo "SHA256: $SHA"
    
    # Check if branch exists, if so delete it and recreate
    if git show-ref --verify --quiet "refs/heads/$BRANCH_NAME"; then
        echo "Branch $BRANCH_NAME already exists, deleting and recreating..."
        git branch -D "$BRANCH_NAME"
    fi
    
    git checkout -b "$BRANCH_NAME"

    sed -i '' "s|url \".*\"|url \"$URL\"|" Formula/cs.rb
    sed -i '' "s|sha256 \".*\"|sha256 \"$SHA\"|" Formula/cs.rb
    sed -i '' "s|version \".*\"|version \"$CLEAN_VERSION\"|" Formula/cs.rb

    git add Formula/cs.rb
    git commit -m "chore: update homebrew formula to $VERSION"

    echo "Pushing branch $BRANCH_NAME..."
    git push -u origin "$BRANCH_NAME"

    # 6. Create PR
    if command -v gh &> /dev/null; then
        echo "Creating Pull Request..."
        gh pr create --title "chore: update homebrew formula to $VERSION" --body "Automated PR to update Homebrew formula SHA256 for release $VERSION." --base main --head "$BRANCH_NAME"
    else
        echo "GitHub CLI (gh) not found. Please create PR manually."
    fi

    # Return to original branch
    echo "Returning to original branch: $ORIGINAL_BRANCH"
    git checkout "$ORIGINAL_BRANCH"

    echo "Done! Release $VERSION published."
    echo ""
    echo "Summary:"
    echo "  - Tag: $VERSION"
    echo "  - Homebrew branch: $BRANCH_NAME"
    echo "  - Formula updated with SHA: $SHA"
    echo ""
    echo "Next steps:"
    echo "  1. Merge the Homebrew PR"
    echo "  2. Users can update with: brew upgrade cs"
    exit 0
fi

echo "Unknown command: $COMMAND"
exit 1
