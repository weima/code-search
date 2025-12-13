// src/parse/key_extractor.rs

use crate::cache::SearchResultCache;
use crate::error::Result;
use std::path::Path;
use walkdir::WalkDir;

use super::js_parser::JsParser;
use super::json_parser::JsonParser;
use super::translation::TranslationEntry;
use super::yaml_parser::YamlParser;

/// `KeyExtractor` provides functionality to search translation entries across
/// multiple YAML translation files, returning the full dot‑notation key path,
/// associated file path and line number for each match.
pub struct KeyExtractor {
    exclusions: Vec<String>,
    verbose: bool,
    quiet: bool,          // Suppress progress indicators (for --simple mode)
    case_sensitive: bool, // Case-sensitive matching
    cache: Option<SearchResultCache>,
    progress_count: std::cell::Cell<usize>, // Track progress for better indicator
}

impl Default for KeyExtractor {
    fn default() -> Self {
        Self::new()
    }
}

impl KeyExtractor {
    /// Create a new `KeyExtractor`.
    pub fn new() -> Self {
        let cache = SearchResultCache::new().ok(); // Silently disable cache on error
        Self {
            exclusions: Vec::new(),
            verbose: false,
            quiet: false,
            case_sensitive: false,
            cache,
            progress_count: std::cell::Cell::new(0),
        }
    }

    /// Set exclusion patterns (e.g., directories or files to ignore)
    pub fn set_exclusions(&mut self, exclusions: Vec<String>) {
        self.exclusions = exclusions;
    }

    /// Set verbose mode for detailed error messages
    pub fn set_verbose(&mut self, verbose: bool) {
        self.verbose = verbose;
    }

    /// Set quiet mode to suppress progress indicators
    pub fn set_quiet(&mut self, quiet: bool) {
        self.quiet = quiet;
    }

    /// Set case-sensitive matching
    pub fn set_case_sensitive(&mut self, case_sensitive: bool) {
        self.case_sensitive = case_sensitive;
    }

    /// Print progress indicator with proper formatting
    /// Only shows meaningful progress - no useless dashes
    fn print_progress(&self, indicator_type: char) {
        if self.quiet {
            return;
        }

        let count = self.progress_count.get();

        // Only show meaningful progress indicators
        match indicator_type {
            '-' => {
                // Don't show skipped files at all - they're just noise
                return;
            }
            'C' => {
                // Show cache hits - indicates good performance
            }
            '.' => {
                // Show successful parses - indicates progress
            }
            'S' => {
                // Show parse errors - important for debugging
            }
            _ => return,
        }

        // Print the colored indicator
        use colored::Colorize;
        let indicator = match indicator_type {
            'C' => "C".cyan(),
            '.' => ".".green(),
            'S' => "S".yellow(),
            _ => return,
        };
        eprint!("{}", indicator);

        // Update count and add newline + reset every 30 characters
        let new_count = count + 1;
        if new_count >= 30 {
            eprintln!(); // Newline after 30 characters
            self.progress_count.set(0);
        } else {
            self.progress_count.set(new_count);
        }
    }

