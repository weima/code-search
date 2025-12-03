#!/bin/bash
# Install git hooks for the project

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
HOOKS_DIR="$PROJECT_ROOT/.git/hooks"

echo "Installing git hooks..."

# Create pre-commit hook
cat > "$HOOKS_DIR/pre-commit" << 'EOF'
#!/bin/sh
# Pre-commit hook to ensure code quality

set -e  # Exit on first error

echo "🔍 Running pre-commit checks..."
echo ""

# 1. Auto-format code
echo "📝 Running cargo fmt..."

# Get list of staged .rs files before formatting
STAGED_RS_FILES=$(git diff --name-only --cached --diff-filter=ACM | grep '\.rs$' || true)

# Run cargo fmt
cargo fmt
if [ $? -ne 0 ]; then
    echo "❌ cargo fmt failed"
    exit 1
fi

# Check if any staged files were modified by cargo fmt
if [ -n "$STAGED_RS_FILES" ]; then
    # Check if formatting changed any staged files
    MODIFIED_FILES=$(git diff --name-only | grep '\.rs$' || true)

    if [ -n "$MODIFIED_FILES" ]; then
        echo "ℹ️  cargo fmt modified some files. Re-staging them..."
        # Re-stage only the files that were already staged AND modified by fmt
        for file in $STAGED_RS_FILES; do
            if echo "$MODIFIED_FILES" | grep -q "^$file$"; then
                git add "$file"
                echo "   ✓ Staged: $file"
            fi
        done
    fi
fi

# 2. Run clippy
echo ""
echo "🔧 Running cargo clippy..."
cargo clippy --all-targets --all-features -- -D warnings
if [ $? -ne 0 ]; then
    echo "❌ cargo clippy found issues"
    exit 1
fi

# 3. Run tests
echo ""
echo "🧪 Running cargo test..."
cargo test --lib
if [ $? -ne 0 ]; then
    echo "❌ Tests failed"
    exit 1
fi

echo ""
echo "✅ All pre-commit checks passed!"
EOF

# Make the hook executable
chmod +x "$HOOKS_DIR/pre-commit"

echo "✅ Git hooks installed successfully!"
echo ""
echo "The pre-commit hook will now:"
echo "  1. Auto-format your code with cargo fmt"
echo "  2. Check for clippy warnings"
echo "  3. Run library tests"
echo ""
echo "To skip hooks (not recommended), use: git commit --no-verify"
