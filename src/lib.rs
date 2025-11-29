pub mod config;
pub mod parse;
pub mod search;

use anyhow::{Context, Result};
use std::path::PathBuf;

// Re-export commonly used types
pub use config::default_patterns;
pub use parse::{KeyExtractor, TranslationEntry, YamlParser};
pub use search::{CodeReference, Match, PatternMatcher, TextSearcher};

/// Query parameters for searching
#[derive(Debug, Clone)]
pub struct SearchQuery {
    pub text: String,
    pub case_sensitive: bool,
    pub base_dir: Option<PathBuf>,
}

impl SearchQuery {
    pub fn new(text: String) -> Self {
        Self {
            text,
            case_sensitive: false,
            base_dir: None,
        }
    }

    pub fn with_case_sensitive(mut self, case_sensitive: bool) -> Self {
        self.case_sensitive = case_sensitive;
        self
    }

    pub fn with_base_dir(mut self, base_dir: PathBuf) -> Self {
        self.base_dir = Some(base_dir);
        self
    }
}

/// Result of a search operation
#[derive(Debug)]
pub struct SearchResult {
    pub query: String,
    pub translation_entries: Vec<TranslationEntry>,
    pub code_references: Vec<CodeReference>,
}

/// Main orchestrator function that coordinates the entire search workflow
///
/// This function:
/// 1. Searches for translation entries matching the query text
/// 2. Extracts translation keys from YAML files
/// 3. Finds code references for each translation key
/// 4. Returns a SearchResult with all findings
pub fn run_search(query: SearchQuery) -> Result<SearchResult> {
    // Determine the base directory to search
    let base_dir = query.base_dir.clone().unwrap_or_else(|| {
        std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
    });

    // Step 1: Extract translation entries matching the search text
    let extractor = KeyExtractor::new();
    let translation_entries = extractor
        .extract(&base_dir, &query.text)
        .context("Failed to extract translation entries")?;

    if translation_entries.is_empty() {
        return Ok(SearchResult {
            query: query.text,
            translation_entries: vec![],
            code_references: vec![],
        });
    }

    // Step 2: Find code references for each translation entry
    let matcher = PatternMatcher::new();
    let mut all_code_refs = Vec::new();

    for entry in &translation_entries {
        let code_refs = matcher
            .find_usages(&entry.key)
            .context(format!("Failed to find usages for key: {}", entry.key))?;
        all_code_refs.extend(code_refs);
    }

    Ok(SearchResult {
        query: query.text,
        translation_entries,
        code_references: all_code_refs,
    })
}

/// Helper function to filter translation files from search results
pub fn filter_translation_files(matches: &[Match]) -> Vec<PathBuf> {
    matches
        .iter()
        .filter(|m| {
            let path = m.file.to_string_lossy();
            path.ends_with(".yml") || path.ends_with(".yaml")
        })
        .map(|m| m.file.clone())
        .collect()
}
