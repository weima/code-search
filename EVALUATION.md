# Technical Implementation Evaluation

## Overview

This document evaluates multiple technical approaches for implementing the `code-search` (cs) tool, analyzing trade-offs across languages, architectures, and tooling choices. The goal is to build a lightweight CLI that traces UI text through i18n files to implementation code with sub-500ms performance.

## Language Options Evaluation

### Option 1: Rust ⭐ RECOMMENDED

**Pros**:
- **Performance**: Native compilation, zero runtime overhead, memory safety without GC
- **Tooling**: Excellent CLI ecosystem (clap, colored), robust error handling (thiserror, anyhow)
- **Distribution**: Single binary, no runtime dependencies, cross-compilation support
- **Safety**: Memory safety prevents common bugs (buffer overflows, null pointers)
- **Ecosystem**: Strong parsing libraries (serde_yaml, regex with Unicode support)
- **Adoption**: Growing community, good documentation, active development

**Cons**:
- **Learning Curve**: Steeper for developers unfamiliar with ownership/borrowing
- **Compile Times**: Slower than interpreted languages (2-5 min for clean build)
- **Development Speed**: More boilerplate than Python, stricter compiler

**Performance Estimates**:
- Small project (< 1k files): 50-80ms
- Medium project (< 10k files): 200-400ms
- Memory footprint: 20-50MB

**Development Time**: 12-15 days for MVP

**Verdict**: ✅ **Best choice** - Aligns perfectly with Constitution Article I (Performance First) and Article II (Simplicity). Single binary distribution is ideal for CLI tools.

---

### Option 2: Go

**Pros**:
- **Performance**: Fast compilation and execution (slightly slower than Rust)
- **Simplicity**: Easier to learn than Rust, straightforward concurrency with goroutines
- **Tooling**: Good CLI libraries (cobra), built-in testing framework
- **Distribution**: Single binary like Rust, good cross-compilation
- **Development Speed**: Faster development than Rust due to simpler syntax

**Cons**:
- **Garbage Collection**: GC pauses can affect tail latency (though usually < 10ms)
- **Error Handling**: More verbose than Rust (if err != nil pattern)
- **Ecosystem**: Fewer specialized parsing libraries compared to Rust
- **Type System**: Weaker than Rust, more runtime errors possible

**Performance Estimates**:
- Small project: 60-100ms
- Medium project: 250-450ms
- Memory footprint: 30-60MB (GC overhead)

**Development Time**: 10-12 days for MVP

**Verdict**: ⚠️ **Strong alternative** - Faster development, good performance, but GC makes performance less predictable. Consider if team is more familiar with Go.

---

### Option 3: Python

**Pros**:
- **Development Speed**: Fastest to prototype (2-3x faster than Rust)
- **Ecosystem**: Rich parsing libraries (PyYAML, regex), good subprocess handling
- **Readability**: Clear, concise code, easy to maintain
- **Tooling**: Click for CLI, pytest for testing, rich ecosystem

**Cons**:
- **Performance**: 5-10x slower than Rust for CPU-bound tasks
- **Distribution**: Requires Python runtime, complex packaging (PyInstaller for binaries)
- **Memory**: Higher memory footprint (80-150MB), GC overhead
- **Type Safety**: Dynamic typing leads to more runtime errors (even with type hints)
- **Startup Time**: 50-100ms Python interpreter startup adds to every search

**Performance Estimates**:
- Small project: 200-400ms (mostly ripgrep time)
- Medium project: 800-1200ms
- Memory footprint: 80-150MB

**Development Time**: 6-8 days for MVP

**Verdict**: ❌ **Not recommended** - Violates performance requirements. Might miss 500ms target for medium projects. Distribution complexity contradicts Article II (Simplicity).

---

### Option 4: TypeScript/Node.js

**Pros**:
- **Development Speed**: Fast prototyping, good for teams familiar with JS
- **Ecosystem**: Excellent package ecosystem (npm), good i18n libraries
- **Tooling**: Good CLI frameworks (commander, yargs), widespread familiarity

**Cons**:
- **Performance**: V8 JIT is fast but still 2-3x slower than native code
- **Startup Time**: 100-200ms Node startup time
- **Distribution**: Requires Node runtime or complex bundling (pkg, nexe)
- **Memory**: Higher footprint (60-100MB), GC pauses
- **Type Safety**: TypeScript helps but runtime errors still common

