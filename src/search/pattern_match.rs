use crate::config::default_patterns;
use crate::error::Result;
use crate::parse::translation::TranslationEntry;
use crate::search::text_search::TextSearcher;
use regex::Regex;
use std::path::PathBuf;

/// Represents a code reference to a translation key
#[derive(Debug, Clone, PartialEq)]
pub struct CodeReference {
    pub file: PathBuf,
    pub line: usize,
    pub pattern: String,
    pub context: String,
    pub key_path: String,
}

/// Pattern matcher for finding i18n key usage in code
pub struct PatternMatcher {
    searcher: TextSearcher,
    patterns: Vec<Regex>,
}

impl PatternMatcher {
    /// Create a new PatternMatcher with default patterns
    pub fn new(base_dir: PathBuf) -> Self {
        Self {
            searcher: TextSearcher::new(base_dir),
            patterns: default_patterns(),
        }
    }

    /// Create a PatternMatcher with custom patterns
    pub fn with_patterns(patterns: Vec<Regex>, base_dir: PathBuf) -> Self {
        Self {
            searcher: TextSearcher::new(base_dir),
            patterns,
        }
    }

    /// Find all code references for a given translation key
    pub fn find_usages(&self, key_path: &str) -> Result<Vec<CodeReference>> {
        // Search for the key path using ripgrep
        let matches = self.searcher.search(key_path)?;

        let mut code_refs = Vec::new();

        for m in matches {
            // Skip tool's own source files and documentation
            let file_str = m.file.to_string_lossy().to_lowercase();
            if file_str.starts_with("src/")
                || (file_str.starts_with("tests/") && !file_str.starts_with("tests/fixtures/"))
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
        };

        assert_eq!(code_ref.file, PathBuf::from("test.rb"));
        assert_eq!(code_ref.line, 10);
        assert_eq!(code_ref.key_path, "invoice.labels.add_new");
    }

    #[test]
    fn test_pattern_matcher_with_custom_patterns() {
        let custom_patterns = vec![
            Regex::new(r#"custom\.t\(['"]([^'"]+)['"]\)"#).unwrap(),
        ];
        let matcher = PatternMatcher::with_patterns(custom_patterns, std::env::current_dir().unwrap());
        assert_eq!(matcher.patterns.len(), 1);
    }
}
