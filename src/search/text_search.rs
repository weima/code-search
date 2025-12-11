//! # Builder Pattern and Concurrency - Rust Book Chapters 5, 10, 16
//!
//! This module demonstrates the builder pattern and concurrent programming from
//! [The Rust Book](https://doc.rust-lang.org/book/).
//!
//! ## Key Concepts Demonstrated
//!
//! 1. **Builder Pattern** (Chapters 5.3, 10.2)
//!    - Method chaining by consuming and returning `Self`
//!    - Ergonomic API design with sensible defaults
//!    - Type-state pattern for compile-time guarantees
//!
//! 2. **Message Passing with Channels** (Chapter 16.2)
//!    - Using `mpsc::channel()` for thread communication
//!    - The critical `drop(tx)` pattern for channel termination
//!    - Collecting results from parallel workers
//!
//! 3. **Closures Capturing Environment** (Chapter 13.1)
//!    - `move` closures transferring ownership to threads
//!    - Cloning for shared access across threads
//!    - Nested closures with different capture modes
//!
//! ## Learning Notes
//!
//! **Why the builder pattern?**
//! - Provides a fluent, readable API: `TextSearcher::new(dir).case_sensitive(true).search("text")`
//! - Allows optional configuration without many constructors
//! - Makes defaults explicit and overridable
//!
//! **Why channels for concurrency?**
//! - Safe message passing between threads (no shared mutable state)
//! - Natural fit for parallel file searching (many producers, one consumer)
//! - Rust's ownership prevents data races at compile time

use crate::error::{Result, SearchError};
use grep_matcher::Matcher;
use grep_regex::RegexMatcherBuilder;
use grep_searcher::sinks::UTF8;
use grep_searcher::SearcherBuilder;
use ignore::overrides::OverrideBuilder;
use ignore::WalkBuilder;
use std::path::PathBuf;
use std::sync::mpsc;

/// Represents a single match from a text search.
///
/// # Rust Book Reference
///
/// **Chapter 5.1: Defining and Instantiating Structs**
/// https://doc.rust-lang.org/book/ch05-01-defining-structs.html
///
/// This is a simple data-carrying struct with public fields.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Match {
    /// File path where the match was found
    pub file: PathBuf,
    /// Line number (1-indexed)
    pub line: usize,
    /// Content of the matching line
    pub content: String,
    /// Context lines before the match
    pub context_before: Vec<String>,
    /// Context lines after the match
    pub context_after: Vec<String>,
}

/// Text searcher that uses ripgrep as a library for fast text searching.
///
/// # Rust Book Reference
///
/// **Chapter 5.3: Method Syntax**
/// https://doc.rust-lang.org/book/ch05-03-method-syntax.html
///
/// **Chapter 10.2: Traits as Parameters**
/// https://doc.rust-lang.org/book/ch10-02-traits.html
///
/// # Educational Notes - The Builder Pattern
///
/// This struct demonstrates the builder pattern, a common Rust idiom for
/// constructing complex objects with many optional parameters.
///
/// **Key characteristics:**
/// 1. Private fields prevent direct construction
/// 2. `new()` provides sensible defaults
/// 3. Builder methods take `mut self` and return `Self`
/// 4. Final `search()` method takes `&self` (doesn't consume)
///
/// **Why this pattern?**
/// - Avoids constructors with many parameters
/// - Makes optional configuration explicit
/// - Enables method chaining for readability
/// - Compile-time validation of configuration
pub struct TextSearcher {
    /// Whether to respect .gitignore files
    respect_gitignore: bool,
    /// Whether search is case-sensitive
    case_sensitive: bool,
    /// Whether to match whole words only
    word_match: bool,
    /// Whether to treat the query as a regex
    is_regex: bool,
    /// Glob patterns to include
    globs: Vec<String>,
    /// Patterns to exclude from search
    exclusions: Vec<String>,
    /// The base directory to search in
    base_dir: PathBuf,
    /// Number of context lines to show before and after matches
    context_lines: usize,
}