**Performance Estimates**:
- Small project: 150-300ms
- Medium project: 500-800ms
- Memory footprint: 60-100MB

**Development Time**: 8-10 days for MVP

**Verdict**: ⚠️ **Consider only if** team is JavaScript-focused and performance can be relaxed. Borderline on 500ms target.

---

## Architecture Approaches

### Approach 1: Synchronous Pipeline ⭐ RECOMMENDED

**Design**:
```
Search Text → Ripgrep → Filter → Parse YAML → Extract Keys → Find Usages → Build Tree → Display
```

**Characteristics**:
- Single-threaded execution
- Each stage completes before next begins
- Clear error handling at each step
- Simple control flow

**Pros**:
- **Simplicity**: Easy to understand and debug
- **Deterministic**: Predictable execution order
- **Error Handling**: Clear failure points
- **Resource Usage**: Lower memory (no buffering between stages)

**Cons**:
- **Parallelism**: Doesn't utilize multiple cores
- **Latency**: Must wait for each stage sequentially

**Performance**: Good enough for MVP. Ripgrep is already highly optimized.

**Verdict**: ✅ **Use for MVP** - Aligns with Constitution Article II (Simplicity). Optimize later if needed.

---

### Approach 2: Parallel Pipeline

**Design**:
```
Search Text → [Ripgrep | Ripgrep | Ripgrep] (parallel) → Merge → Parse → ...
```

**Characteristics**:
- Parallelize independent operations (e.g., searching multiple directories)
- Use thread pools or async I/O
- More complex coordination logic

**Pros**:
- **Performance**: Can achieve 30-50% speedup on multi-core systems
- **Scalability**: Better for very large projects

**Cons**:
- **Complexity**: Harder to implement and debug
- **Resource Usage**: Higher memory (buffering, thread overhead)
- **Diminishing Returns**: Ripgrep already parallelizes internally
- **Coordination Overhead**: Can slow down small projects

**Performance**: 30-50% faster on large projects, potentially slower on small projects

**Verdict**: ⚠️ **Defer to Phase 2** - Premature optimization. Add only if benchmarks show need.

---

### Approach 3: Streaming Pipeline

**Design**:
```
Search Stream → Parse Stream → Match Stream → Display Stream
```

**Characteristics**:
- Process results as they arrive
- Lower memory footprint for huge datasets
- More complex state management

**Pros**:
- **Memory**: Constant memory usage regardless of result size
- **Latency**: Can display first results while still searching
- **UX**: Progressive output for slow searches

**Cons**:
- **Complexity**: Async streams are harder to implement
- **Error Handling**: More complex with partial results
- **MVP Scope**: Overkill for typical use cases

**Verdict**: ❌ **Not for MVP** - Over-engineering for expected use cases. Consider for Phase 3 if needed.

---

## Search Strategy Options

### Option 1: Ripgrep-Only ⭐ RECOMMENDED

**Design**:
- Use ripgrep for all text searches
- Parse ripgrep output in Rust
- No additional search tools

**Pros**:
- **Simplicity**: Single external dependency
- **Performance**: Ripgrep is extremely fast (written in Rust)
- **Ubiquity**: Widely installed on developer machines
- **Features**: Supports .gitignore, glob patterns, line numbers

**Cons**:
- **Pattern Matching**: Limited to literal/regex (no AST-level matching)
- **Dependency**: Requires ripgrep in PATH

**Verdict**: ✅ **Use for MVP** - Proven, fast, aligns with simplicity principle.

---

### Option 2: Ripgrep + Semgrep

**Design**:
- Ripgrep for literal text search
- Semgrep for complex i18n pattern matching (AST-level)
- Combine results

**Pros**:
- **Accuracy**: Semgrep can find complex patterns (e.g., `t(variable + '.suffix')`)
- **Completeness**: Catches edge cases regex might miss

**Cons**:
- **Complexity**: Two tools to orchestrate
- **Performance**: Semgrep is slower (Python-based)
- **Dependency**: Another tool users must install
- **Overkill**: Most i18n patterns are simple enough for regex