    /// Recursively walk `base_dir` for `*.yml` (or `*.yaml`) files, parse each,
    /// and return entries whose **value** contains `query`.
    ///
    /// Matching respects case sensitivity setting.
    pub fn extract(&self, base_dir: &Path, query: &str) -> Result<Vec<TranslationEntry>> {
        let mut matches = Vec::new();
        let search_query = if self.case_sensitive {
            query.to_string()
        } else {
            query.to_lowercase()
        };
        let mut skipped_files = 0;

        let walker = WalkDir::new(base_dir).into_iter();
        for entry in walker
            .filter_entry(|e| {
                if is_ignored(e) {
                    return false;
                }
                let name = e.file_name().to_string_lossy();
                for excl in &self.exclusions {
                    if name == excl.as_str() {
                        return false;
                    }
                }
                true
            })
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
        {
            let path = entry.path();
            if let Some(ext) = path.extension() {
                let ext_str = ext.to_string_lossy();

                if ext_str == "yml" || ext_str == "yaml" {
                    // OPTIMIZATION: Use ripgrep to pre-filter files before parsing
                    // This avoids expensive YAML parsing for files without matches
                    match YamlParser::contains_query(path, query) {
                        Ok(false) => {
                            // No match in file, skip it entirely
                            self.print_progress('-');
                            continue;
                        }
                        Err(_e) => {
                            // ripgrep failed, fall back to full parsing
                            // (don't skip the file, just proceed with parsing)
                        }
                        Ok(true) => {
                            // Match found, proceed with parsing below
                        }
                    }

                    // Try cache first
                    let metadata = std::fs::metadata(path).ok();
                    let cached_results = if let (Some(cache), Some(meta)) = (&self.cache, metadata)
                    {
                        let mtime = meta.modified().ok();
                        let size = meta.len();
                        if let Some(mt) = mtime {
                            cache.get(path, query, false, mt, size)
                        } else {
                            None
                        }
                    } else {
                        None
                    };

                    let all_entries = if let Some(cached) = cached_results {
                        if !self.quiet {
                            self.print_progress('C');
                        }
                        cached
                    } else {
                        // Cache miss - parse file with query for optimization
                        match YamlParser::parse_file_with_query(path, Some(query)) {
                            Ok(entries) => {
                                self.print_progress('.');

                                // Store in cache
                                if let (Some(cache), Ok(meta)) =
                                    (&self.cache, std::fs::metadata(path))
                                {
                                    if let (Ok(mtime), size) = (meta.modified(), meta.len()) {
                                        let _ =
                                            cache.set(path, query, false, mtime, size, &entries);
                                    }
                                }

                                entries
                            }
                            Err(e) => {
                                skipped_files += 1;
                                self.print_progress('S');
                                if self.verbose {
                                    eprintln!(
                                        "\nWarning: Failed to parse YAML file {}: {}",
                                        path.display(),
                                        e
                                    );
                                }
                                continue;
                            }
                        }
                    };

                    // Filter for matching entries
                    for e in all_entries {
                        let value_to_check = if self.case_sensitive {
                            e.value.clone()
                        } else {
                            e.value.to_lowercase()
                        };

                        if value_to_check.contains(&search_query) {
                            matches.push(e);
                        }
                    }
                } else if ext_str == "json" {
                    // OPTIMIZATION: Use ripgrep to pre-filter files before parsing
                    // Note: We don't have a contains_query for JSON yet, so we use YAML's
                    match YamlParser::contains_query(path, query) {
                        Ok(false) => {
                            // No match in file, skip it entirely
                            self.print_progress('-');
                            continue;
                        }
                        Err(_e) => {
                            // ripgrep failed, fall back to full parsing
                        }
                        Ok(true) => {
                            // Match found, proceed with parsing below
                        }
                    }

                    // Try cache first
                    let metadata = std::fs::metadata(path).ok();
                    let cached_results = if let (Some(cache), Some(meta)) = (&self.cache, metadata)
                    {
                        let mtime = meta.modified().ok();
                        let size = meta.len();
                        if let Some(mt) = mtime {
                            cache.get(path, query, false, mt, size)
                        } else {
                            None
                        }
                    } else {
                        None
                    };

                    let all_entries = if let Some(cached) = cached_results {
                        if !self.quiet {
                            self.print_progress('C');
                        }
                        cached
                    } else {
                        // Cache miss - parse file with query for optimization
                        match JsonParser::parse_file_with_query(path, Some(query)) {
                            Ok(entries) => {
                                self.print_progress('.');

                                // Store in cache
                                if let (Some(cache), Ok(meta)) =
                                    (&self.cache, std::fs::metadata(path))
                                {
                                    if let (Ok(mtime), size) = (meta.modified(), meta.len()) {
                                        let _ =
                                            cache.set(path, query, false, mtime, size, &entries);
                                    }
                                }

                                entries
                            }
                            Err(e) => {
                                skipped_files += 1;
                                self.print_progress('S');
                                if self.verbose {
                                    eprintln!(
                                        "\nWarning: Failed to parse JSON file {}: {}",
                                        path.display(),
                                        e
                                    );
                                }
                                continue;
                            }
                        }
                    };

                    // Filter for matching entries
                    for e in all_entries {
                        let value_to_check = if self.case_sensitive {
                            e.value.clone()
                        } else {
                            e.value.to_lowercase()
                        };

                        if value_to_check.contains(&search_query) {
                            matches.push(e);
                        }
                    }
                } else if ext_str == "js" {
                    // OPTIMIZATION: Use ripgrep to pre-filter files before parsing
                    match JsParser::contains_query(path, query) {
                        Ok(false) => {
                            // No match in file, skip it entirely
                            self.print_progress('-');
                            continue;
                        }
                        Err(_e) => {
                            // ripgrep failed, fall back to full parsing
                        }
                        Ok(true) => {
                            // Match found, proceed with parsing below
                        }
                    }

                    // Try cache first
                    let metadata = std::fs::metadata(path).ok();
                    let cached_results = if let (Some(cache), Some(meta)) = (&self.cache, metadata)
                    {
                        let mtime = meta.modified().ok();
                        let size = meta.len();
                        if let Some(mt) = mtime {
                            cache.get(path, query, false, mt, size)
                        } else {
                            None
                        }
                    } else {
                        None
                    };

                    let all_entries = if let Some(cached) = cached_results {
                        if !self.quiet {
                            self.print_progress('C');
                        }
                        cached
                    } else {
                        // Cache miss - parse file with query for optimization
                        match JsParser::parse_file_with_query(path, Some(query)) {
                            Ok(entries) => {
                                self.print_progress('.');

                                // Store in cache
                                if let (Some(cache), Ok(meta)) =
                                    (&self.cache, std::fs::metadata(path))
                                {
                                    if let (Ok(mtime), size) = (meta.modified(), meta.len()) {
                                        let _ =
                                            cache.set(path, query, false, mtime, size, &entries);
                                    }
                                }

                                entries
                            }
                            Err(e) => {
                                skipped_files += 1;
                                self.print_progress('S');
                                if self.verbose {
                                    eprintln!(
                                        "\nWarning: Failed to parse JavaScript file {}: {}",
                                        path.display(),
                                        e
                                    );
                                }
                                continue;
                            }
                        }
                    };

                    // Filter for matching entries
                    for e in all_entries {
                        let value_to_check = if self.case_sensitive {
                            e.value.clone()
                        } else {
                            e.value.to_lowercase()
                        };

                        if value_to_check.contains(&search_query) {
                            matches.push(e);
                        }
                    }
                }
            }
        }

        // Print final newline and summary if files were skipped (only in verbose mode)
        // Note: Skipped files are typically config files (package.json, tsconfig.json, etc.)
        // that aren't translation files, which is expected behavior.
        if !self.quiet {
            // Always print final newline if we showed any progress
            if self.progress_count.get() > 0 {
                eprintln!();
            }

            if skipped_files > 0 && self.verbose {
                eprintln!(
                    "(Skipped {} non-translation file{})",
                    skipped_files,
                    if skipped_files == 1 { "" } else { "s" }
                );
            }
        }

        Ok(matches)
    }
}

