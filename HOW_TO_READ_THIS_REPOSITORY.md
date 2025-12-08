# How to Read This Repository

This guide helps you navigate the code-search codebase, whether you're learning Rust, contributing, or just exploring how a CLI tool is built.

## Overview

`code-search` (command: `cs`) is a CLI tool for searching translation files and finding their usage in code. It's also designed as an educational resource demonstrating concepts from [The Rust Programming Language](https://doc.rust-lang.org/book/) book.

## Quick Start

```bash
# Clone and build
git clone https://github.com/weima/code-search.git
cd code-search
cargo build --release

# Run tests
cargo test

# Try it out
./target/release/cs "search term"
```

## Repository Structure

```
code-search/
├── src/
│   ├── main.rs              # Entry point, CLI argument parsing
│   ├── lib.rs               # Public API and module declarations
│   ├── error.rs             # Error types (Rust Book Ch 9)
│   ├── config/              # Configuration and patterns
│   ├── parse/               # Translation file parsing (YAML/JSON)
│   ├── search/              # File and text searching
│   ├── tree/                # Tree data structure for results
│   ├── output/              # Formatting and display
│   ├── trace/               # Function call tracing
│   └── cache/               # Caching layer
├── tests/                   # Integration tests
└── benches/                 # Performance benchmarks
```

## Reading Paths by Goal

### Goal: Learn Rust from Real Code

Follow this path to see Rust Book concepts in production code:

1. **Start with Error Handling** (`src/error.rs`)
   - Custom error types with `thiserror`
   - The `#[from]` attribute for error conversion
   - Type aliases for `Result<T>`
   - Rust Book Chapter 9 concepts

2. **Understand Ownership** (`src/tree/builder.rs`)
   - Borrowing with `&` references
   - When to clone vs. borrow
   - Ownership in data structures
   - Rust Book Chapter 4 concepts

3. **See the Builder Pattern** (`src/search/text_search.rs`)
   - Method chaining with `self`
   - Consuming builders
   - Rust Book Chapter 5 concepts

4. **Explore Concurrency** (`src/cache/mod.rs`, `src/search/text_search.rs`)
   - Message passing with channels
   - Shared state with `Mutex`
   - The critical `drop(tx)` pattern
   - Rust Book Chapter 16 concepts

5. **Study Iterators** (throughout codebase)
   - Iterator chains with closures
   - `filter_map`, `collect`, `enumerate`
   - Rust Book Chapter 13 concepts

### Goal: Understand the Architecture

Follow this path to understand how the tool works:

1. **Entry Point** (`src/main.rs`)
   - CLI argument parsing with `clap`
   - Main execution flow
   - Error handling at the top level

2. **Core Search Logic** (`src/search/`)
   - `text_search.rs` - Parallel file searching with ripgrep
   - `pattern_match.rs` - Regex pattern matching for code references
   - `file_search.rs` - Filename searching

3. **Parsing** (`src/parse/`)
   - `yaml_parser.rs` - YAML translation file parsing
   - `json_parser.rs` - JSON translation file parsing
   - `key_extractor.rs` - Extracting translation keys

4. **Data Structures** (`src/tree/`)
   - `node.rs` - Tree node definitions
   - `builder.rs` - Building result trees

5. **Output** (`src/output/`)
   - `formatter.rs` - Formatting results for display

6. **Caching** (`src/cache/`)
   - Local caching with `sled` database
   - Thread-safe cache operations

### Goal: Contribute to the Project

Follow this path to start contributing:

1. **Read Contributing Guide** (`CONTRIBUTING.md`)
   - Code style and conventions
   - Testing requirements
   - PR process

2. **Run Tests** 
   ```bash
   cargo test              # All tests
   cargo test --lib        # Unit tests only
   cargo test --test '*'   # Integration tests only
   ```

3. **Check Code Quality**
   ```bash
   cargo fmt               # Format code
   cargo clippy            # Lint checks
   cargo doc --open        # Generate and view docs
   ```

