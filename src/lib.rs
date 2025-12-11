pub mod cache;
pub mod config;
pub mod error;
pub mod output;
pub mod parse;
pub mod search;
pub mod trace;
pub mod tree;

use std::path::PathBuf;

// Re-export commonly used types
pub use cache::SearchResultCache;
pub use config::default_patterns;
pub use error::{Result, SearchError};
pub use output::TreeFormatter;
pub use parse::{KeyExtractor, TranslationEntry, YamlParser};
pub use search::{CodeReference, FileMatch, FileSearcher, Match, PatternMatcher, TextSearcher};
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
    pub exclude_patterns: Vec<String>,
}

impl TraceQuery {
    pub fn new(function_name: String, direction: TraceDirection, max_depth: usize) -> Self {
        Self {
            function_name,
            direction,
            max_depth,
            base_dir: None,
            exclude_patterns: Vec::new(),
        }
    }

    pub fn with_base_dir(mut self, base_dir: PathBuf) -> Self {
        self.base_dir = Some(base_dir);
        self
    }

    pub fn with_exclusions(mut self, exclusions: Vec<String>) -> Self {
        self.exclude_patterns = exclusions;
        self
    }
}

/// Query parameters for searching
#[derive(Debug, Clone)]
pub struct SearchQuery {
    pub text: String,
    pub case_sensitive: bool,
    pub word_match: bool,
    pub is_regex: bool,
    pub base_dir: Option<PathBuf>,
    pub exclude_patterns: Vec<String>,
    pub include_patterns: Vec<String>,
    pub verbose: bool,
    pub quiet: bool, // Suppress progress indicators (for --simple mode)
}

impl SearchQuery {
    pub fn new(text: String) -> Self {
        Self {
            text,
            case_sensitive: true,
            word_match: false,
            is_regex: false,
            base_dir: None,
            exclude_patterns: Vec::new(),
            include_patterns: Vec::new(),
            verbose: false,
            quiet: false,
        }
    }

    pub fn with_word_match(mut self, word_match: bool) -> Self {
        self.word_match = word_match;
        self
    }

    pub fn with_regex(mut self, is_regex: bool) -> Self {
        self.is_regex = is_regex;
        self
    }

    pub fn with_includes(mut self, includes: Vec<String>) -> Self {
        self.include_patterns = includes;
        self
    }

    pub fn with_case_sensitive(mut self, case_sensitive: bool) -> Self {
        self.case_sensitive = case_sensitive;
        self
    }

    pub fn with_base_dir(mut self, base_dir: PathBuf) -> Self {
        self.base_dir = Some(base_dir);
        self
    }

    pub fn with_exclusions(mut self, exclusions: Vec<String>) -> Self {
        self.exclude_patterns = exclusions;
        self
    }

    pub fn with_verbose(mut self, verbose: bool) -> Self {
        self.verbose = verbose;
        self
    }

    pub fn with_quiet(mut self, quiet: bool) -> Self {
        self.quiet = quiet;
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
///
/// # Rust Book Reference
///
/// **Chapter 9.2: Recoverable Errors with Result**
/// https://doc.rust-lang.org/book/ch09-02-recoverable-errors-with-result.html
///
/// # Educational Notes - The `#[must_use]` Attribute
///
/// The `#[must_use]` attribute causes a compiler warning if the Result is ignored:
///
/// ```rust,ignore
/// run_search(query);  // WARNING: unused Result that must be used
/// ```
///
/// This prevents accidentally ignoring errors. You must either:
/// - Handle the error: `match run_search(query) { Ok(r) => ..., Err(e) => ... }`
/// - Propagate with `?`: `let result = run_search(query)?;`
/// - Explicitly ignore: `let _ = run_search(query);`
///
/// **Why this matters:**
/// - Rust doesn't have exceptions - errors must be explicitly handled
/// - Ignoring a Result means ignoring potential errors
/// - `#[must_use]` makes error handling explicit and intentional
#[must_use = "this function returns a Result that should be handled"]
pub fn run_search(query: SearchQuery) -> Result<SearchResult> {
    // Determine the base directory to search
    let raw_base_dir = query
        .base_dir
        .clone()
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));

    // Handle case where base_dir is a file vs directory
    let (search_dir, specific_file) = if raw_base_dir.is_file() {
        // If it's a file, search in its parent directory but only that specific file
        let parent_dir = raw_base_dir
            .parent()
            .map(|p| p.to_path_buf())
            .unwrap_or_else(|| PathBuf::from("."));
        (parent_dir, Some(raw_base_dir.clone()))
    } else {
        // If it's a directory, search the whole directory
        (raw_base_dir.clone(), None)
    };

    // Use the search directory for project type detection
    let project_type = config::detect_project_type(&search_dir);
    let mut exclusions: Vec<String> = config::get_default_exclusions(project_type)
        .iter()
        .map(|&s| s.to_string())
        .collect();
    exclusions.extend(query.exclude_patterns.clone());

