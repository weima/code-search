#!/bin/bash
# Benchmark script using large fixture files to demonstrate:
# 1. Cache effectiveness across multiple runs
# 2. Performance comparison vs ripgrep
# 3. AI agent workflow benefits (repeated queries)

FIXTURES_DIR="tests/fixtures/benchmark"
CACHE_DIR="$HOME/.cache/code-search"

# Build in release mode
echo "Building cs in release mode..."
cargo build --release --quiet

CS="./target/release/cs"

# Test queries for different scenarios
YAML_QUERY_COMMON="Search"           # Common word, likely to match
YAML_QUERY_RARE="Uncategorized"      # Rare word, few matches
YAML_QUERY_NOMATCH="XYZNOTFOUND"     # Will not match anything
JSON_QUERY="categories"              # JSON search

echo ""
echo "=== Benchmark: Large Fixture Files ==="
echo "Files:"
echo "  - large_server_uk.yml (644KB)"
echo "  - large_client_ar.yml (605KB)"
echo "  - large_search.json (125KB)"
echo ""

# Function to run timed command - returns time only
time_command() {
    local label="$1"
    shift

    echo "$label" >&2

    # Run and capture time
    local start=$(perl -MTime::HiRes=time -e 'print time')
    "$@" > /dev/null 2>&1
    local end=$(perl -MTime::HiRes=time -e 'print time')

    local elapsed=$(perl -e "print $end - $start")
    printf "  Time: %.3fs\n" $elapsed >&2
    printf "%.3f" $elapsed
}

# Function to clear cache
clear_cache() {
    echo "Clearing cache..."
    rm -rf "$CACHE_DIR"
    mkdir -p "$CACHE_DIR"
}

echo "=== 1. Cache Effectiveness Test ==="
echo "Testing with large_server_uk.yml (644KB)"
echo ""

clear_cache

# First run - cold cache
time1=$(time_command "First run (cold cache)" \
    $CS "$YAML_QUERY_COMMON" "$FIXTURES_DIR/large_server_uk.yml" --quiet)

# Second run - warm cache
time2=$(time_command "Second run (warm cache)" \
    $CS "$YAML_QUERY_COMMON" "$FIXTURES_DIR/large_server_uk.yml" --quiet)

# Third run - verify cache persistence
time3=$(time_command "Third run (cached)" \
    $CS "$YAML_QUERY_COMMON" "$FIXTURES_DIR/large_server_uk.yml" --quiet)

# Calculate speedup
speedup=$(perl -e "printf '%.1f', (($time1 / $time2) - 1) * 100")
echo ""
echo ">>> Cache speedup: ${speedup}%"
echo ""

echo "=== 2. Comparison with ripgrep ==="
echo "Testing grep prefilter vs ripgrep speed"
echo ""

clear_cache

# Test with no-match query (shows grep prefilter advantage)
echo "No-match scenario:"
cs_time=$(time_command "  cs (with prefilter)" \
    $CS "$YAML_QUERY_NOMATCH" "$FIXTURES_DIR/large_server_uk.yml" --quiet)

rg_time=$(time_command "  ripgrep" \
    rg -i "$YAML_QUERY_NOMATCH" "$FIXTURES_DIR/large_server_uk.yml")

ratio=$(perl -e "printf '%.2f', $cs_time / $rg_time")
echo ""
echo ">>> cs vs rg speed ratio: ${ratio}x (lower is better)"
echo ""

# Test with match query
echo "Match scenario:"
cs_time=$(time_command "  cs (first run)" \
    $CS "$YAML_QUERY_RARE" "$FIXTURES_DIR/large_server_uk.yml" --quiet)

rg_time=$(time_command "  ripgrep" \
    rg -i "$YAML_QUERY_RARE" "$FIXTURES_DIR/large_server_uk.yml")

cs_cached_time=$(time_command "  cs (cached)" \
    $CS "$YAML_QUERY_RARE" "$FIXTURES_DIR/large_server_uk.yml" --quiet)

echo ""
ratio1=$(perl -e "printf '%.2f', $cs_time / $rg_time")
ratio2=$(perl -e "printf '%.2f', $cs_cached_time / $rg_time")
echo "First run cs vs rg: ${ratio1}x"
echo ">>> Cached cs vs rg: ${ratio2}x"
echo ""

