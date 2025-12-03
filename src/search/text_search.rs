use crate::error::{Result, SearchError};
use grep_regex::RegexMatcherBuilder;
use grep_searcher::sinks::UTF8;
use grep_searcher::SearcherBuilder;
use ignore::WalkBuilder;
use std::path::PathBuf;
use std::sync::mpsc;

/// Represents a single match from a text search
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Match {
    /// File path where the match was found
    pub file: PathBuf,
    /// Line number (1-indexed)
    pub line: usize,
    /// Content of the matching line
    pub content: String,
}

/// Text searcher that uses ripgrep as a library for fast text searching
pub struct TextSearcher {
    /// Whether to respect .gitignore files
    respect_gitignore: bool,
    /// Whether search is case-sensitive
    case_sensitive: bool,
    /// The base directory to search in
    base_dir: PathBuf,
}

impl TextSearcher {
    /// Create a new TextSearcher with default settings
    pub fn new(base_dir: PathBuf) -> Self {
        Self {
            respect_gitignore: true,
            case_sensitive: false,
            base_dir,
        }
    }

    /// Set whether to respect .gitignore files (default: true)
    pub fn respect_gitignore(mut self, value: bool) -> Self {
        self.respect_gitignore = value;
        self
    }

    /// Set whether search is case-sensitive (default: false)
    pub fn case_sensitive(mut self, value: bool) -> Self {
        self.case_sensitive = value;
        self
    }

    /// Search for text and return all matches
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
            .fixed_strings(true) // Literal string matching, not regex
            .build(text)
            .map_err(|e| SearchError::Generic(format!("Failed to build matcher: {}", e)))?;

        // Create a channel for collecting matches from parallel threads
        let (tx, rx) = mpsc::channel();

        // Build parallel walker with .gitignore support
        WalkBuilder::new(&self.base_dir)
            .git_ignore(self.respect_gitignore)
            .git_global(self.respect_gitignore)
            .git_exclude(self.respect_gitignore)
            .hidden(false) // Don't skip hidden files by default
            .build_parallel()
            .run(|| {
                // Each thread gets its own sender and matcher
                let tx = tx.clone();
                let matcher = matcher.clone();

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

                    // Thread-local vector to collect matches for this file
                    let mut file_matches = Vec::new();

                    // Build searcher
                    let mut searcher = SearcherBuilder::new().line_number(true).build();

                    // Search the file
                    let result = searcher.search_path(
                        &matcher,
                        path,
                        UTF8(|line_num, line_content| {
                            file_matches.push(Match {
                                file: path_buf.clone(),
                                line: line_num as usize,
                                content: line_content.trim_end().to_string(),
                            });
                            Ok(true) // Continue searching
                        }),
                    );

                    // Send matches for this file (if any) through the channel
                    if result.is_ok() && !file_matches.is_empty() {
                        let _ = tx.send(file_matches);
                    }

                    WalkState::Continue
                })
            });

        // Drop the original sender so rx.iter() will terminate
        drop(tx);

        // Collect all matches from all threads
        let mut all_matches = Vec::new();
        for file_matches in rx {
            all_matches.extend(file_matches);
        }

        Ok(all_matches)
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
