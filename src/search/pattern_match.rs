use crate::config::default_patterns;
use crate::error::Result;
use crate::parse::translation::TranslationEntry;
use crate::search::text_search::TextSearcher;
use regex::Regex;
use std::path::PathBuf;

/// Represents a code reference to a translation key
#[derive(Debug, Clone, PartialEq)]
pub struct CodeReference {
    /// Path to the file containing the reference
    pub file: PathBuf,
    /// Line number (1-indexed)
    pub line: usize,
    /// The regex pattern that matched
    pub pattern: String,
    /// The actual line of code containing the match
    pub context: String,
    /// The translation key path that was matched
    pub key_path: String,
    /// Context lines before the match
    pub context_before: Vec<String>,
    /// Context lines after the match
    pub context_after: Vec<String>,
}

/// Pattern matcher for finding i18n key usage in code
pub struct PatternMatcher {
    exclusions: Vec<String>,
    searcher: TextSearcher,
    patterns: Vec<Regex>,
}

impl PatternMatcher {
    /// Create a new PatternMatcher with default patterns
    pub fn new(base_dir: PathBuf) -> Self {
        Self {
            exclusions: Vec::new(),
            searcher: TextSearcher::new(base_dir),
            patterns: default_patterns(),
        }
    }

    /// Create a PatternMatcher with custom patterns
    pub fn with_patterns(patterns: Vec<Regex>, base_dir: PathBuf) -> Self {
        Self {
            exclusions: Vec::new(),
            searcher: TextSearcher::new(base_dir),
            patterns,
        }
    }

    /// Set exclusion patterns (file or directory names to ignore)
    pub fn set_exclusions(&mut self, exclusions: Vec<String>) {
        self.exclusions = exclusions;
    }

    /// Find all code references for a given translation key
    pub fn find_usages(&self, key_path: &str) -> Result<Vec<CodeReference>> {
        // Search for the key path using ripgrep
        let matches = self.searcher.search(key_path)?;

        let mut code_refs = Vec::new();

        for m in matches {
            // Apply exclusions: skip if any exclusion matches the file path
            let file_str = m.file.to_string_lossy();
            if self.exclusions.iter().any(|ex| file_str.contains(ex)) {
                continue;
            }

            // Skip tool's own source files and documentation (cross-platform)
            let file_str = m.file.to_string_lossy().to_lowercase();
            let path_components: Vec<_> = m
                .file
                .components()
                .map(|c| c.as_os_str().to_string_lossy().to_lowercase())
                .collect();

            // Check if path starts with "src" or "tests" (but not "tests/fixtures")
            let skip_file = !path_components.is_empty()
                && (path_components[0] == "src"
                    || (path_components[0] == "tests"
                        && (path_components.len() < 2 || path_components[1] != "fixtures")));

            // Also skip markdown files
            if skip_file
                || file_str.ends_with("readme.md")
                || file_str.ends_with("evaluation.md")
                || file_str.ends_with(".md")
            {
                continue;
            }

            // Try to match against each pattern
            for pattern in &self.patterns {
                if let Some(captures) = pattern.captures(&m.content) {
                    // Extract the key from the capture group
                    if let Some(captured_key) = captures.get(1) {
                        if captured_key.as_str() == key_path {
                            code_refs.push(CodeReference {
                                file: m.file.clone(),
                                line: m.line,
                                pattern: pattern.as_str().to_string(),
                                context: m.content.clone(),
                                key_path: key_path.to_string(),
                                context_before: m.context_before.clone(),
                                context_after: m.context_after.clone(),
                            });
                            break; // Found a match, no need to check other patterns
                        }
                    }
                }
            }
        }

        Ok(code_refs)
    }

    /// Find usages for multiple translation entries
    pub fn find_usages_batch(&self, entries: &[TranslationEntry]) -> Result<Vec<CodeReference>> {
        let mut all_refs = Vec::new();

        for entry in entries {
            let refs = self.find_usages(&entry.key)?;
            all_refs.extend(refs);
        }

        Ok(all_refs)
    }
}

impl Default for PatternMatcher {
    fn default() -> Self {
        Self::new(std::env::current_dir().unwrap())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pattern_matcher_creation() {
        let matcher = PatternMatcher::new(std::env::current_dir().unwrap());
        assert!(!matcher.patterns.is_empty());
    }

    #[test]
    fn test_code_reference_creation() {
        let code_ref = CodeReference {
            file: PathBuf::from("test.rb"),
            line: 10,
            pattern: r#"I18n\.t\(['"]([^'"]+)['"]\)"#.to_string(),
            context: r#"I18n.t('invoice.labels.add_new')"#.to_string(),
            key_path: "invoice.labels.add_new".to_string(),
            context_before: vec![],
            context_after: vec![],
        };

        assert_eq!(code_ref.file, PathBuf::from("test.rb"));
        assert_eq!(code_ref.line, 10);
        assert_eq!(code_ref.key_path, "invoice.labels.add_new");
    }

    #[test]
    fn test_pattern_matcher_with_custom_patterns() {
        let custom_patterns = vec![Regex::new(r#"custom\.t\(['"]([^'"]+)['"]\)"#).unwrap()];
        let matcher =
            PatternMatcher::with_patterns(custom_patterns, std::env::current_dir().unwrap());
        assert_eq!(matcher.patterns.len(), 1);
    }
}
