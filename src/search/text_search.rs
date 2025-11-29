use crate::error::{Result, SearchError};
use std::path::PathBuf;
use std::process::Command;

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

/// Text searcher that wraps ripgrep for fast text searching
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

    /// Check if ripgrep is available in PATH
    fn check_ripgrep_installed() -> Result<()> {
        Command::new("rg")
            .arg("--version")
            .output()
            .map_err(|_| SearchError::RipgrepNotFound)?;
        Ok(())
    }

    /// Search for text and return all matches
    ///
    /// # Arguments
    /// * `text` - The text to search for
    ///
    /// # Returns
    /// A vector of Match structs containing file path, line number, and content
    pub fn search(&self, text: &str) -> Result<Vec<Match>> {
        // Check if ripgrep is installed
        Self::check_ripgrep_installed()?;

        // Build ripgrep command
        let mut cmd = Command::new("rg");

        // Set the directory to search in
        cmd.current_dir(&self.base_dir);

        // Add flags
        cmd.arg("--line-number") // Include line numbers
            .arg("--no-heading") // Don't group by file
            .arg("--with-filename"); // Always include filename

        // Case sensitivity
        if !self.case_sensitive {
            cmd.arg("--ignore-case");
        }

        // Respect .gitignore
        if !self.respect_gitignore {
            cmd.arg("--no-ignore");
        }

        // Add search text (use -F for fixed string, not regex)
        cmd.arg("--fixed-strings").arg(text);

        // Execute the command
        let output = cmd.output().map_err(|e| {
            SearchError::RipgrepExecutionFailed(format!("Failed to execute ripgrep: {}", e))
        })?;

        // ripgrep returns exit code 1 when no matches found (not an error)
        if output.status.code() == Some(1) && output.stdout.is_empty() {
            return Ok(Vec::new());
        }

        // Parse the output
        let stdout = String::from_utf8(output.stdout)?;

        self.parse_output(&stdout)
    }

    /// Parse ripgrep output into Match structs
    ///
    /// Expected format: "file:line:content"
    fn parse_output(&self, output: &str) -> Result<Vec<Match>> {
        let mut matches = Vec::new();

        for line in output.lines() {
            if line.is_empty() {
                continue;
            }

            // Split on first two colons: file:line:content
            let parts: Vec<&str> = line.splitn(3, ':').collect();

            if parts.len() != 3 {
                // Skip malformed lines
                continue;
            }

            let file = self.base_dir.join(parts[0]);
            let line_number = parts[1].parse::<usize>().unwrap_or(0);
            let content = parts[2].to_string();

            if line_number == 0 {
                // Skip if line number couldn't be parsed
                continue;
            }

            matches.push(Match {
                file,
                line: line_number,
                content,
            });
        }

        Ok(matches)
    }
}

impl Default for TextSearcher {
    fn default() -> Self {
        Self::new(std::env::current_dir().unwrap())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_output_single_line() {
        let base_dir = std::env::current_dir().unwrap();
        let searcher = TextSearcher::new(base_dir.clone());
        let output = "src/main.rs:42:    let value = \"test\";";

        let matches = searcher.parse_output(output).unwrap();

        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].file, base_dir.join("src/main.rs"));
        assert_eq!(matches[0].line, 42);
        assert_eq!(matches[0].content, "    let value = \"test\";");
    }

    #[test]
    fn test_parse_output_multiple_lines() {
        let base_dir = std::env::current_dir().unwrap();
        let searcher = TextSearcher::new(base_dir.clone());
        let output = "src/main.rs:10:first line\nsrc/lib.rs:20:second line\nsrc/main.rs:30:third line";

        let matches = searcher.parse_output(output).unwrap();

        assert_eq!(matches.len(), 3);
        assert_eq!(matches[0].file, base_dir.join("src/main.rs"));
        assert_eq!(matches[0].line, 10);
        assert_eq!(matches[1].file, base_dir.join("src/lib.rs"));
        assert_eq!(matches[1].line, 20);
        assert_eq!(matches[2].file, base_dir.join("src/main.rs"));
        assert_eq!(matches[2].line, 30);
    }

    #[test]
    fn test_parse_output_with_colons_in_content() {
        let searcher = TextSearcher::new(std::env::current_dir().unwrap());
        let output = "config.yml:5:url: http://example.com:8080";

        let matches = searcher.parse_output(output).unwrap();

        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].content, "url: http://example.com:8080");
    }

    #[test]
    fn test_parse_output_empty() {
        let searcher = TextSearcher::new(std::env::current_dir().unwrap());
        let output = "";

        let matches = searcher.parse_output(output).unwrap();

        assert_eq!(matches.len(), 0);
    }

    #[test]
    fn test_parse_output_malformed_skipped() {
        let searcher = TextSearcher::new(std::env::current_dir().unwrap());
        let output = "src/main.rs:10:valid line\nmalformed line\nsrc/lib.rs:20:another valid";

        let matches = searcher.parse_output(output).unwrap();

        // Should skip malformed line
        assert_eq!(matches.len(), 2);
        assert_eq!(matches[0].line, 10);
        assert_eq!(matches[1].line, 20);
    }

    #[test]
    fn test_builder_pattern() {
        let searcher = TextSearcher::new(std::env::current_dir().unwrap())
            .case_sensitive(true)
            .respect_gitignore(false);

        assert_eq!(searcher.case_sensitive, true);
        assert_eq!(searcher.respect_gitignore, false);
    }
    
    #[test]
    fn test_default() {
        let searcher = TextSearcher::default();

        assert_eq!(searcher.case_sensitive, false);
        assert_eq!(searcher.respect_gitignore, true);
    }
}
