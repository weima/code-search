# Implementation Plan: Code Search CLI Tool

## Metadata

- **Feature Branch**: `feature/cs-001-core-functionality`
- **Planning Date**: 2025-11-28
- **Spec Reference**: `.specify/specs/code-search/spec.md`

## Summary

Implement a lightweight CLI tool in **Rust** that automates tracing UI text through YAML translation files to source code implementation. The tool uses ripgrep for fast text searching, serde_yaml for parsing, and regex for pattern matching. The core workflow: search text → identify translation files → parse YAML → extract key paths → search for key usage → build reference tree → display formatted output.

## Technical Context

### Language & Runtime
- **Language**: Rust 1.75+
- **Target Platforms**: Linux, macOS, Windows (x86_64, arm64)
- **Minimum Dependencies**: Maintain < 50 direct dependencies

**Rationale**: Rust provides:
- Performance matching C/C++ (critical for Article I: Performance First)
- Memory safety (supports Article VII: Security and Safety)
- Excellent CLI tooling ecosystem (clap, colored, etc.)
- Cross-compilation support (broad platform reach)
- Strong type system (fewer bugs, aligns with Article IV: Testing and Quality)

### Dependencies

**Core**:
- `clap` 4.x - CLI argument parsing with derive macros
- `serde` 1.x + `serde_yaml` 0.9.x - YAML parsing
- `regex` 1.x - i18n pattern matching
- `anyhow` 1.x - Error handling
- `walkdir` 2.x - File system traversal

**Process Execution**:
- `std::process::Command` - Execute ripgrep (no external crate needed)

**Testing**:
- `assert_cmd` 2.x - CLI integration testing
- `predicates` 3.x - Test assertions
- `tempfile` 3.x - Temporary test directories
- `criterion` 0.5.x - Performance benchmarking

### Project Classification
**Type**: CLI Tool (single binary)

**Deployment**: Distributed as standalone executable via:
- GitHub Releases (pre-compiled binaries)
- Cargo (source distribution: `cargo install code-search-cli`)
- Homebrew formula (macOS: `brew install code-search`)
- Debian package (Linux: `apt install code-search`)

### Performance Targets
(Aligned with Constitution Article I and spec NFR-001)

- **Response Time**:
  - Small projects (< 1k files): < 100ms
  - Medium projects (< 10k files): < 500ms
  - Large projects (< 100k files): < 2s

- **Resource Limits**:
  - Memory usage: < 100MB peak
  - CPU: Single-threaded (simplicity over parallelism for MVP)
  - Disk I/O: Minimize by using ripgrep for initial search

### Operational Constraints

- **External Dependencies**:
  - ripgrep must be installed and in PATH
  - Fail fast with clear error if ripgrep missing