impl TextSearcher {
    /// Create a new TextSearcher with default settings.
    ///
    /// # Rust Book Reference
    ///
    /// **Chapter 5.3: Method Syntax - Associated Functions**
    /// https://doc.rust-lang.org/book/ch05-03-method-syntax.html#associated-functions
    ///
    /// # Educational Notes - Builder Constructor
    ///
    /// This is an associated function (not a method) that creates a new instance.
    /// It's called with `TextSearcher::new(...)` rather than on an instance.
    ///
    /// **Design decisions:**
    /// - Takes only required parameter (`base_dir`)
    /// - Sets sensible defaults for all optional fields
    /// - Returns owned `Self` (not `&Self`)
    ///
    /// **Usage pattern:**
    /// ```rust,ignore
    /// let searcher = TextSearcher::new(PathBuf::from("/path"))
    ///     .case_sensitive(true)    // Optional: override default
    ///     .respect_gitignore(false); // Optional: override default
    /// ```
    pub fn new(base_dir: PathBuf) -> Self {
        Self {
            respect_gitignore: true,
            case_sensitive: false,
            word_match: false,
            is_regex: false,
            globs: Vec::new(),
            exclusions: Vec::new(),
            base_dir,
            context_lines: 2, // Default: 2 lines before and after
        }
    }

    /// Set whether to respect .gitignore files (default: true).
    ///
    /// # Rust Book Reference
    ///
    /// **Chapter 5.3: Method Syntax**
    /// https://doc.rust-lang.org/book/ch05-03-method-syntax.html
    ///
    /// # Educational Notes - Builder Method Pattern
    ///
    /// This method demonstrates the builder pattern's key technique:
    ///
    /// ```rust,ignore
    /// pub fn respect_gitignore(mut self, value: bool) -> Self {
    /// //                       ^^^^^^^^              ^^^^^^
    /// //                       Takes ownership       Returns ownership
    ///     self.respect_gitignore = value;
    ///     self  // Return modified self for chaining
    /// }
    /// ```
    ///
    /// **Why `mut self` instead of `&mut self`?**
    /// - `mut self` takes ownership, allowing method chaining
    /// - `&mut self` would require explicit returns and be less ergonomic
    /// - Ownership transfer prevents using partially-configured builders
    ///
    /// **Method chaining:**
    /// ```rust,ignore
    /// TextSearcher::new(dir)
    ///     .respect_gitignore(false)  // Consumes and returns Self
    ///     .case_sensitive(true)      // Consumes and returns Self
    ///     .search("text")            // Final method takes &self
    /// ```
    pub fn respect_gitignore(mut self, value: bool) -> Self {
        self.respect_gitignore = value;
        self
    }

    /// Set whether search is case-sensitive (default: false).
    ///
    /// # Educational Notes
    ///
    /// Same builder pattern as `respect_gitignore()`. Each builder method:
    /// 1. Takes ownership of `self`
    /// 2. Modifies the field
    /// 3. Returns ownership for chaining
    pub fn case_sensitive(mut self, value: bool) -> Self {
        self.case_sensitive = value;
        self
    }

    /// Set whether to match whole words only (default: false)
    pub fn word_match(mut self, value: bool) -> Self {
        self.word_match = value;
        self
    }

    /// Set whether to treat the query as a regex (default: false)
    pub fn is_regex(mut self, value: bool) -> Self {
        self.is_regex = value;
        self
    }

    /// Add glob patterns to include
    pub fn add_globs(mut self, globs: Vec<String>) -> Self {
        self.globs.extend(globs);
        self
    }

    /// Add exclusion patterns
    pub fn add_exclusions(mut self, exclusions: Vec<String>) -> Self {
        self.exclusions.extend(exclusions);
        self
    }

    /// Set number of context lines to show before and after matches (default: 2)
    pub fn context_lines(mut self, lines: usize) -> Self {
        self.context_lines = lines;
        self
    }