4. **Pick an Issue**
   - Look for `good-first-issue` labels
   - Check `learning-question` for educational discussions

5. **Understand the Module You're Working On**
   - Read the module-level documentation (`//!` comments)
   - Look at the tests for examples
   - Check related modules

## Key Files Explained

### `src/main.rs`
The entry point. Parses CLI arguments and orchestrates the search process. Start here to understand the overall flow.

### `src/error.rs`
Demonstrates production-quality error handling. Shows how to create rich, context-aware errors using the `thiserror` crate. Heavily documented with Rust Book references.

### `src/tree/builder.rs`
Shows ownership and borrowing patterns in a complex data structure. Demonstrates when to borrow (`&T`), when to clone, and when to move ownership. Heavily documented with Rust Book references.

### `src/search/text_search.rs`
The core search engine. Uses parallel file walking and channels for concurrent searching. Shows the builder pattern and thread communication.

### `src/cache/mod.rs`
Demonstrates thread-safe caching with `Mutex` and message passing. Shows how to build a concurrent cache layer.

### `src/parse/key_extractor.rs`
Orchestrates parsing of multiple translation files. Shows error handling and iterator patterns.

## Code Documentation Style

This codebase uses extensive inline documentation:

- **Module-level docs** (`//!`) - Explain the module's purpose and key concepts
- **Item-level docs** (`///`) - Document functions, structs, enums
- **Inline comments** (`//`) - Explain specific implementation decisions
- **Rust Book references** - Links to relevant chapters for learning

Example:
```rust
/// Build a reference tree from search results.
///
/// # Rust Book Reference
/// Chapter 4.2: References and Borrowing
/// https://doc.rust-lang.org/book/ch04-02-references-and-borrowing.html
///
/// # Educational Notes
/// This method takes `&SearchResult` (a reference) instead of `SearchResult`
/// (owned value) because...
pub fn build(result: &SearchResult) -> ReferenceTree {
    // OWNERSHIP: Clone the query string because TreeNode needs to own its content
    let mut root = TreeNode::new(NodeType::Root, result.query.clone());
    // ...
}
```

## Testing Strategy

The codebase has multiple test levels:

1. **Unit Tests** - In each module file (`#[cfg(test)] mod tests`)
2. **Integration Tests** - In `tests/` directory
3. **Doc Tests** - In documentation comments (marked `ignore` for examples)
4. **Benchmarks** - In `benches/` directory

Run specific tests:
```bash
cargo test error::tests::test_io_error_conversion
cargo test --test cli_integration_test
cargo bench
```

## Common Patterns

### Error Handling
```rust
// Using ? operator for propagation
let contents = std::fs::read_to_string(path)?;

// Custom error with context
return Err(SearchError::no_translation_files("search term"));

// Ignoring errors when appropriate
let _ = self.db.remove(&key);
```

### Ownership
```rust
// Borrow for reading
fn process(data: &Data) { }

// Take ownership when needed
fn consume(data: Data) { }

// Clone when you need both
let owned = borrowed.clone();
```

### Iterators
```rust
// Chain operations
results.iter()
    .filter(|r| r.is_valid())
    .map(|r| r.transform())
    .collect()
```

## Learning Resources

- **The Rust Book** - https://doc.rust-lang.org/book/
- **Rust by Example** - https://doc.rust-lang.org/rust-by-example/
- **Rust API Guidelines** - https://rust-lang.github.io/api-guidelines/
- **This Codebase** - Real-world examples with educational comments

## Getting Help

- **Issues** - Open an issue with the `learning-question` label
- **Discussions** - Use GitHub Discussions for general questions
- **Code Comments** - Many functions have detailed explanations

## Next Steps

1. Pick a reading path above based on your goal
2. Clone the repository and build it
3. Read the code with the inline documentation
4. Run tests to see how things work
5. Try making small changes and running tests
6. Open an issue if you have questions

Happy reading! 🦀