    // Step 1: Extract translation entries matching the search text
    // Only search for translation entries if we're not searching a specific file
    let translation_entries = if specific_file.is_none() {
        let mut extractor = KeyExtractor::new();
        extractor.set_exclusions(exclusions.clone());
        extractor.set_verbose(query.verbose);
        extractor.set_quiet(query.quiet);
        extractor.set_case_sensitive(query.case_sensitive);
        extractor.extract(&search_dir, &query.text)?
    } else {
        Vec::new() // Skip translation search for specific files
    };

    // Step 2: Find code references for each translation entry
    // Search for full key AND partial keys (for namespace caching patterns)
    let mut all_code_refs = Vec::new();

    if specific_file.is_none() {
        let mut matcher = PatternMatcher::new(search_dir.clone());
        matcher.set_exclusions(exclusions.clone());

        for entry in &translation_entries {
            // Generate all key variations (full key + partial keys)
            let key_variations = generate_partial_keys(&entry.key);

            // Search for each key variation
            for key in &key_variations {
                let code_refs = matcher.find_usages(key)?;
                all_code_refs.extend(code_refs);
            }
        }
    }

    // Step 3: Perform direct text search for the query text
    // This ensures we find hardcoded text even if no translation keys are found
    let text_searcher = TextSearcher::new(search_dir.clone())
        .case_sensitive(query.case_sensitive)
        .word_match(query.word_match)
        .is_regex(query.is_regex)
        .add_globs(query.include_patterns.clone())
        .add_exclusions(exclusions.clone())
        .respect_gitignore(true); // Always respect gitignore for now

    if let Ok(direct_matches) = text_searcher.search(&query.text) {
        for m in direct_matches {
            // If searching a specific file, only include matches from that file
            if let Some(ref target_file) = specific_file {
                if m.file != *target_file {
                    continue;
                }
            }

            // Filter out matches that are in translation files (already handled)
            // But only if we're not searching a specific file
            let path_str = m.file.to_string_lossy();
            if specific_file.is_none()
                && (path_str.ends_with(".yml")
                    || path_str.ends_with(".yaml")
                    || path_str.ends_with(".json")
                    || path_str.ends_with(".js"))
            {
                continue;
            }

            // Apply exclusions
            if exclusions.iter().any(|ex| path_str.contains(ex)) {
                continue;
            }

            // Convert Match to CodeReference
            all_code_refs.push(CodeReference {
                file: m.file.clone(),
                line: m.line,
                pattern: "Direct Match".to_string(),
                context: m.content.clone(),
                key_path: query.text.clone(), // Use the search text as the "key"
                context_before: m.context_before.clone(),
                context_after: m.context_after.clone(),
            });
        }
    }

    // Deduplicate code references (in case same reference matches multiple key variations)
    // We prioritize "traced" matches (where key_path != query) over "direct" matches (where key_path == query)
    // This ensures that if we have both for the same line, we keep the one that links to a translation key.
    all_code_refs.sort_by(|a, b| {
        a.file.cmp(&b.file).then(a.line.cmp(&b.line)).then_with(|| {
            let a_is_direct = a.key_path == query.text;
            let b_is_direct = b.key_path == query.text;
            // We want traced (false) to come before direct (true) so it is kept by dedup
            a_is_direct.cmp(&b_is_direct)
        })
    });
    all_code_refs.dedup_by(|a, b| a.file == b.file && a.line == b.line);

    Ok(SearchResult {
        query: query.text,
        translation_entries,
        code_references: all_code_refs,
    })
}

/// Orchestrates the call graph tracing process
///
/// This function:
/// 1. Finds the starting function definition
/// 2. Extracts function calls or callers based on the direction
/// 3. Builds a call graph tree up to the specified depth
///
/// # Arguments
/// * `query` - Configuration for the trace operation
///
/// # Returns
/// A `CallTree` representing the call graph, or `None` if the start function is not found.
#[must_use = "this function returns a Result that should be handled"]
pub fn run_trace(query: TraceQuery) -> Result<Option<CallTree>> {
    let base_dir = query
        .base_dir
        .clone()
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));

    let mut finder = FunctionFinder::new(base_dir.clone());
    if let Some(start_fn) = finder.find_function(&query.function_name) {
        let extractor = CallExtractor::new(base_dir);
        let mut builder =
            CallGraphBuilder::new(query.direction, query.max_depth, &mut finder, &extractor);
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
            path.ends_with(".yml") || path.ends_with(".yaml") || path.ends_with(".json")
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
        // Generate all suffixes with at least 2 segments
        // e.g. for "a.b.c.d":
        // - "b.c.d" (skip 1)
        // - "c.d"   (skip 2)
        for i in 1..segments.len() {
            if segments.len() - i >= 2 {
                keys.push(segments[i..].join("."));
            }
        }

        // Generate key without last segment (e.g., "invoice.labels" from "invoice.labels.add_new")
        // This matches patterns like: labels = I18n.t('invoice.labels'); labels.t('add_new')
        if segments.len() > 1 {
            let without_last = segments[..segments.len() - 1].join(".");
            // Avoid duplicates if without_last happens to be one of the suffixes (unlikely but possible)
            if !keys.contains(&without_last) {
                keys.push(without_last);
            }
        }
    }

    keys
}