fn is_ignored(entry: &walkdir::DirEntry) -> bool {
    // Always allow the root directory of the search
    if entry.depth() == 0 {
        return false;
    }

    entry
        .file_name()
        .to_str()
        .map(|s| {
            s.starts_with('.') // Hidden files/dirs
                || s == "node_modules"
                || s == "target"
                || s == "dist"
                || s == "build"
                || s == "vendor"
        })
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    use tempfile::tempdir;

    #[test]
    fn test_key_extractor_simple() -> Result<()> {
        let dir = tempdir()?;
        let en_path = dir.path().join("en.yml");
        let fr_path = dir.path().join("fr.yml");

        // Write simple yaml files with proper format
        fs::write(
            &en_path,
            "greeting:\n  hello: \"Hello World\"\n  goodbye: \"Goodbye\"",
        )?;
        fs::write(
            &fr_path,
            "greeting:\n  hello: \"Bonjour World\"\n  goodbye: \"Au revoir\"",
        )?;

        let extractor = KeyExtractor::new();
        let results = extractor.extract(dir.path(), "world")?;

        // Should find two entries (en and fr)
        assert_eq!(results.len(), 2);
        let keys: Vec<_> = results.iter().map(|e| e.key.clone()).collect();
        assert!(keys.contains(&"greeting.hello".to_string()));
        Ok(())
    }

    #[test]
    fn test_key_extractor_case_insensitive() -> Result<()> {
        let dir = tempdir()?;
        let yaml_path = dir.path().join("test.yml");

        fs::write(
            &yaml_path,
            "app:\n  title: \"My Application\"\n  description: \"A great APP for everyone\"",
        )?;

        let extractor = KeyExtractor::new();

        // Test case insensitive search
        let results = extractor.extract(dir.path(), "APP")?;
        assert_eq!(results.len(), 2); // Should match both "Application" and "APP"

        let values: Vec<_> = results.iter().map(|e| e.value.clone()).collect();
        assert!(values.contains(&"My Application".to_string()));
        assert!(values.contains(&"A great APP for everyone".to_string()));

        Ok(())
    }

    #[test]
    fn test_key_extractor_multiple_files() -> Result<()> {
        let dir = tempdir()?;

        // Create multiple language files
        let en_path = dir.path().join("en.yml");
        let fr_path = dir.path().join("fr.yml");
        let de_path = dir.path().join("de.yml");

        fs::write(&en_path, "common:\n  action: \"Save Data\"")?;
        fs::write(&fr_path, "common:\n  action: \"Sauvegarder Data\"")?;
        fs::write(&de_path, "common:\n  action: \"Speichern Data\"")?;

        let extractor = KeyExtractor::new();
        let results = extractor.extract(dir.path(), "data")?;

        // Should find all three files (case-insensitive)
        assert_eq!(results.len(), 3);

        let files: Vec<_> = results
            .iter()
            .map(|e| e.file.file_name().unwrap().to_string_lossy().to_string())
            .collect();
        assert!(files.contains(&"en.yml".to_string()));
        assert!(files.contains(&"fr.yml".to_string()));
        assert!(files.contains(&"de.yml".to_string()));

        Ok(())
    }

    #[test]
    fn test_key_extractor_deep_nested() -> Result<()> {
        let dir = tempdir()?;
        let yaml_path = dir.path().join("nested.yml");

        fs::write(
            &yaml_path,
            "level1:\n  level2:\n    level3:\n      deep_key: \"Deep nested value\"\n      another: \"test value\"",
        )?;

        let extractor = KeyExtractor::new();
        let results = extractor.extract(dir.path(), "deep")?;

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].key, "level1.level2.level3.deep_key");
        assert_eq!(results[0].value, "Deep nested value");

        Ok(())
    }

    #[test]
    fn test_key_extractor_no_matches() -> Result<()> {
        let dir = tempdir()?;
        let yaml_path = dir.path().join("test.yml");

        fs::write(
            &yaml_path,
            "greeting:\n  hello: \"Hello\"\n  goodbye: \"Goodbye\"",
        )?;

        let extractor = KeyExtractor::new();
        let results = extractor.extract(dir.path(), "nonexistent")?;

        assert_eq!(results.len(), 0);

        Ok(())
    }

    #[test]
    fn test_key_extractor_supports_yaml_json_and_js() -> Result<()> {
        let dir = tempdir()?;
        let yaml_path = dir.path().join("test.yml");
        let txt_path = dir.path().join("test.txt");
        let json_path = dir.path().join("test.json");
        let js_path = dir.path().join("test.js");

        fs::write(&yaml_path, "key: \"test value\"")?;
        fs::write(&txt_path, "key: test value")?; // This should be ignored
        fs::write(&json_path, "{\"key\": \"test value\"}")?;
        fs::write(&js_path, "export default { key: 'test value' };")?;

        let extractor = KeyExtractor::new();
        let results = extractor.extract(dir.path(), "test")?;

        // Should find YAML, JSON, and JS files
        assert_eq!(results.len(), 3);
        let extensions: Vec<_> = results
            .iter()
            .map(|e| e.file.extension().unwrap().to_string_lossy().to_string())
            .collect();
        assert!(extensions.contains(&"yml".to_string()));
        assert!(extensions.contains(&"json".to_string()));
        assert!(extensions.contains(&"js".to_string()));

        Ok(())
    }

    #[test]
    fn test_key_extractor_malformed_file() -> Result<()> {
        let dir = tempdir()?;
        let good_path = dir.path().join("good.yml");
        let bad_path = dir.path().join("bad.yml");

        fs::write(&good_path, "key: \"value\"")?;
        fs::write(&bad_path, "key: value: invalid: yaml")?; // Malformed YAML

        let extractor = KeyExtractor::new();
        // This should NOT return an error, but just skip the bad file
        let results = extractor.extract(dir.path(), "value")?;

        // Should find the good file
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].value, "value");

        Ok(())
    }

    #[test]
    fn test_key_extractor_with_js_file() -> Result<()> {
        let dir = tempdir()?;
        let js_path = dir.path().join("en.js");

        fs::write(
            &js_path,
            r#"
export default {
  table: {
    emptyText: 'No Data',
    confirmFilter: 'Confirm'
  }
};
"#,
        )?;

        let extractor = KeyExtractor::new();
        let results = extractor.extract(dir.path(), "No Data")?;

        println!("Found {} translation entries:", results.len());
        for entry in &results {
            println!(
                "  {} = {} ({}:{})",
                entry.key,
                entry.value,
                entry.file.display(),
                entry.line
            );
        }

        assert!(
            !results.is_empty(),
            "Should find translation entries in JS file"
        );

        let no_data_entry = results.iter().find(|e| e.value == "No Data");
        assert!(no_data_entry.is_some(), "Should find 'No Data' entry");

        let entry = no_data_entry.unwrap();
        assert_eq!(entry.key, "table.emptyText");
        assert_eq!(entry.value, "No Data");

        Ok(())
    }
}