**Verdict**: ⚠️ **Defer to Phase 2** - Adds complexity. Start with regex, add semgrep if users report missing patterns.

---

### Option 3: Native File Walking

**Design**:
- Walk file system with Rust (walkdir crate)
- Read and search files directly in Rust
- No external tools

**Pros**:
- **Control**: Full control over search logic
- **No Dependencies**: Works without ripgrep

**Cons**:
- **Performance**: Ripgrep is highly optimized (SIMD, mmap), hard to match
- **Complexity**: Must reimplement .gitignore parsing, glob matching, etc.
- **Maintenance**: More code to maintain

**Verdict**: ❌ **Not recommended** - Reinventing the wheel. Ripgrep solves this better.

---

## Translation File Parsing

### Option 1: serde_yaml (Rust) ⭐ RECOMMENDED

**Pros**:
- **Maturity**: Widely used, well-tested
- **Integration**: Works seamlessly with Rust
- **Performance**: Fast deserialization
- **Type Safety**: Strongly typed parsing

**Cons**:
- **Line Numbers**: Doesn't preserve line numbers natively (need workaround)
- **Error Messages**: Can be cryptic for malformed YAML

**Verdict**: ✅ **Use for MVP** - Industry standard for Rust.

**Line Number Workaround**:
```rust
// Read file line-by-line, track positions
// Parse YAML, then match values back to lines
// Or use serde_yaml::Location (experimental)
```

---

### Option 2: yaml-rust

**Pros**:
- **Pure Rust**: No unsafe code
- **Simpler**: Lighter than serde_yaml

**Cons**:
- **Less Maintained**: Last updated 2020
- **Fewer Features**: Missing some serde integrations
- **No serde**: Can't use serde derive macros

**Verdict**: ⚠️ **Backup option** - Use only if serde_yaml has issues.

---

## Call Graph Tracing Strategies

### Option 1: Regex-Based Function Extraction ⭐ RECOMMENDED FOR MVP

**Design**:
- Use regex patterns to identify function definitions and calls
- Build call graph by matching function names across files
- Traverse graph for `--trace` (forward) and `--traceback` (reverse)

**Patterns**:
```rust
// Function definitions
vec![
    Regex::new(r"function\s+(\w+)\s*\(").unwrap(),           // JS: function foo()
    Regex::new(r"(\w+)\s*=\s*(?:async\s+)?\([^)]*\)\s*=>").unwrap(), // JS: foo = () =>
    Regex::new(r"def\s+(\w+)\s*\(").unwrap(),               // Ruby/Python: def foo()
    Regex::new(r"fn\s+(\w+)\s*\(").unwrap(),                // Rust: fn foo()
]

// Function calls (within a function body)
vec![
    Regex::new(r"(\w+)\s*\(").unwrap(),                     // Generic: foo()
]
```

**Pros**:
- **Simplicity**: Reuses existing ripgrep infrastructure
- **Performance**: Fast regex matching
- **Language Agnostic**: Works across multiple languages with common patterns

**Cons**:
- **Accuracy**: May miss complex patterns (method calls, closures)
- **False Positives**: May match non-function identifiers
- **No Scope Awareness**: Can't distinguish local vs imported functions

**Verdict**: ✅ **Use for MVP** - Good enough for common cases, aligns with simplicity principle.

---

### Option 2: Tree-sitter AST Parsing

**Design**:
- Use tree-sitter for language-specific AST parsing
- Extract function definitions and call sites from AST
- Build accurate call graph with scope awareness

**Pros**:
- **Accuracy**: Precise function identification
- **Scope Awareness**: Distinguishes local, imported, and method calls
- **Language Support**: Tree-sitter has grammars for 100+ languages

**Cons**:
- **Complexity**: Significant implementation effort
- **Dependencies**: Adds tree-sitter and language grammars
- **Performance**: Slower than regex (must parse entire files)

**Verdict**: ⚠️ **Consider for Phase 2** - Better accuracy but adds complexity. Start with regex.

---

### Option 3: Language Server Protocol (LSP)

**Design**:
- Connect to language servers (TypeScript, rust-analyzer, etc.)
- Use "Go to Definition" and "Find References" capabilities
- Leverage existing IDE infrastructure

