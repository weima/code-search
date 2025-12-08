# Code-Search as a Rust Book Learning Tool

This document maps concepts from [The Rust Programming Language](https://doc.rust-lang.org/book/) (commonly called "The Rust Book") to real-world examples in the code-search codebase.

## Purpose

The code-search project demonstrates production-quality Rust code that implements concepts taught in The Rust Book. This serves as:
- **Learning resource**: See how book concepts apply in a real CLI tool
- **Best practices**: Industry-standard patterns beyond the book's basic examples
- **Code review tool**: Learn from well-tested, production code

## How to Use This Guide

1. Read a chapter in The Rust Book
2. Find the corresponding examples below
3. Read the code with the detailed inline comments
4. Compare the book's teaching examples with production usage

---

## Chapter 4: Understanding Ownership

### Example 1: String vs &str in Function Signatures
**File**: `src/search/pattern_match.rs:45-100`
**Concept**: Choosing between owned (`String`, `PathBuf`) and borrowed (`&str`, `&Path`) types

```rust
pub fn find_usages(&self, key_path: &str) -> Result<Vec<CodeReference>> {
    // Takes &str (borrowed) - caller keeps ownership
    // This is efficient and flexible (accepts &String, &str, string literals)
}

pub struct CodeReference {
    pub file: PathBuf,        // Owned - struct needs to own this data
    pub key_path: String,     // Owned - must outlive function calls
}
```

**Why this matters**: Shows the ownership trade-offs in API design.

**Rust Book reference**: https://doc.rust-lang.org/book/ch04-03-slices.html

---

### Example 2: Ownership Transfer in Tree Building
**File**: `src/tree/builder.rs:20-85`
**Concept**: Managing ownership in complex data structures

```rust
for entry in &result.translation_entries {  // Borrow for iteration
    let mut key_node = Self::build_key_node(entry);  // Create owned node
    key_node.add_child(code_node);  // Transfer ownership to tree
    used_code_refs.insert(idx);
}
```

**Why this matters**: Real-world tree construction showing when to borrow, clone, or move.

**Rust Book reference**: https://doc.rust-lang.org/book/ch04-01-what-is-ownership.html

---

## Chapter 5: Using Structs to Structure Related Data

### Example: Builder Pattern with Method Chaining
**File**: `src/search/text_search.rs:45-90`
**Concept**: Consuming `self` to enable ergonomic configuration

```rust
pub fn case_sensitive(mut self, value: bool) -> Self {
    self.case_sensitive = value;
    self  // Return self for chaining
}

// Usage enables fluent API:
let searcher = TextSearcher::new(base_dir)
    .case_sensitive(true)
    .respect_gitignore(false);
```

**Why this matters**: Shows how to design ergonomic APIs by consuming and returning `Self`.

**Rust Book reference**: https://doc.rust-lang.org/book/ch05-03-method-syntax.html

---

## Chapter 6: Enums and Pattern Matching

### Example: Enum-Based Protocol Design
**File**: `src/cache/mod.rs:40-58, 550-575`
**Concept**: Using enums with data for type-safe protocols

```rust
#[derive(Serialize, Deserialize, Debug)]
enum CacheRequest {
    Get {
        file: PathBuf,
        query: String,
        case_sensitive: bool,
        mtime_secs: u64,
        file_size: u64,
    },
    Set { /* ... */ },
    Clear,
    Ping,
}

// Exhaustive pattern matching ensures all cases handled
let resp = match req {
    CacheRequest::Get { file, query, case_sensitive, mtime_secs, file_size } => {
        // Destructure all fields
        let hit = local.get(&file, &query, case_sensitive, ts, file_size);
        CacheResponse::Get(hit)
    }
    CacheRequest::Clear => {
        let res = local.clear();
        CacheResponse::Ack(res.is_ok())
    }
    // Compiler ensures we handle all variants
};
```

**Why this matters**: Shows how enums create type-safe communication protocols.

**Rust Book reference**: https://doc.rust-lang.org/book/ch06-01-defining-an-enum.html

---

## Chapter 9: Error Handling

### Example 1: Custom Error Types with thiserror
**File**: `src/error.rs:1-95`
**Concept**: Defining rich, context-aware error types

```rust
#[derive(Debug, Error)]
pub enum SearchError {
    #[error("No translation files found containing '{text}'...")]
    NoTranslationFiles { text: String, searched_paths: String },
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),  // Automatic From implementation
}

pub type Result<T> = std::result::Result<T, SearchError>;
```

**Why this matters**: Industry-standard error handling beyond the book's examples.

**Rust Book reference**: https://doc.rust-lang.org/book/ch09-02-recoverable-errors-with-result.html

---

### Example 2: Option and Result Chaining
**File**: `src/cache/mod.rs:200-235`
**Concept**: Using combinators for error propagation

```rust
fn get(&self, ...) -> Option<Vec<TranslationEntry>> {
    // Early return with ?
    let cached_bytes = self.db.get(&key).ok()??;  // Double ? pattern
    
    // Convert Result -> Option
    let current_secs = current_mtime
        .duration_since(SystemTime::UNIX_EPOCH)
        .ok()?  // ? on Option for early return
        .as_secs();
    
    // Ignore errors when appropriate
    let _ = self.db.remove(&key);  // Let _ = suppresses "unused Result" warning
    
    Some(cached.results)
}
```

**Why this matters**: Shows advanced error handling patterns like `.ok()?`, double `??`, and ignoring errors.

**Rust Book reference**: https://doc.rust-lang.org/book/ch09-02-recoverable-errors-with-result.html#propagating-errors

---

## Chapter 10: Generic Types, Traits, and Lifetimes

### Example: Generic APIs with impl Into<T>
**File**: `src/error.rs:60-85`
**Concept**: Using trait bounds for flexible APIs

```rust
impl SearchError {
    pub fn yaml_parse_error(
        file: impl Into<PathBuf>,
        reason: impl Into<String>
    ) -> Self {
        Self::YamlParseError {
            file: file.into(),    // Accepts &Path, PathBuf, &str, String
            reason: reason.into(), // Accepts &str, String
        }
    }
}

// Can call with different types:
SearchError::yaml_parse_error("/path/to/file", "parse failed");
SearchError::yaml_parse_error(path_buf, error_string);
```

**Why this matters**: Makes APIs ergonomic by accepting multiple types that can convert to the target type.

**Rust Book reference**: https://doc.rust-lang.org/book/ch10-02-traits.html#traits-as-parameters

---

## Chapter 13: Functional Language Features

### Example: Iterator Chains with Closures
**File**: `src/search/text_search.rs:138-175`
**Concept**: Closures capturing environment variables

```rust
walk_builder.build_parallel().run(|| {
    let tx = tx.clone();      // Clone for closure
    let matcher = matcher.clone();

    Box::new(move |entry| {   // move keyword transfers ownership
        // Closure captures tx and matcher
        searcher.search_path(&matcher, path, UTF8(|line_num, line_content| {
            // Nested closure
            file_matches.push(Match { line_num, line_content });
            Ok(true)
        }))
    })
});
```

**Why this matters**: Shows real-world closure usage with move semantics and captures.

**Rust Book reference**: https://doc.rust-lang.org/book/ch13-01-closures.html#capturing-the-environment-with-closures

---

## Chapter 16: Fearless Concurrency

### Example 1: Message Passing with Channels
**File**: `src/search/text_search.rs:107-180`
**Concept**: Using mpsc channels for thread communication

```rust
pub fn search(&self, text: &str) -> Result<Vec<Match>> {
    let (tx, rx) = mpsc::channel();  // Create channel

    walk_builder.build_parallel().run(|| {
        let tx = tx.clone();  // Clone sender for each thread
        
        Box::new(move |entry| {
            let mut file_matches = Vec::new();  // Thread-local accumulator
            
            // ... search logic ...
            
            if !file_matches.is_empty() {
                let _ = tx.send(file_matches);  // Send to main thread
            }
            WalkState::Continue
        })
    });

    drop(tx);  // CRITICAL: Drop original sender so rx.iter() terminates

    let mut all_matches = Vec::new();
    for file_matches in rx {  // Collect from all threads
        all_matches.extend(file_matches);
    }

    Ok(all_matches)
}
```

**Why this matters**: Shows the critical `drop(tx)` pattern for channel termination.

**Rust Book reference**: https://doc.rust-lang.org/book/ch16-02-message-passing.html

---

### Example 2: Shared State with Mutex
**File**: `src/cache/mod.rs:149-170, 309-325`
**Concept**: Thread-safe interior mutability with Mutex

```rust
front_cache: Mutex<HashMap<Vec<u8>, CacheValue>>,

fn front_set(&self, key: Vec<u8>, value: CacheValue) {
    if let Ok(mut map) = self.front_cache.lock() {  // Acquire lock
        if map.len() >= FRONT_CACHE_CAP {
            // LRU eviction - work within locked scope
            if let Some(oldest_key) = map
                .iter()
                .min_by_key(|(_, v)| v.last_accessed)
                .map(|(k, _)| k.clone())
            {
                map.remove(&oldest_key);
            }
        }
        map.insert(key, value);
    } // Lock released here automatically
}
```

**Why this matters**: Shows thread-safe mutation with automatic lock release via RAII.

**Rust Book reference**: https://doc.rust-lang.org/book/ch16-03-shared-state.html#using-mutexes-to-allow-access-to-data-from-one-thread-at-a-time

---

## Learning Path Recommendations

### Beginner (Chapters 1-8)
1. Start with `src/main.rs` - see how a Rust program is structured
2. Read `src/error.rs` - understand Result and error types
3. Look at `src/parse/translation.rs` - simple struct definitions

### Intermediate (Chapters 9-13)
1. Study `src/search/text_search.rs` - error handling, builder pattern
2. Read `src/parse/yaml_parser.rs` - Option/Result chaining
3. Examine `src/tree/builder.rs` - ownership in complex structures

### Advanced (Chapters 14-16)
1. Analyze `src/cache/mod.rs` - concurrency, Mutex, channels
2. Study `src/search/pattern_match.rs` - regex, iterator chains
3. Review `src/trace/` - complex type relationships

---

## Contributing

If you're learning Rust using this codebase:
1. Read the relevant Rust Book chapter first
2. Find the corresponding code example above
3. Read the code with inline comments
4. Try modifying the code to test your understanding
5. Run `cargo test` to ensure you didn't break anything

Questions? Open an issue labeled `learning-question` and we'll help explain!

---

## Additional Resources

- [The Rust Book](https://doc.rust-lang.org/book/) - Official guide
- [Rust by Example](https://doc.rust-lang.org/rust-by-example/) - Learn by doing
- [Rustlings](https://github.com/rust-lang/rustlings) - Small exercises
- This codebase - Real-world production code

Happy learning! 🦀