- **File System**:
  - Respect `.gitignore` by default (use ripgrep's `--respect-gitignore`)
  - Support `.csignore` for project-specific exclusions (Phase 2)

- **Search Scope**:
  - Default to current directory and subdirectories
  - Maximum depth: 10 levels (prevent infinite loops per Article VII)

## Constitution Adherence Check

### Pre-Phase 0 Validation

| Article | Compliance Status | Notes |
|---------|------------------|--------|
| I: Performance First | ✅ PASS | Rust + ripgrep + < 500ms target |
| II: Simplicity and Focus | ✅ PASS | Single binary, zero config, CLI only |
| III: Developer Experience | ✅ PASS | clap for std CLI, clear error messages |
| IV: Testing and Quality | ✅ PASS | assert_cmd, criterion, 80% coverage target |
| V: Multi-Framework Support | ✅ PASS | Regex patterns extensible, YAML support |
| VI: Clear Architecture | ✅ PASS | Modular design (see Architecture section) |
| VII: Security and Safety | ✅ PASS | Rust memory safety, input validation |
| VIII: Documentation Standards | ⚠️  PENDING | README exists, needs API docs after impl |
| IX: Open Source Best Practices | ✅ PASS | Apache 2.0, semantic versioning planned |

**Outcome**: APPROVED to proceed to Phase 0

## Project Structure

### Documentation Artifacts
```
code-search/
├── .specify/
│   ├── memory/
│   │   └── constitution.md
│   └── specs/
│       └── code-search/
│           ├── spec.md
│           ├── plan.md (this file)
│           └── tasks.md
├── README.md
├── LICENSE
├── CHANGELOG.md
├── CONTRIBUTING.md
└── docs/
    ├── architecture.md
    ├── api.md
    └── patterns.md
```

### Source Code Structure
```
code-search/
├── Cargo.toml
├── Cargo.lock
├── src/
│   ├── main.rs              # CLI entry point, argument parsing
│   ├── lib.rs               # Public library API
│   ├── search/
│   │   ├── mod.rs           # Search module exports
│   │   ├── text_search.rs   # Ripgrep wrapper, literal text search
│   │   └── pattern_match.rs # i18n pattern matching (regex)
│   ├── parse/
│   │   ├── mod.rs           # Parser module exports
│   │   ├── yaml_parser.rs   # YAML file parsing (serde_yaml)
│   │   ├── key_extractor.rs # Key path extraction from YAML
│   │   └── translation.rs   # TranslationEntry data structures
│   ├── trace/
│   │   ├── mod.rs           # Call tracing module exports
│   │   ├── function_finder.rs  # Function definition detection
│   │   ├── call_extractor.rs   # Function call extraction
│   │   └── graph_builder.rs    # Call graph construction & traversal
│   ├── tree/
│   │   ├── mod.rs           # Tree module exports
│   │   ├── builder.rs       # ReferenceTree construction
│   │   └── node.rs          # TreeNode data structures
│   ├── output/
│   │   ├── mod.rs           # Output module exports
│   │   ├── formatter.rs     # Tree formatting logic
│   │   └── display.rs       # Terminal output rendering
│   ├── config/
│   │   ├── mod.rs           # Configuration exports
│   │   └── patterns.rs      # i18n pattern definitions + function patterns
│   └── error.rs             # Custom error types
├── tests/
│   ├── integration/
│   │   ├── basic_search.rs  # End-to-end search tests
│   │   ├── multi_match.rs   # Multiple match scenarios
│   │   └── error_cases.rs   # Error handling tests
│   └── fixtures/
│       ├── rails-app/       # Sample Rails project
│       ├── react-app/       # Sample React project
│       └── vue-app/         # Sample Vue project
└── benches/
    └── search_benchmark.rs  # Criterion performance tests
```

## Architecture

### System Components

```
┌─────────────────────────────────────────────────────┐
│                   CLI Entry Point                    │
│              (main.rs, clap parser)                  │
└─────────────────┬───────────────────────────────────┘
                  │
                  v
┌─────────────────────────────────────────────────────┐
│              Search Orchestrator                     │
│         (lib.rs: run_search function)                │
└──┬──────────┬──────────┬──────────┬─────────────────┘
   │          │          │          │
   v          v          v          v
┌──────┐  ┌──────┐  ┌──────┐  ┌────────┐
│ Text │  │ YAML │  │Pattern│  │  Tree  │
│Search│  │Parser│  │Matcher│  │Builder │
└──────┘  └──────┘  └──────┘  └────────┘
   │          │          │          │
   └──────────┴──────────┴──────────┘
                  │
                  v
         ┌────────────────┐
         │ Output         │
         │ Formatter      │
         └────────────────┘
                  │
                  v
         Terminal Display
```

### Module Responsibilities

**main.rs** - CLI Entry Point
- Parse command-line arguments with clap
- Validate input (non-empty search text)
- Check ripgrep availability
- Call orchestrator, handle errors
- Exit with appropriate status codes

**CLI Arguments**:
```rust
#[derive(Parser)]
#[command(name = "cs")]
struct Cli {
    /// Text or function name to search for
    search_text: String,

    /// Case-sensitive search
    #[arg(short, long)]
    case_sensitive: bool,

    /// Trace forward call graph (what does this function call?)
    #[arg(long)]
    trace: bool,

    /// Trace backward call graph (who calls this function?)
    #[arg(long)]
    traceback: bool,

    /// Trace both directions
    #[arg(long)]
    trace_all: bool,

    /// Maximum depth for call tracing (default: 3, max: 10)
    #[arg(long, default_value = "3")]
    depth: usize,
}
```

**lib.rs** - Orchestrator
```rust
pub fn run_search(query: SearchQuery) -> Result<ReferenceTree, SearchError> {
    // 1. Search for literal text
    let text_matches = text_search::search(&query.text)?;

    // 2. Identify translation files
    let translation_files = filter_translation_files(&text_matches);

    // 3. Parse translation files
    let translations = parse_translations(&translation_files)?;

    // 4. Extract key paths
    let keys = extract_key_paths(&translations, &query.text)?;

    // 5. Search for key usage in code
    let code_refs = find_code_references(&keys)?;

    // 6. Build reference tree
    let tree = build_reference_tree(&query, &translations, &code_refs)?;

    Ok(tree)
}
```

**search/text_search.rs** - Ripgrep Wrapper
```rust
pub struct TextSearcher {
    // Wrapper around std::process::Command for ripgrep
}

impl TextSearcher {
    pub fn search(&self, text: &str) -> Result<Vec<Match>, SearchError> {
        // Execute: rg --line-number --no-heading "literal text"
        // Parse output into Match structs (file, line, content)
    }
}
```

**parse/yaml_parser.rs** - YAML Parser
```rust
pub struct YamlParser;

impl YamlParser {
    pub fn parse_file(&self, path: &Path) -> Result<HashMap<String, String>, ParseError> {
        // Use serde_yaml to deserialize YAML
        // Flatten nested structures into dot-notation keys
        // Return map of "key.path" -> "translation value"
    }
}
```

**parse/key_extractor.rs** - Key Path Extraction
```rust
pub fn extract_key_path(
    yaml_value: &serde_yaml::Value,
    current_path: &str
) -> Vec<(String, String)> {
    // Recursively traverse YAML structure
    // Build dot-notation paths (e.g., "invoice.labels.add_new")
    // Return (key_path, value) tuples
}
```

**search/pattern_match.rs** - i18n Pattern Matching
```rust
pub struct PatternMatcher {
    patterns: Vec<Regex>,  // Compiled regex patterns
}

impl PatternMatcher {
    pub fn find_usages(&self, key_path: &str) -> Result<Vec<CodeReference>, SearchError> {
        // Search for key usage with ripgrep: rg "key_path"
        // Apply regex patterns to extract function calls
        // Return CodeReference structs (file, line, pattern, context)
    }

    fn default_patterns() -> Vec<String> {
        vec![
            r#"I18n\.t\(['"]([^'"]+)['"]\)"#,      // Ruby: I18n.t('key')
            r#"\bt\(['"]([^'"]+)['"]\)"#,          // Ruby/JS: t('key')
            r#"\$t\(['"]([^'"]+)['"]\)"#,          // Vue: $t('key')
            r#"i18n\.t\(['"]([^'"]+)['"]\)"#,      // JS: i18n.t('key')
        ]
    }
}
```

**tree/builder.rs** - Reference Tree Builder
```rust
pub struct ReferenceTreeBuilder;

impl ReferenceTreeBuilder {
    pub fn build(
        query: &SearchQuery,
        translations: &[TranslationEntry],
        code_refs: &[CodeReference]
    ) -> ReferenceTree {
        // Create root node (search text)
        // Add translation nodes as children
        // Add code reference nodes as grandchildren
        // Handle multiple matches (multiple branches)
    }
}
```

**output/formatter.rs** - Tree Formatter
```rust
pub struct TreeFormatter {
    max_width: usize,  // Default: 80 columns
}

impl TreeFormatter {
    pub fn format(&self, tree: &ReferenceTree) -> String {
        // Render tree with visual connectors (|, |->)
        // Format file paths and line numbers
        // Truncate long lines to fit max_width
        // Return formatted string ready for terminal display
    }
}
```

### Call Graph Tracing Module

**trace/mod.rs** - Call Graph Tracing
```rust
pub mod function_finder;
pub mod call_extractor;
pub mod graph_builder;
```

**trace/function_finder.rs** - Function Definition Finder
```rust
pub struct FunctionFinder {
    patterns: Vec<Regex>,  // Language-specific function definition patterns
}

impl FunctionFinder {
    pub fn find_definition(&self, name: &str) -> Result<Vec<FunctionDef>, SearchError> {
        // Use ripgrep to search for function definition patterns
        // Parse results to extract function name, file, line, and body range
    }

    fn default_patterns() -> Vec<Regex> {
        vec![
            // JavaScript/TypeScript
            Regex::new(r"function\s+(\w+)\s*\(").unwrap(),
            Regex::new(r"(\w+)\s*=\s*(?:async\s+)?\([^)]*\)\s*=>").unwrap(),
            Regex::new(r"(\w+)\s*:\s*(?:async\s+)?function").unwrap(),
            // Ruby
            Regex::new(r"def\s+(\w+)").unwrap(),
            // Python
            Regex::new(r"def\s+(\w+)\s*\(").unwrap(),
            // Rust
            Regex::new(r"fn\s+(\w+)\s*[<(]").unwrap(),
        ]
    }
}

pub struct FunctionDef {
    pub name: String,
    pub file: PathBuf,
    pub line: usize,
    pub body_start: usize,
    pub body_end: Option<usize>,  // None if we can't determine end
}
```

**trace/call_extractor.rs** - Function Call Extractor
```rust
pub struct CallExtractor;

impl CallExtractor {
    pub fn extract_calls(&self, func: &FunctionDef) -> Result<Vec<String>, SearchError> {
        // Read function body from file
        // Apply call pattern regex to extract function names
        // Filter out keywords, built-ins, and the function itself
    }

    pub fn find_callers(&self, func_name: &str) -> Result<Vec<CallerInfo>, SearchError> {
        // Use ripgrep to find all occurrences of func_name followed by (
        // For each match, determine the containing function
        // Return list of caller functions with file/line info
    }
}

pub struct CallerInfo {
    pub caller_name: String,
    pub file: PathBuf,
    pub line: usize,
}
```

**trace/graph_builder.rs** - Call Graph Builder
```rust
use std::collections::{HashSet, VecDeque};

pub enum TraceDirection {
    Forward,   // --trace: what does this function call?
    Backward,  // --traceback: who calls this function?
}

pub struct CallGraphBuilder {
    max_depth: usize,  // Default: 3, Max: 10
    finder: FunctionFinder,
    extractor: CallExtractor,
}

impl CallGraphBuilder {
    pub fn build_trace(
        &self,
        start_func: &str,
        direction: TraceDirection,
        depth: usize,
    ) -> Result<CallTree, SearchError> {
        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();
        let mut tree = CallTree::new(start_func);

        queue.push_back((start_func.to_string(), 0, tree.root_id()));

        while let Some((func_name, level, parent_id)) = queue.pop_front() {
            if level >= depth.min(self.max_depth) || visited.contains(&func_name) {
                continue;
            }
            visited.insert(func_name.clone());

            let related = match direction {
                TraceDirection::Forward => self.get_callees(&func_name)?,
                TraceDirection::Backward => self.get_callers(&func_name)?,
            };

            for (name, location) in related {
                let node_id = tree.add_child(parent_id, &name, location);
                queue.push_back((name, level + 1, node_id));
            }
        }

        Ok(tree)
    }

    fn get_callees(&self, func_name: &str) -> Result<Vec<(String, Location)>, SearchError> {
        let func_def = self.finder.find_definition(func_name)?
            .into_iter().next()
            .ok_or_else(|| SearchError::FunctionNotFound(func_name.to_string()))?;
        
        let calls = self.extractor.extract_calls(&func_def)?;
        // For each call, try to find its definition location
        Ok(calls.into_iter().filter_map(|name| {
            self.finder.find_definition(&name).ok()
                .and_then(|defs| defs.into_iter().next())
                .map(|def| (name, Location { file: def.file, line: def.line }))
        }).collect())
    }

    fn get_callers(&self, func_name: &str) -> Result<Vec<(String, Location)>, SearchError> {
        let callers = self.extractor.find_callers(func_name)?;
        Ok(callers.into_iter().map(|c| {
            (c.caller_name, Location { file: c.file, line: c.line })
        }).collect())
    }
}

pub struct CallTree {
    nodes: Vec<CallNode>,
}

pub struct CallNode {
    pub name: String,
    pub location: Option<Location>,
    pub children: Vec<usize>,  // Indices into nodes vec
}
```

### Data Structures

```rust
// Core data structures (in src/parse/translation.rs)

pub struct SearchQuery {
    pub text: String,
    pub case_sensitive: bool,
    pub file_patterns: Option<Vec<String>>,
}

// Call tracing mode
pub enum SearchMode {
    Translation,           // Default: i18n text tracing
    Trace { depth: usize },      // --trace: forward call graph
    Traceback { depth: usize },  // --traceback: reverse call graph
    TraceAll { depth: usize },   // --trace-all: both directions
}

pub struct TranslationEntry {
    pub file: PathBuf,
    pub line: usize,
    pub key_path: String,      // e.g., "invoice.labels.add_new"
    pub value: String,          // e.g., "add new"
    pub language: String,       // e.g., "en"
}

pub struct CodeReference {
    pub file: PathBuf,
    pub line: usize,
    pub pattern: String,        // Matched i18n pattern
    pub context: String,        // Surrounding code snippet
}

pub struct TreeNode {
    pub node_type: NodeType,
    pub content: String,
    pub location: Option<Location>,
    pub children: Vec<TreeNode>,
}

pub enum NodeType {
    Root,           // Search text
    Translation,    // Translation file entry
    KeyPath,        // Full key path
    CodeRef,        // Code reference
}

pub struct Location {
    pub file: PathBuf,
    pub line: usize,
}

pub struct ReferenceTree {
    pub root: TreeNode,
}
```

### Error Handling Strategy

```rust
// src/error.rs

#[derive(Debug, thiserror::Error)]
pub enum SearchError {
    #[error("ripgrep not found in PATH. Install from https://github.com/BurntSushi/ripgrep")]
    RipgrepNotFound,

    #[error("No translation files found containing '{0}'")]
    NoTranslationFiles(String),

    #[error("Failed to parse YAML file {file}: {reason}")]
    YamlParseError {
        file: PathBuf,
        reason: String,
    },

    #[error("Translation key '{key}' found but no code references detected")]
    NoCodeReferences {
        key: String,
    },

    #[error("Function '{0}' not found in codebase")]
    FunctionNotFound(String),

    #[error("No outgoing calls found for function '{0}'")]
    NoCallees(String),

    #[error("No incoming calls found for function '{0}'")]
    NoCallers(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Regex error: {0}")]
    Regex(#[from] regex::Error),
}
```

## Implementation Phases

### Phase 0: Environment Setup & Validation (1 day)
**Objective**: Establish development environment and validate toolchain

**Tasks**:
1. Initialize Cargo project: `cargo init --bin code-search`
2. Configure Cargo.toml with dependencies
3. Set up CI/CD (GitHub Actions):
   - Lint (clippy)
   - Test (cargo test)
   - Benchmark (cargo bench)
4. Create test fixtures (sample Rails, React, Vue projects)
5. Verify ripgrep installation on dev machine

**Success Criteria**:
- `cargo build` completes successfully
- CI pipeline runs (even if tests are empty)
- Test fixtures created with valid structure

### Phase 1: Core Infrastructure (2-3 days)
**Objective**: Build foundational modules without full integration

**Tasks**:
1. Implement CLI argument parsing (main.rs with clap)
2. Implement ripgrep wrapper (text_search.rs)
3. Implement YAML parser (yaml_parser.rs)
4. Implement basic error types (error.rs)
5. Add unit tests for each module

**Success Criteria**:
- Can invoke ripgrep from Rust code
- Can parse YAML files into flat key-value maps
- CLI help text displays correctly
- Unit tests achieve > 80% coverage

**Deliverables**:
- Working TextSearcher
- Working YamlParser
- CLI skeleton with `--help` and `--version`

### Phase 2: Integration & Orchestration (2-3 days)
**Objective**: Connect modules into end-to-end workflow

**Tasks**:
1. Implement key path extraction (key_extractor.rs)
2. Implement pattern matching (pattern_match.rs)
3. Implement orchestrator (lib.rs: run_search function)
4. Implement tree builder (tree/builder.rs)
5. Add integration tests

**Success Criteria**:
- Can trace text → translation → code in test fixtures
- Integration tests pass for Rails, React, Vue projects
- Handles single match correctly

**Deliverables**:
- Working end-to-end search for single match
- Integration tests covering all three frameworks

### Phase 3: Output & UX (1-2 days)
**Objective**: Implement tree visualization and error handling

**Tasks**:
1. Implement tree formatter (output/formatter.rs)
2. Implement error message formatting
3. Add display tests (snapshot testing)
4. Handle edge cases (no matches, multiple matches)

**Success Criteria**:
- Tree output matches spec example format
- Error messages are clear and actionable
- Output fits in 80-column terminal

**Deliverables**:
- Polished tree visualization
- Comprehensive error handling
- User-facing documentation in README

### Phase 4: Polish & Performance (1-2 days)
**Objective**: Optimize performance and add final touches

**Tasks**:
1. Implement performance benchmarks (benches/search_benchmark.rs)
2. Optimize hot paths (profiling with cargo flamegraph)
3. Add progress indicator for slow searches (optional)
4. Validate against performance targets

**Success Criteria**:
- Meets performance targets (< 500ms for 10k files)
- Memory usage < 100MB
- Benchmarks documented in CHANGELOG

**Deliverables**:
- Performance benchmarks
- Optimized binary
- Performance documentation

### Phase 5: Release Preparation (1 day)
**Objective**: Prepare for v0.1.0 release

**Tasks**:
1. Complete README with examples and installation instructions
2. Write CONTRIBUTING.md
3. Update CHANGELOG.md
4. Tag v0.1.0 release
5. Build release binaries (Linux, macOS, Windows)
6. Publish to crates.io
7. Create GitHub release with binaries

**Success Criteria**:
- All documentation complete
- Release builds succeed
- Published to crates.io
- GitHub release created

**Deliverables**:
- v0.1.0 release on GitHub and crates.io
- Installation instructions tested
- Pre-compiled binaries for 3 platforms

## Testing Strategy

### Unit Tests (80% coverage target)
```rust
// tests/text_search_test.rs
#[test]
fn test_ripgrep_wrapper_finds_matches() {
    let searcher = TextSearcher::new();
    let matches = searcher.search("test").unwrap();
    assert!(!matches.is_empty());
}

// tests/yaml_parser_test.rs
#[test]
fn test_parse_nested_yaml() {
    let yaml = "en:\n  invoice:\n    labels:\n      add_new: 'add new'";
    let parsed = YamlParser::new().parse_str(yaml).unwrap();
    assert_eq!(parsed.get("en.invoice.labels.add_new"), Some(&"add new".to_string()));
}
```

### Integration Tests (assert_cmd)
```rust
// tests/integration/basic_search.rs
use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn test_basic_search_rails_project() {
    let mut cmd = Command::cargo_bin("cs").unwrap();
    cmd.arg("add new")
        .current_dir("tests/fixtures/rails-app")
        .assert()
        .success()
        .stdout(predicate::str::contains("invoice.labels.add_new"))
        .stdout(predicate::str::contains("components/invoices.ts"));
}

#[test]
fn test_no_matches_error() {
    let mut cmd = Command::cargo_bin("cs").unwrap();
    cmd.arg("nonexistent text")
        .assert()
        .failure()
        .stderr(predicate::str::contains("No translation files found"));
}
```

### Performance Benchmarks (Criterion)
```rust
// benches/search_benchmark.rs
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn benchmark_search(c: &mut Criterion) {
    c.bench_function("search rails project", |b| {
        b.iter(|| {
            let query = SearchQuery { text: "add new".to_string(), .. };
            run_search(black_box(query))
        });
    });
}

criterion_group!(benches, benchmark_search);
criterion_main!(benches);
```

## Deployment & Distribution

### Release Process
1. **Version Bump**: Update Cargo.toml version (semantic versioning)
2. **Changelog**: Update CHANGELOG.md with changes
3. **Tag**: Create git tag `vX.Y.Z`
4. **Build Binaries**: GitHub Actions builds for:
   - Linux (x86_64-unknown-linux-gnu)
   - macOS (x86_64-apple-darwin, aarch64-apple-darwin)
   - Windows (x86_64-pc-windows-msvc)
5. **Publish**: Release to crates.io and GitHub Releases
6. **Announce**: Post release notes in README

### Distribution Channels
- **Cargo**: `cargo install code-search-cli`
- **GitHub Releases**: Download pre-compiled binaries
- **Homebrew** (Phase 2): `brew install code-search`
- **Debian Package** (Phase 3): `apt install code-search`

## Security Considerations

### Input Validation
- Sanitize search text to prevent command injection in ripgrep calls
- Validate file paths to prevent directory traversal
- Limit search depth to prevent infinite recursion

### Dependencies
- Run `cargo audit` in CI to detect vulnerable dependencies
- Keep dependencies minimal (< 50 direct deps)
- Pin versions in Cargo.lock

### Resource Limits
- Implement timeout for ripgrep searches (e.g., 10 seconds)
- Limit YAML file size (e.g., 10MB) to prevent DoS
- Respect .gitignore to avoid scanning sensitive files

## Open Questions & Decisions

### Q1: Regex Library Choice
**Options**:
- A) `regex` crate (pure Rust, well-tested)
- B) `pcre2` crate (Perl-compatible, more features)

**Decision**: Option A (`regex` crate)
**Rationale**: Pure Rust, no C dependencies, faster compilation, good enough for our patterns

### Q2: Async I/O?
**Options**:
- A) Sync I/O with std::fs (simpler)
- B) Async I/O with tokio (more complex, potentially faster)

**Decision**: Option A (sync I/O)
**Rationale**: Constitution Article II (Simplicity). Ripgrep is fast enough that async overhead isn't justified for MVP.

### Q3: Config File Format
**Options**:
- A) YAML (.csrc.yml)
- B) TOML (.csrc.toml)
- C) JSON (.csrc.json)

**Decision**: DEFERRED to Phase 2
**Rationale**: MVP is zero-config. When added, likely TOML (Rust community standard).

## Success Metrics

### Technical Metrics
- [ ] Compiles with zero warnings
- [ ] Test coverage ≥ 80%
- [ ] All benchmarks pass performance targets
- [ ] Binary size < 10MB (release build)

### User Metrics (to be measured post-release)
- GitHub stars (target: 100 in first 3 months)
- crates.io downloads (target: 500 in first month)
- Issue response time < 48 hours
- Zero critical bugs in first month

---

**Next Steps**: Proceed to tasks.md for detailed task breakdown