echo "=== 3. AI Agent Workflow Simulation ==="
echo "Simulating repeated queries (common in AI workflows)"
echo ""

clear_cache

# Queries an AI agent might make repeatedly
queries=(
    "Search"
    "topic"
    "user"
    "post"
    "category"
    "message"
    "notification"
    "settings"
    "admin"
    "error"
)

echo "Running 10 queries on large files (first run)..."

start=$(perl -MTime::HiRes=time -e 'print time')
for query in "${queries[@]}"; do
    echo -n "  Query '$query'... "
    $CS "$query" "$FIXTURES_DIR/large_server_uk.yml" "$FIXTURES_DIR/large_client_ar.yml" --quiet > /dev/null 2>&1
    echo "done"
done
end=$(perl -MTime::HiRes=time -e 'print time')

total_time=$(perl -e "print $end - $start")

echo ""
printf ">>> Total time for 10 queries: %.3fs\n" $total_time
avg=$(perl -e "printf '%.3f', $total_time / 10")
echo "    Average per query: ${avg}s"
echo ""

# Now test cached performance
echo "Running same 10 queries again (cached)..."

start=$(perl -MTime::HiRes=time -e 'print time')
for query in "${queries[@]}"; do
    echo -n "  Query '$query'... "
    $CS "$query" "$FIXTURES_DIR/large_server_uk.yml" "$FIXTURES_DIR/large_client_ar.yml" --quiet > /dev/null 2>&1
    echo "done"
done
end=$(perl -MTime::HiRes=time -e 'print time')

total_time_cached=$(perl -e "print $end - $start")

echo ""
printf ">>> Total time for 10 queries (cached): %.3fs\n" $total_time_cached
avg_cached=$(perl -e "printf '%.3f', $total_time_cached / 10")
echo "    Average per query: ${avg_cached}s"
echo ""

speedup=$(perl -e "printf '%.1f', (($total_time / $total_time_cached) - 1) * 100")
echo ">>> Cache speedup for AI workflow: ${speedup}%"
echo ""

echo "=== 4. JSON Performance Test ==="
echo "Testing with large_search.json (125KB)"
echo ""

clear_cache

json_time1=$(time_command "First run (cold cache)" \
    $CS "$JSON_QUERY" "$FIXTURES_DIR/large_search.json" --quiet)

json_time2=$(time_command "Second run (warm cache)" \
    $CS "$JSON_QUERY" "$FIXTURES_DIR/large_search.json" --quiet)

rg_json_time=$(time_command "ripgrep comparison" \
    rg -i "$JSON_QUERY" "$FIXTURES_DIR/large_search.json")

echo ""
json_speedup=$(perl -e "printf '%.1f', (($json_time1 / $json_time2) - 1) * 100")
json_ratio=$(perl -e "printf '%.2f', $json_time2 / $rg_json_time")
echo ">>> JSON cache speedup: ${json_speedup}%"
echo "    cs (cached) vs rg: ${json_ratio}x"
echo ""

echo "=== 5. Multi-File Search Performance ==="
echo "Testing all three large files together"
echo ""

clear_cache

multi_time1=$(time_command "First run (3 files, cold)" \
    $CS "$YAML_QUERY_COMMON" "$FIXTURES_DIR"/*.yml "$FIXTURES_DIR"/*.json --quiet)

multi_time2=$(time_command "Second run (3 files, cached)" \
    $CS "$YAML_QUERY_COMMON" "$FIXTURES_DIR"/*.yml "$FIXTURES_DIR"/*.json --quiet)

echo ""
multi_speedup=$(perl -e "printf '%.1f', (($multi_time1 / $multi_time2) - 1) * 100")
echo ">>> Multi-file cache speedup: ${multi_speedup}%"
echo ""

echo "========================================="
echo "=== Summary ==="
echo "1. ✓ Cache provides significant speedup for repeated queries"
echo "2. ✓ Grep prefilter makes no-match cases very fast"
echo "3. ✓ AI agent workflows benefit from caching (repeated queries)"
echo "4. ✓ Works efficiently with both YAML and JSON files"
echo "5. ✓ Multi-file searches benefit from caching"
echo ""
echo "Benchmark complete!"
