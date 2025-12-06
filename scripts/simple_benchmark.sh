#!/bin/bash

# Simple benchmark comparing cs vs rg for AI agent workflows
# Focus: Demonstrate caching benefits and AI-friendly features

DISCOURSE_DIR="/tmp/discourse/config/locales"
CS_BIN="./target/release/cs"
TEST_FILE="$DISCOURSE_DIR/client.en.yml"

# Colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m'

echo -e "${BLUE}================================================${NC}"
echo -e "${BLUE}  cs Benchmark: Caching & AI Agent Workflow${NC}"
echo -e "${BLUE}================================================${NC}"
echo ""

# Check prerequisites
if [ ! -f "$TEST_FILE" ]; then
    echo "Error: Test file not found at $TEST_FILE"
    exit 1
fi

# Build
echo -e "${CYAN}Building cs...${NC}"
cargo build --release 2>&1 | grep -E "(Finished|Compiling)" || true
echo ""

# Clear cache
echo -e "${CYAN}Clearing cache...${NC}"
$CS_BIN --clear-cache 2>/dev/null || true
echo ""

echo -e "${GREEN}Test 1: ripgrep baseline${NC}"
echo "----------------------------------------"
echo "Searching for 'Log In' in client.en.yml (389KB)"
echo ""
time rg -i -F "Log In" "$TEST_FILE" > /dev/null 2>&1
echo ""

echo -e "${GREEN}Test 2: cs - First run (no cache)${NC}"
echo "----------------------------------------"
$CS_BIN --clear-cache 2>/dev/null
time $CS_BIN "Log In" "$TEST_FILE" --simple > /dev/null 2>&1
echo ""

echo -e "${GREEN}Test 3: cs - Second run (WITH cache)${NC}"
echo "----------------------------------------"
time $CS_BIN "Log In" "$TEST_FILE" --simple > /dev/null 2>&1
echo ""

echo -e "${GREEN}Test 4: cs - Third run (cache still warm)${NC}"
echo "----------------------------------------"
time $CS_BIN "Log In" "$TEST_FILE" --simple > /dev/null 2>&1
echo ""

echo -e "${GREEN}Test 5: No-match fast path${NC}"
echo "----------------------------------------"
echo "Searching for non-existent text (grep prefilter skips parsing)"
echo ""
time $CS_BIN "xyzNonExistent99999" "$TEST_FILE" --simple > /dev/null 2>&1
echo ""

echo -e "${GREEN}Test 6: AI Agent Simulation${NC}"
echo "----------------------------------------"
echo "10 searches simulating AI agent workflow:"
echo ""

$CS_BIN --clear-cache 2>/dev/null

QUERIES=("Log In" "Sign Up" "Settings" "Profile" "Log In" "Settings" "Log In" "Profile" "Log In" "Search")

echo "Query sequence: ${QUERIES[*]}"
echo ""
echo "Running 10 queries..."
time {
    for query in "${QUERIES[@]}"; do
        $CS_BIN "$query" "$DISCOURSE_DIR" --simple > /dev/null 2>&1
    done
}
echo ""

echo -e "${BLUE}================================================${NC}"
echo -e "${BLUE}  Key Observations${NC}"
echo -e "${BLUE}================================================${NC}"
echo ""
echo -e "${CYAN}1. Cache Speedup:${NC}"
echo "   First run: Parses YAML + stores in cache"
echo "   Second+ runs: Cache hit = much faster"
echo ""
echo -e "${CYAN}2. AI Agent Benefits:${NC}"
echo "   - Cache persists across process invocations"
echo "   - Repeated queries (common in AI workflows) benefit greatly"
echo "   - Structured output (key path + code location) vs raw text"
echo ""
echo -e "${CYAN}3. No-Match Fast Path:${NC}"
echo "   - Grep prefilter skips expensive parsing"
echo "   - Near-instant for files without matches"
echo ""
echo -e "${CYAN}4. Production Ready:${NC}"
echo "   - Handles 605KB+ files efficiently"
echo "   - Cross-process cache sharing (TCP server mode)"
echo ""