**Pros**:
- **Accuracy**: Uses same analysis as IDEs
- **Completeness**: Handles all language features
- **Maintained**: Language servers are actively developed

**Cons**:
- **Complexity**: Must manage LSP connections
- **Dependencies**: Requires language servers installed
- **Performance**: LSP startup can be slow (1-5 seconds)
- **Scope Creep**: Significant deviation from simple CLI tool

**Verdict**: ❌ **Not recommended** - Over-engineering for CLI tool. Consider for IDE plugin.

---

### Call Graph Traversal Strategy

**Depth-Limited BFS/DFS**:
```rust
fn trace_calls(start: &str, depth: usize, direction: Direction) -> CallTree {
    let mut visited = HashSet::new();
    let mut queue = VecDeque::new();
    queue.push_back((start.to_string(), 0));
    
    while let Some((func, level)) = queue.pop_front() {
        if level >= depth || visited.contains(&func) {
            continue;
        }
        visited.insert(func.clone());
        
        let related = match direction {
            Direction::Forward => find_callees(&func),  // What does func call?
            Direction::Backward => find_callers(&func), // Who calls func?
        };
        
        for r in related {
            queue.push_back((r, level + 1));
        }
    }
}
```

**Key Considerations**:
- **Cycle Detection**: Use visited set to prevent infinite loops
- **Depth Limit**: Default 3, max 10 to prevent resource exhaustion
- **Performance**: Cache function definitions to avoid re-parsing

---

## Pattern Matching Strategies

### Option 1: Hardcoded Regex Patterns ⭐ RECOMMENDED FOR MVP

**Design**:
```rust
vec![
    Regex::new(r#"I18n\.t\(['"]([^'"]+)['"]\)"#).unwrap(),
    Regex::new(r#"\bt\(['"]([^'"]+)['"]\)"#).unwrap(),
    // ...
]
```

**Pros**:
- **Simplicity**: Easy to implement and test
- **Performance**: Compiled regexes are very fast
- **Coverage**: Handles 90% of common patterns

**Cons**:
- **Flexibility**: Can't handle dynamic patterns without code changes
- **Maintenance**: Must update code to add new patterns

**Verdict**: ✅ **Use for MVP** - Good enough for initial release.

---

### Option 2: Configuration File Patterns

**Design**:
```yaml
# .csrc
patterns:
  ruby:
    - 'I18n\.t\([''"]([^''"]+)[''"]\)'
    - '\bt\([''"]([^''"]+)[''"]\)'
```

**Pros**:
- **Flexibility**: Users can add custom patterns without recompiling
- **Extensibility**: Easy to support new frameworks

**Cons**:
- **Complexity**: Need config file parsing, validation
- **Security**: User-provided regex can cause ReDoS attacks
- **Scope Creep**: MVP is zero-config

**Verdict**: ⚠️ **Defer to Phase 2** - Adds complexity, violates zero-config principle for MVP.

---

### Option 3: Semgrep Rules

**Design**:
```yaml
# semgrep rule
patterns:
  - pattern: I18n.t("...")
  - pattern: t("...")
```

**Pros**:
- **Accuracy**: AST-level matching, no false positives from strings
- **Power**: Can match complex patterns (variable interpolation)

**Cons**:
- **Dependency**: Requires semgrep installation
- **Performance**: Slower than regex (Python overhead)
- **Overkill**: Most patterns are simple

**Verdict**: ⚠️ **Consider for Phase 3** - Useful for advanced use cases, not needed for MVP.

---

## Distribution Strategies

### Option 1: Cargo + GitHub Releases ⭐ RECOMMENDED

**Distribution Channels**:
- **Cargo**: `cargo install code-search` (source distribution)
- **GitHub Releases**: Pre-compiled binaries (Linux, macOS, Windows)
- **Manual**: Download binary, add to PATH

**Pros**:
- **Simplicity**: Standard Rust distribution
- **Reach**: Cargo used by all Rust developers, GitHub accessible to all
- **Automation**: GitHub Actions can build releases automatically

**Cons**:
- **Discoverability**: Requires users to find the project
- **Platform Coverage**: Manual work for each platform

**Verdict**: ✅ **Use for initial launch** - Standard approach, proven workflow.

---