    /// Search for text and return all matches.
    ///
    /// # Rust Book Reference
    ///
    /// **Chapter 16.2: Message Passing with Channels**
    /// https://doc.rust-lang.org/book/ch16-02-message-passing.html
    ///
    /// **Chapter 13.1: Closures**
    /// https://doc.rust-lang.org/book/ch13-01-closures.html
    ///
    /// # Educational Notes - Concurrent Search with Channels
    ///
    /// This method demonstrates concurrent programming using message passing:
    ///
    /// 1. **Create channel**: `let (tx, rx) = mpsc::channel()`
    /// 2. **Spawn workers**: Each thread gets a cloned sender (`tx.clone()`)
    /// 3. **Send results**: Workers send matches through the channel
    /// 4. **Drop original sender**: Critical for terminating the receiver
    /// 5. **Collect results**: Main thread receives all matches
    ///
    /// **Why channels instead of shared state?**
    /// - No locks needed (no `Mutex`)
    /// - Ownership prevents data races
    /// - Natural producer-consumer pattern
    /// - Rust's type system ensures thread safety
    ///
    /// # Arguments
    /// * `text` - The text to search for
    ///
    /// # Returns
    /// A vector of Match structs containing file path, line number, and content
    pub fn search(&self, text: &str) -> Result<Vec<Match>> {
        // Build the regex matcher with fixed string (literal) matching
        let matcher = RegexMatcherBuilder::new()
            .case_insensitive(!self.case_sensitive)
            .word(self.word_match)
            .fixed_strings(!self.is_regex) // Use fixed strings unless regex is enabled
            .build(text)
            .map_err(|e| SearchError::Generic(format!("Failed to build matcher: {}", e)))?;

        // Build searcher with context lines (for reference, but we use manual context capture)
        let _searcher = SearcherBuilder::new()
            .before_context(self.context_lines)
            .after_context(self.context_lines)
            .line_number(true)
            .build();

        // CHANNEL CREATION: Create a channel for collecting matches from parallel threads
        // Chapter 16.2: mpsc = "multiple producer, single consumer"
        // tx (transmitter) can be cloned for each thread
        // rx (receiver) stays in the main thread
        let (tx, rx) = mpsc::channel();

        // Build parallel walker with .gitignore support
        // Build overrides if any globs are provided
        let mut builder = WalkBuilder::new(&self.base_dir);
        let mut walk_builder = builder
            .git_ignore(self.respect_gitignore)
            .git_global(self.respect_gitignore)
            .git_exclude(self.respect_gitignore)
            .hidden(false); // Don't skip hidden files by default

        if !self.globs.is_empty() {
            let mut override_builder = OverrideBuilder::new(&self.base_dir);
            for glob in &self.globs {
                if let Err(e) = override_builder.add(glob) {
                    return Err(SearchError::Generic(format!(
                        "Invalid glob pattern '{}': {}",
                        glob, e
                    )));
                }
            }
            if let Ok(overrides) = override_builder.build() {
                walk_builder = walk_builder.overrides(overrides);
            }
        }

        walk_builder.build_parallel().run(|| {
            // CLONING FOR THREADS: Each thread gets its own sender and matcher
            // Chapter 16.2: Clone tx so each thread can send messages
            // Chapter 13.1: These clones will be moved into the closure below
            let tx = tx.clone();
            let matcher = matcher.clone();
            let context_lines = self.context_lines;

            // MOVE CLOSURE: Transfer ownership of tx and matcher to this thread
            // Chapter 13.1: The `move` keyword forces the closure to take ownership
            // Without `move`, the closure would try to borrow, which doesn't work across threads
            Box::new(move |entry| {
                use ignore::WalkState;

                let entry = match entry {
                    Ok(e) => e,
                    Err(_) => return WalkState::Continue,
                };

                // Skip directories
                if entry.file_type().is_none_or(|ft| ft.is_dir()) {
                    return WalkState::Continue;
                }

                let path = entry.path();
                let path_buf = path.to_path_buf();

                // THREAD-LOCAL ACCUMULATOR: Each thread collects its own matches
                // This avoids contention - no need for Mutex or Arc
                let mut file_matches = Vec::new();

                // Use grep-searcher to search the file with context
                let mut searcher = SearcherBuilder::new()
                    .before_context(context_lines)
                    .after_context(context_lines)
                    .line_number(true)
                    .build();

                let result = searcher.search_path(
                    &matcher,
                    path,
                    UTF8(|line_num, line_content| {
                        // line_content is already a &str from UTF8 sink
                        let line_str = line_content;

                        // For now, we'll collect all matches and handle context parsing later
                        // The grep library provides context in the output, but we need to parse it
                        file_matches.push(Match {
                            file: path_buf.clone(),
                            line: line_num as usize,
                            content: line_str.trim_end().to_string(),
                            context_before: Vec::new(), // Will be populated by post-processing
                            context_after: Vec::new(),  // Will be populated by post-processing
                        });

                        Ok(true) // Continue searching
                    }),
                );

                // SEND THROUGH CHANNEL: Send matches to main thread
                // Chapter 16.2: tx.send() transfers ownership of file_matches
                // The `let _ =` ignores send errors (receiver might be dropped)
                if result.is_ok() && !file_matches.is_empty() {
                    let _ = tx.send(file_matches);
                }

                WalkState::Continue
            })
        });

        // CRITICAL: Drop the original sender so rx.iter() will terminate
        // Chapter 16.2: The receiver's iterator only ends when ALL senders are dropped
        // We cloned tx for each thread, but we still have the original here
        // Without this drop, rx would wait forever!
        drop(tx);

        // COLLECT RESULTS: Receive all matches from worker threads
        // Chapter 16.2: The for loop iterates until all senders are dropped
        // This blocks until all threads finish and send their results
        let mut all_matches = Vec::new();
        for file_matches in rx {
            all_matches.extend(file_matches);
        }

        // Post-process to add context lines using a second pass
        self.add_context_to_matches(&mut all_matches, &matcher)?;

        Ok(all_matches)
    }

