pub mod config;
pub mod error;
pub mod output;
pub mod parse;
pub mod search;
pub mod trace;
pub mod tree;

use std::path::PathBuf;

// Re-export commonly used types
pub use config::default_patterns;
pub use error::{Result, SearchError};
pub use output::TreeFormatter;
pub use parse::{KeyExtractor, TranslationEntry, YamlParser};
pub use search::{CodeReference, Match, PatternMatcher, TextSearcher};
pub use trace::{
    CallExtractor, CallGraphBuilder, CallNode, CallTree, FunctionDef, FunctionFinder,
    TraceDirection,
};
pub use tree::{Location, NodeType, ReferenceTree, ReferenceTreeBuilder, TreeNode};

/// Query parameters for tracing
#[derive(Debug, Clone)]
pub struct TraceQuery {
    pub function_name: String,
    pub direction: TraceDirection,
    pub max_depth: usize,
    pub base_dir: Option<PathBuf>,
}

impl TraceQuery {
    pub fn new(function_name: String, direction: TraceDirection, max_depth: usize) -> Self {
        Self {
            function_name,
            direction,
            max_depth,
            base_dir: None,
        }
    }

    pub fn with_base_dir(mut self, base_dir: PathBuf) -> Self {
        self.base_dir = Some(base_dir);
        self
    }
}

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
    let base_dir = query
        .base_dir
        .clone()
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));

    // Step 1: Extract translation entries matching the search text
    let extractor = KeyExtractor::new();
    let translation_entries = extractor.extract(&base_dir, &query.text)?;

    if translation_entries.is_empty() {
        return Ok(SearchResult {
            query: query.text,
            translation_entries: vec![],
            code_references: vec![],
        });
    }

    // Step 2: Find code references for each translation entry
    // Search for full key AND partial keys (for namespace caching patterns)
    let matcher = PatternMatcher::new(base_dir);
    let mut all_code_refs = Vec::new();

    for entry in &translation_entries {
        // Generate all key variations (full key + partial keys)
        let key_variations = generate_partial_keys(&entry.key);

        // Search for each key variation
        for key in &key_variations {
            let code_refs = matcher.find_usages(key)?;
            all_code_refs.extend(code_refs);
        }
    }

    // Deduplicate code references (in case same reference matches multiple key variations)
    all_code_refs.sort_by(|a, b| a.file.cmp(&b.file).then(a.line.cmp(&b.line)));
    all_code_refs.dedup_by(|a, b| a.file == b.file && a.line == b.line);

    Ok(SearchResult {
        query: query.text,
        translation_entries,
        code_references: all_code_refs,
    })
}

pub fn run_trace(query: TraceQuery) -> Result<Option<CallTree>> {
    let base_dir = query
        .base_dir
        .clone()
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));

    let finder = FunctionFinder::new(base_dir.clone());
    if let Some(start_fn) = finder.find_function(&query.function_name) {
        let extractor = CallExtractor::new(base_dir);
        let builder = CallGraphBuilder::new(query.direction, query.max_depth, &finder, &extractor);
        builder.build_trace(&start_fn)
    } else {
        Ok(None)
    }
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

/// Generate partial keys from a full translation key for common i18n patterns
///
/// For a key like "invoice.labels.add_new", this generates:
/// - "invoice.labels.add_new" (full key)
/// - "labels.add_new" (without first segment - namespace pattern)
/// - "invoice.labels" (without last segment - parent namespace pattern)
pub fn generate_partial_keys(full_key: &str) -> Vec<String> {
    let mut keys = Vec::new();

    // Always include the full key
    keys.push(full_key.to_string());

    let segments: Vec<&str> = full_key.split('.').collect();

    // Only generate partial keys if we have at least 2 segments
    if segments.len() >= 2 {
        // Generate key without first segment (e.g., "labels.add_new" from "invoice.labels.add_new")
        // This matches patterns like: ns = I18n.t('invoice.labels'); ns.t('add_new')
        if segments.len() > 1 {
            let without_first = segments[1..].join(".");
            keys.push(without_first);
        }

        // Generate key without last segment (e.g., "invoice.labels" from "invoice.labels.add_new")
        // This matches patterns like: labels = I18n.t('invoice.labels'); labels.t('add_new')
        if segments.len() > 1 {
            let without_last = segments[..segments.len() - 1].join(".");
            keys.push(without_last);
        }
    }

    keys
}
