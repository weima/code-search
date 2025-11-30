#!/bin/bash

# Usage: ./scripts/release_homebrew.sh v0.1.0

set -e

VERSION=$1

if [ -z "$VERSION" ]; then
  echo "Usage: $0 <version-tag>"
  echo "Example: $0 v0.1.0"
  exit 1
fi

# Ensure we are in the project root
if [ ! -f "Formula/cs.rb" ]; then
    echo "Error: Could not find Formula/cs.rb. Please run this script from the project root."
    exit 1
fi

TAG=$VERSION
# Remove 'v' prefix for the version field (e.g., v0.1.0 -> 0.1.0)
CLEAN_VERSION=${TAG#v}

# URL for the macOS binary
URL="https://github.com/weima/code-search/releases/download/${TAG}/cs-darwin-amd64"
TEMP_FILE="cs-darwin-amd64-temp"

echo "Downloading $URL..."
curl -L -o "$TEMP_FILE" "$URL"

if [ ! -f "$TEMP_FILE" ]; then
    echo "Error: Failed to download file"
    exit 1
fi

# Calculate SHA256
SHA=$(shasum -a 256 "$TEMP_FILE" | awk '{print $1}')
echo "Calculated SHA256: $SHA"

# Clean up
rm "$TEMP_FILE"

# Update Formula/cs.rb
echo "Updating Formula/cs.rb..."

# Update URL
sed -i '' "s|url \".*\"|url \"$URL\"|" Formula/cs.rb

# Update SHA256
sed -i '' "s|sha256 \".*\"|sha256 \"$SHA\"|" Formula/cs.rb

# Update Version
sed -i '' "s|version \".*\"|version \"$CLEAN_VERSION\"|" Formula/cs.rb

echo "Success! Formula/cs.rb updated for version $CLEAN_VERSION"
echo "Don't forget to commit and push:"
echo "  git add Formula/cs.rb"
echo "  git commit -m \"chore: update homebrew formula to $TAG\""
echo "  git push origin main"