    /// Add context lines to matches by re-reading files
    fn add_context_to_matches(&self, matches: &mut [Match], _matcher: &impl Matcher) -> Result<()> {
        use std::collections::HashMap;

        // Group matches by file to minimize file reads
        let mut matches_by_file: HashMap<PathBuf, Vec<usize>> = HashMap::new();
        for (idx, m) in matches.iter().enumerate() {
            matches_by_file.entry(m.file.clone()).or_default().push(idx);
        }

        // Process each file
        for (file_path, match_indices) in matches_by_file {
            if let Ok(content) = std::fs::read_to_string(&file_path) {
                let lines: Vec<&str> = content.lines().collect();

                for &match_idx in &match_indices {
                    let match_ref = &mut matches[match_idx];
                    let line_idx = match_ref.line.saturating_sub(1); // Convert to 0-indexed

                    if line_idx < lines.len() {
                        // Capture context lines
                        let context_start = line_idx.saturating_sub(self.context_lines);
                        let context_end =
                            std::cmp::min(line_idx + self.context_lines + 1, lines.len());

                        match_ref.context_before = lines[context_start..line_idx]
                            .iter()
                            .map(|s| s.to_string())
                            .collect();

                        match_ref.context_after = lines[line_idx + 1..context_end]
                            .iter()
                            .map(|s| s.to_string())
                            .collect();
                    }
                }
            }
        }

        Ok(())
    }
}