### Option 2: Homebrew (macOS/Linux)

**Distribution**:
```ruby
# Formula: code-search.rb
class CodeSearch < Formula
  desc "Code search tool for i18n tracing"
  homepage "https://github.com/user/code-search"
  url "https://github.com/user/code-search/archive/v0.1.0.tar.gz"
  # ...
end
```

**Pros**:
- **Discoverability**: Users search Homebrew for tools
- **UX**: `brew install code-search` is simple
- **Updates**: `brew upgrade` handles updates

**Cons**:
- **Platform**: macOS/Linux only (no Windows)
- **Approval**: Requires Homebrew core approval (or maintain tap)
- **Maintenance**: Must update formula for each release

**Verdict**: ⚠️ **Add in Phase 2** - Great for macOS users, worth the effort post-MVP.

---

### Option 3: OS Package Managers

**Distribution**:
- **Debian/Ubuntu**: .deb package
- **Fedora/RHEL**: .rpm package
- **Arch**: AUR package
- **Windows**: Chocolatey package

**Pros**:
- **Native**: Integrates with OS package manager
- **Trust**: Users trust OS repos more than random binaries

**Cons**:
- **Complexity**: Different packaging for each OS
- **Approval**: Most require maintainer approval
- **Lag**: Updates may lag behind GitHub releases

**Verdict**: ⚠️ **Consider for Phase 3** - High value but significant maintenance overhead.

---

## Output Formatting Options

### Option 1: Plain Text Tree ⭐ RECOMMENDED

**Example**:
```
'add new'
   |
   |-> 'add_new: add new' at line 56 of en.yml
                |
                |-> 'invoice.labels.add_new'
                         |
                         |-> I18n.t('invoice.labels.add_new') at line 128 of components/invoices.ts
```

**Pros**:
- **Readability**: Clear visual hierarchy
- **Simplicity**: No external dependencies
- **Compatibility**: Works in all terminals
- **Accessibility**: Screen reader friendly

**Cons**:
- **Limited Interaction**: No clickable links
- **Verbosity**: Can be long for many matches

**Verdict**: ✅ **Use for MVP** - Clear, simple, aligns with spec requirements.

---

### Option 2: Colored Output

**Example**: (Same tree with colors)
- Search text: **bold white**
- File paths: **blue**
- Line numbers: **yellow**
- Code snippets: **green**

**Pros**:
- **UX**: Easier to scan visually
- **Professionalism**: Looks polished

**Cons**:
- **Complexity**: Need terminal capability detection
- **Accessibility**: Problems for colorblind users, must support NO_COLOR env var
- **Portability**: May not work in all terminals

**Verdict**: ⚠️ **Phase 2** - Nice-to-have, not critical. Add with `--color` flag and respect NO_COLOR.

---

### Option 3: JSON Output

**Example**:
```json
{
  "query": "add new",
  "matches": [{
    "translation": { "file": "en.yml", "line": 56, "key": "invoice.labels.add_new" },
    "references": [{ "file": "invoices.ts", "line": 128 }]
  }]
}
```

**Pros**:
- **Machine Readable**: Can be parsed by other tools
- **Integration**: Enables editor plugins, CI/CD integration

**Cons**:
- **Readability**: Not human-friendly
- **Scope**: Not required for MVP

**Verdict**: ⚠️ **Phase 2** - Valuable for integrations. Add with `--format json` flag.

---

## Risk Analysis

### High-Priority Risks

| Risk | Impact | Probability | Mitigation |
|------|--------|-------------|------------|
| **Ripgrep not installed** | High (tool unusable) | Medium (30% of users) | Check at startup, provide clear install instructions with links |
| **Complex YAML parsing** | Medium (crashes) | Low (YAML is well-defined) | Graceful error handling, skip malformed files with warning |
| **Performance target miss** | High (violates Article I) | Low (ripgrep is fast) | Early benchmarking, profile hot paths |
| **Pattern false positives** | Medium (bad UX) | Medium (regex limitations) | Test with diverse codebases, document limitations |
| **Call graph explosion** | High (memory/time) | Medium (deep hierarchies) | Enforce depth limit (default 3, max 10), cycle detection |
| **Inaccurate call tracing** | Medium (misleading results) | Medium (regex limitations) | Document limitations, consider tree-sitter for Phase 2 |

### Medium-Priority Risks

| Risk | Impact | Probability | Mitigation |
|------|--------|-------------|------------|
| **Cross-platform issues** | Medium (Windows bugs) | Medium | Test on all platforms, use GitHub Actions matrix |
| **Large file performance** | Medium (slowdown) | Low | Implement file size limits, skip binary files |
| **Unicode in translation keys** | Low (edge case) | Low | Test with non-ASCII keys, verify regex Unicode support |

---

## Recommendations Summary

### Technology Stack (Chosen in plan.md)
1. **Language**: Rust 1.75+
2. **Search**: Ripgrep (external dependency)
3. **YAML Parsing**: serde_yaml
4. **CLI Framework**: clap 4.x (derive macros)
5. **Error Handling**: thiserror + anyhow
6. **Testing**: assert_cmd, predicates, criterion

### Architecture
1. **Style**: Synchronous pipeline (simple, deterministic)
2. **Pattern Matching**: Hardcoded regex patterns (extensible later)
3. **Output**: Plain text tree (colors in Phase 2)

### Distribution
1. **Initial**: Cargo + GitHub Releases
2. **Phase 2**: Homebrew formula
3. **Phase 3**: OS package managers

### Development Approach
1. Start with MVP (spec.md User Stories 1-3)
2. Benchmark early and often
3. Defer optimizations until proven necessary
4. Prioritize simplicity over features

---

## Alternative Scenarios

### Scenario: Team Has No Rust Experience

**Recommendation**: Use **Go** instead
- Faster to learn (1-2 weeks vs 1-2 months for Rust)
- Still achieves performance targets (250-450ms)
- Single binary distribution like Rust
- **Trade-off**: Slightly worse performance, weaker type safety

**Estimated Impact**:
- Development time: -2 days (faster learning curve)
- Performance: +50-100ms (still within targets)
- Maintenance: Similar (both have good tooling)

---

### Scenario: Need Rapid Prototype in 3 Days

**Recommendation**: Use **Python** for prototype, rewrite in Rust later
- Build MVP in Python (3 days)
- Validate UX and feature set
- Rewrite in Rust for performance (10 days)
- **Trade-off**: Extra work, but de-risks design decisions

**Estimated Impact**:
- Total time: 13 days (vs 12 for Rust-first)
- Risk reduction: High (validate before heavy Rust investment)

---

### Scenario: Must Support Custom Pattern Configuration

**Recommendation**: Add `.csrc.toml` config file
```toml
[patterns.ruby]
patterns = ['I18n\.t\([''"]([^''"]+)[''"]\)']

[patterns.custom]
patterns = ['myI18n\(([^)]+)\)']
```

**Implementation**:
- Use `serde` to deserialize TOML
- Validate regex patterns at startup
- Implement timeout for regex execution (prevent ReDoS)

**Estimated Impact**:
- Development time: +2 days
- Complexity: +20% (config parsing, validation)
- Flexibility: High (users can extend without code changes)

---

## Conclusion

The **Rust + Ripgrep + Synchronous Pipeline** approach documented in `plan.md` is the optimal choice for this project:

1. ✅ Meets all performance targets (Article I)
2. ✅ Maintains simplicity (Article II)
3. ✅ Provides excellent developer experience (Article III)
4. ✅ Enables high quality and testing (Article IV)
5. ✅ Supports multi-framework extensibility (Article V)
6. ✅ Promotes clear architecture (Article VI)
7. ✅ Ensures security and safety (Article VII)

For the **call graph tracing feature** (`--trace` / `--traceback`):
- **MVP**: Regex-based function extraction with depth-limited traversal
- **Phase 2**: Consider tree-sitter for improved accuracy if regex proves insufficient
- **Key safeguards**: Default depth of 3, max depth of 10, cycle detection to prevent infinite loops

This evaluation confirms the technical plan is sound and well-aligned with project goals.

---

**Next Steps**:
1. Proceed with implementation per `tasks.md`
2. Create benchmarks early (Task 6.1)
3. Test on diverse codebases (Rails, React, Vue)
4. Monitor performance against targets
5. Be prepared to optimize if benchmarks show issues