impl Default for TextSearcher {
    fn default() -> Self {
        Self::new(std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_basic_search() {
        let temp_dir = TempDir::new().unwrap();
        fs::write(
            temp_dir.path().join("test.txt"),
            "hello world\nfoo bar\nhello again",
        )
        .unwrap();

        let searcher = TextSearcher::new(temp_dir.path().to_path_buf());
        let matches = searcher.search("hello").unwrap();

        assert_eq!(matches.len(), 2);
        assert_eq!(matches[0].line, 1);
        assert_eq!(matches[0].content, "hello world");
        assert_eq!(matches[1].line, 3);
        assert_eq!(matches[1].content, "hello again");
    }

    #[test]
    fn test_case_insensitive_default() {
        let temp_dir = TempDir::new().unwrap();
        fs::write(
            temp_dir.path().join("test.txt"),
            "Hello World\nHELLO\nhello",
        )
        .unwrap();

        let searcher = TextSearcher::new(temp_dir.path().to_path_buf());
        let matches = searcher.search("hello").unwrap();

        assert_eq!(matches.len(), 3); // Should match all variations
    }

    #[test]
    fn test_case_sensitive() {
        let temp_dir = TempDir::new().unwrap();
        fs::write(
            temp_dir.path().join("test.txt"),
            "Hello World\nHELLO\nhello",
        )
        .unwrap();

        let searcher = TextSearcher::new(temp_dir.path().to_path_buf()).case_sensitive(true);
        let matches = searcher.search("hello").unwrap();

        assert_eq!(matches.len(), 1); // Should only match exact case
        assert_eq!(matches[0].content, "hello");
    }

    #[test]
    fn test_no_matches() {
        let temp_dir = TempDir::new().unwrap();
        fs::write(temp_dir.path().join("test.txt"), "foo bar baz").unwrap();

        let searcher = TextSearcher::new(temp_dir.path().to_path_buf());
        let matches = searcher.search("notfound").unwrap();

        assert_eq!(matches.len(), 0);
    }

    #[test]
    fn test_multiple_files() {
        let temp_dir = TempDir::new().unwrap();
        fs::write(temp_dir.path().join("file1.txt"), "target line 1").unwrap();
        fs::write(temp_dir.path().join("file2.txt"), "target line 2").unwrap();
        fs::write(temp_dir.path().join("file3.txt"), "other content").unwrap();

        let searcher = TextSearcher::new(temp_dir.path().to_path_buf());
        let matches = searcher.search("target").unwrap();

        assert_eq!(matches.len(), 2);
    }

    #[test]
    fn test_gitignore_respected() {
        let temp_dir = TempDir::new().unwrap();

        // Initialize git repository (required for .gitignore to work)
        fs::create_dir(temp_dir.path().join(".git")).unwrap();

        // Create .gitignore
        fs::write(temp_dir.path().join(".gitignore"), "ignored.txt\n").unwrap();

        // Create files
        fs::write(temp_dir.path().join("ignored.txt"), "target content").unwrap();
        fs::write(temp_dir.path().join("tracked.txt"), "target content").unwrap();

        let searcher = TextSearcher::new(temp_dir.path().to_path_buf()).respect_gitignore(true);
        let matches = searcher.search("target").unwrap();

        // Should only find in tracked.txt
        assert_eq!(matches.len(), 1);
        assert!(matches[0].file.ends_with("tracked.txt"));
    }

    #[test]
    fn test_gitignore_disabled() {
        let temp_dir = TempDir::new().unwrap();

        // Initialize git repository
        fs::create_dir(temp_dir.path().join(".git")).unwrap();

        // Create .gitignore
        fs::write(temp_dir.path().join(".gitignore"), "ignored.txt\n").unwrap();

        // Create files
        fs::write(temp_dir.path().join("ignored.txt"), "target content").unwrap();
        fs::write(temp_dir.path().join("tracked.txt"), "target content").unwrap();

        let searcher = TextSearcher::new(temp_dir.path().to_path_buf()).respect_gitignore(false);
        let matches = searcher.search("target").unwrap();

        // Should find in both files
        assert_eq!(matches.len(), 2);
    }

    #[test]
    fn test_builder_pattern() {
        let searcher = TextSearcher::new(std::env::current_dir().unwrap())
            .case_sensitive(true)
            .respect_gitignore(false);

        assert!(searcher.case_sensitive);
        assert!(!searcher.respect_gitignore);
    }

    #[test]
    fn test_default() {
        let searcher = TextSearcher::default();

        assert!(!searcher.case_sensitive);
        assert!(searcher.respect_gitignore);
        assert_eq!(searcher.context_lines, 2);
    }

    #[test]
    fn test_special_characters() {
        let temp_dir = TempDir::new().unwrap();
        fs::write(
            temp_dir.path().join("test.txt"),
            "price: $19.99\nurl: http://example.com",
        )
        .unwrap();

        let searcher = TextSearcher::new(temp_dir.path().to_path_buf());

        // Test with special regex characters (should be treated as literals)
        let matches = searcher.search("$19.99").unwrap();
        assert_eq!(matches.len(), 1);

        let matches = searcher.search("http://").unwrap();
        assert_eq!(matches.len(), 1);
    }

    #[test]
    fn test_line_numbers_accurate() {
        let temp_dir = TempDir::new().unwrap();
        let content = "line 1\nline 2\ntarget line 3\nline 4\ntarget line 5\nline 6";
        fs::write(temp_dir.path().join("test.txt"), content).unwrap();

        let searcher = TextSearcher::new(temp_dir.path().to_path_buf());
        let matches = searcher.search("target").unwrap();

        assert_eq!(matches.len(), 2);
        assert_eq!(matches[0].line, 3);
        assert_eq!(matches[1].line, 5);
    }
}
