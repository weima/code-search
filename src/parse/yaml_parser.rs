use crate::error::{Result, SearchError};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use yaml_rust::{Yaml, YamlLoader};

use super::translation::TranslationEntry;

/// Parser for YAML translation files
pub struct YamlParser;

impl YamlParser {
    /// Fast pre-check: does this file contain the search query?
    /// Uses grep library for exact match before expensive YAML parsing.
    /// Returns true if the file contains the query (case-insensitive).
    pub fn contains_query(path: &Path, query: &str) -> Result<bool> {
        use grep_regex::RegexMatcherBuilder;
        use grep_searcher::sinks::UTF8;
        use grep_searcher::SearcherBuilder;

        // Build matcher for case-insensitive fixed-string search
        let matcher = RegexMatcherBuilder::new()
            .case_insensitive(true)
            .fixed_strings(true) // Treat as literal string, not regex
            .build(query)
            .map_err(|e| {
                SearchError::yaml_parse_error(path, format!("Failed to build matcher: {}", e))
            })?;

        // Use searcher to check if file contains the query
        let mut searcher = SearcherBuilder::new().build();
        let mut found = false;

        searcher
            .search_path(
                &matcher,
                path,
                UTF8(|_line_num, _line_content| {
                    found = true;
                    Ok(false) // Stop searching after first match
                }),
            )
            .map_err(|e| SearchError::yaml_parse_error(path, format!("Search failed: {}", e)))?;

        Ok(found)
    }

    pub fn parse_file(path: &Path) -> Result<Vec<TranslationEntry>> {
        Self::parse_file_with_query(path, None)
    }

    /// Parse YAML file, optionally filtering by query for better performance.
    /// If query is provided, uses bottom-up approach: finds exact matches with grep,
    /// then traces keys upward WITHOUT parsing the entire YAML structure.
    pub fn parse_file_with_query(
        path: &Path,
        query: Option<&str>,
    ) -> Result<Vec<TranslationEntry>> {
        let content = fs::read_to_string(path).map_err(|e| {
            SearchError::yaml_parse_error(path, format!("Failed to read file: {}", e))
        })?;

        // Strip ERB templates to support Rails-style YAML fixtures
        let cleaned_content = Self::strip_erb_templates(&content);

        // If query is provided, use bottom-up approach
        if let Some(q) = query {
            return Self::parse_with_bottom_up_trace(path, &cleaned_content, q);
        }

        // No query - parse entire file (fallback to old method)
        let mut value_to_line: HashMap<String, usize> = HashMap::new();
        for (line_num, line) in cleaned_content.lines().enumerate() {
            if let Some(colon_pos) = line.find(':') {
                let value = line[colon_pos + 1..].trim();
                if !value.is_empty() && !value.starts_with('#') {
                    let clean_value = value.trim_matches('"').trim_matches('\'');
                    if !clean_value.is_empty() {
                        value_to_line
                            .entry(clean_value.to_string())
                            .or_insert(line_num + 1);
                    }
                }
            }
        }

        let docs = YamlLoader::load_from_str(&cleaned_content).map_err(|e| {
            SearchError::yaml_parse_error(path, format!("Invalid YAML syntax: {}", e))
        })?;

        let mut entries = Vec::new();
        for doc in docs {
            Self::flatten_yaml(doc, String::new(), path, &value_to_line, &mut entries, true);
        }

        Ok(entries)
    }

    /// Bottom-up approach: Find matching lines with grep, then trace keys upward.
    /// This avoids parsing the entire YAML structure.
    fn parse_with_bottom_up_trace(
        path: &Path,
        content: &str,
        query: &str,
    ) -> Result<Vec<TranslationEntry>> {
        use grep_regex::RegexMatcherBuilder;
        use grep_searcher::sinks::UTF8;
        use grep_searcher::SearcherBuilder;
        use std::collections::HashMap;

        // Use grep to find exact line numbers with matches
        let matcher = RegexMatcherBuilder::new()
            .case_insensitive(true)
            .fixed_strings(true)
            .build(query)
            .map_err(|e| SearchError::yaml_parse_error(path, format!("Matcher error: {}", e)))?;

        let mut searcher = SearcherBuilder::new().line_number(true).build();
        let mut matched_lines: Vec<(usize, String)> = Vec::new();

        searcher
            .search_path(
                &matcher,
                path,
                UTF8(|line_num, line_content| {
                    matched_lines.push((line_num as usize, line_content.to_string()));
                    Ok(true) // Continue searching
                }),
            )
            .map_err(|e| SearchError::yaml_parse_error(path, format!("Search error: {}", e)))?;

        if matched_lines.is_empty() {
            return Ok(Vec::new());
        }

        // For each matched line, trace the key path bottom-up
        let lines: Vec<&str> = content.lines().collect();
        let mut entries = Vec::new();

        // Optimization: tree is non-tangled, later matches appear after earlier ones.
        // Maintain a cutoff and ancestor cache to stop climbing once we cross earlier paths.
        let mut cutoff_line: usize = 0;
        let mut ancestor_cache: HashMap<usize, Vec<String>> = HashMap::new();

        for (line_num, _line_content) in matched_lines {
            if let Some(trace) =
                Self::trace_key_from_line(&lines, line_num, path, cutoff_line, &ancestor_cache)
            {
                // Register ancestors for future lookups (so later matches can stop early)
                for (line_idx, prefix) in trace.parent_prefixes {
                    ancestor_cache.entry(line_idx).or_insert(prefix);
                }

                entries.push(trace.entry);
            }

            // Monotonic guarantee: subsequent matches start after the previous leaf
            cutoff_line = line_num;
        }

        Ok(entries)
    }

    /// Trace the YAML key path from a specific line number bottom-up.
    /// Uses indentation to walk up the tree without parsing the entire YAML structure.
    fn trace_key_from_line(
        lines: &[&str],
        line_num: usize,
        path: &Path,
        cutoff_line: usize,
        ancestor_cache: &HashMap<usize, Vec<String>>,
    ) -> Option<TraceResult> {
        if line_num == 0 || line_num > lines.len() {
            return None;
        }

        let target_line = lines[line_num - 1]; // Convert to 0-indexed

        // Extract the key and value from the target line
        let colon_pos = target_line.find(':')?;
        let key_part = target_line[..colon_pos].trim();
        let value_part = target_line[colon_pos + 1..].trim();

        // Check for malformed YAML: multiple colons without quotes
        // e.g., "key: value: invalid: yaml" should be rejected
        if value_part.contains(':') && !value_part.starts_with('"') && !value_part.starts_with('\'')
        {
            return None; // Skip malformed lines
        }

        let value = value_part.trim_matches('"').trim_matches('\'').to_string();

        // Skip empty values
        if value.is_empty() {
            return None;
        }

        // Get the indentation level of the target line
        let target_indent = target_line.len() - target_line.trim_start().len();

        // Build the key path by walking up the tree
        let mut key_parts = vec![key_part.to_string()];
        let mut current_indent = target_indent;
        let mut parent_lines: Vec<usize> = Vec::new();

        // Walk backwards through lines to find parent keys
        // Start from line before target and go up to line 0
        for i in (0..line_num - 1).rev() {
            let line = lines[i];

            // If we've crossed into an earlier region than the previous match, only continue
            // if we can attach to a known ancestor prefix; otherwise stop early.
            let line_idx = i + 1; // convert to 1-based to align with matched line numbers
            if line_idx <= cutoff_line {
                if let Some(prefix) = ancestor_cache.get(&line_idx) {
                    let mut combined = prefix.clone();
                    combined.extend(key_parts);
                    return Some(TraceResult::new(
                        combined,
                        value,
                        line_num,
                        path,
                        parent_lines,
                    ));
                }
                break;
            }

            // Skip empty lines and comments
            if line.trim().is_empty() || line.trim().starts_with('#') {
                continue;
            }

            let line_indent = line.len() - line.trim_start().len();

            // Found a parent (less indented)
            if line_indent < current_indent {
                if let Some(colon_pos) = line.find(':') {
                    let parent_key = line[..colon_pos].trim();

                    // Skip locale root keys (en, fr, de, etc.)
                    if line_indent == 0
                        && (parent_key == "en"
                            || parent_key == "fr"
                            || parent_key == "de"
                            || parent_key == "es"
                            || parent_key == "ja"
                            || parent_key == "zh")
                    {
                        break;
                    }

                    key_parts.insert(0, parent_key.to_string());
                    parent_lines.push(line_idx);
                    current_indent = line_indent;

                    if line_indent == 0 {
                        break; // Reached root
                    }
                }
            }
        }

        Some(TraceResult::new(
            key_parts,
            value,
            line_num,
            path,
            parent_lines,
        ))
    }
    /// Strip ERB templates (<%= ... %> and <% ... %>) from YAML
    /// This enables parsing of Rails fixture files
    fn strip_erb_templates(content: &str) -> String {
        let mut result = String::with_capacity(content.len());
        let mut chars = content.chars().peekable();

        while let Some(ch) = chars.next() {
            if ch == '<' {
                if let Some(&'%') = chars.peek() {
                    chars.next(); // consume '%'

                    // Check for <%= or <%
                    let _has_equals = if let Some(&'=') = chars.peek() {
                        chars.next(); // consume '='
                        true
                    } else {
                        false
                    };

                    // Skip until we find %>
                    let mut prev = ' ';
                    for c in chars.by_ref() {
                        if prev == '%' && c == '>' {
                            break;
                        }
                        if c == '\n' {
                            result.push('\n'); // preserve newlines
                        }
                        prev = c;
                    }

                    // Replace ERB tag with empty string (already skipped)
                    continue;
                }
            }

            result.push(ch);
        }

        result
    }

    fn flatten_yaml(
        yaml: Yaml,
        prefix: String,
        file_path: &Path,
        value_to_line: &HashMap<String, usize>,
        entries: &mut Vec<TranslationEntry>,
        is_root: bool,
    ) {
        match yaml {
            Yaml::Hash(hash) => {
                for (key, value) in hash {
                    if let Some(key_str) = key.as_str() {
                        // Check if this is a locale root BEFORE building prefix
                        let is_locale_root = is_root
                            && prefix.is_empty()
                            && (key_str == "en"
                                || key_str == "fr"
                                || key_str == "de"
                                || key_str == "es"
                                || key_str == "ja"
                                || key_str == "zh");

                        // For locale roots, skip the locale prefix entirely
                        let new_prefix = if is_locale_root {
                            String::new()
                        } else if prefix.is_empty() {
                            key_str.to_string()
                        } else {
                            format!("{}.{}", prefix, key_str)
                        };

                        // Only flatten once, not twice!
                        Self::flatten_yaml(
                            value,
                            new_prefix,
                            file_path,
                            value_to_line,
                            entries,
                            false,
                        );
                    }
                }
            }
            Yaml::String(value) => {
                let line = value_to_line.get(&value).copied().unwrap_or(0);

                entries.push(TranslationEntry {
                    key: prefix,
                    value,
                    line,
                    file: PathBuf::from(file_path),
                });
            }
            Yaml::Integer(value) => {
                let value_str = value.to_string();
                let line = value_to_line.get(&value_str).copied().unwrap_or(0);

                entries.push(TranslationEntry {
                    key: prefix,
                    value: value_str,
                    line,
                    file: PathBuf::from(file_path),
                });
            }
            Yaml::Boolean(value) => {
                let value_str = value.to_string();
                let line = value_to_line.get(&value_str).copied().unwrap_or(0);

                entries.push(TranslationEntry {
                    key: prefix,
                    value: value_str,
                    line,
                    file: PathBuf::from(file_path),
                });
            }
            Yaml::Array(arr) => {
                for (index, val) in arr.into_iter().enumerate() {
                    let new_prefix = if prefix.is_empty() {
                        index.to_string()
                    } else {
                        format!("{}.{}", prefix, index)
                    };
                    Self::flatten_yaml(val, new_prefix, file_path, value_to_line, entries, false);
                }
            }
            _ => {
                // Ignore other types for now
            }
        }
    }
}

/// Result of a trace with ancestor bookkeeping so future traces can short-circuit.
struct TraceResult {
    entry: TranslationEntry,
    parent_prefixes: Vec<(usize, Vec<String>)>,
}

impl TraceResult {
    fn new(
        key_parts: Vec<String>,
        value: String,
        line_num: usize,
        path: &Path,
        parent_lines: Vec<usize>,
    ) -> Self {
        let entry = TranslationEntry {
            key: key_parts.join("."),
            value,
            line: line_num,
            file: PathBuf::from(path),
        };

        // Build prefix cache for each ancestor line (root first) so later traces can stop early.
        let mut parent_prefixes = Vec::new();
        for (idx, line_idx) in parent_lines.iter().rev().enumerate() {
            // idx corresponds to prefix length in key_parts
            let prefix_len = idx + 1;
            if prefix_len <= key_parts.len() {
                parent_prefixes.push((*line_idx, key_parts[..prefix_len].to_vec()));
            }
        }

        Self {
            entry,
            parent_prefixes,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_parse_simple_yaml() {
        let mut file = NamedTempFile::new().unwrap();
        write!(file, "key: value").unwrap();

        let entries = YamlParser::parse_file(file.path()).unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].key, "key");
        assert_eq!(entries[0].value, "value");
        assert_eq!(entries[0].line, 1);
    }

    #[test]
    fn test_parse_nested_yaml() {
        let mut file = NamedTempFile::new().unwrap();
        write!(file, "parent:\n  child: value").unwrap();

        let entries = YamlParser::parse_file(file.path()).unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].key, "parent.child");
        assert_eq!(entries[0].value, "value");
        assert_eq!(entries[0].line, 2);
    }

    #[test]
    fn test_parse_multiple_keys() {
        let mut file = NamedTempFile::new().unwrap();
        write!(
            file,
            "
key1: value1
key2: value2
nested:
  key3: value3
"
        )
        .unwrap();

        let entries = YamlParser::parse_file(file.path()).unwrap();
        assert_eq!(entries.len(), 3);

        // Find entries by key
        let entry1 = entries.iter().find(|e| e.key == "key1").unwrap();
        assert_eq!(entry1.value, "value1");
        assert_eq!(entry1.line, 2);

        let entry2 = entries.iter().find(|e| e.key == "key2").unwrap();
        assert_eq!(entry2.value, "value2");
        assert_eq!(entry2.line, 3);

        let entry3 = entries.iter().find(|e| e.key == "nested.key3").unwrap();
        assert_eq!(entry3.value, "value3");
        assert_eq!(entry3.line, 5);
    }

    #[test]
    fn test_parse_yaml_array() {
        let mut file = NamedTempFile::new().unwrap();
        write!(file, "list:\n  - item1\n  - item2").unwrap();

        let entries = YamlParser::parse_file(file.path()).unwrap();
        assert_eq!(entries.len(), 2);

        let item1 = entries.iter().find(|e| e.value == "item1").unwrap();
        assert_eq!(item1.key, "list.0");

        let item2 = entries.iter().find(|e| e.value == "item2").unwrap();
        assert_eq!(item2.key, "list.1");
    }

    #[test]
    fn test_bottom_up_trace() {
        let mut file = NamedTempFile::new().unwrap();
        write!(
            file,
            "en:
  js:
    user:
      log_in: \"Log In\"
      sign_up: \"Sign Up\"
"
        )
        .unwrap();

        let entries = YamlParser::parse_file_with_query(file.path(), Some("Log In")).unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].key, "js.user.log_in");
        assert_eq!(entries[0].value, "Log In");
        assert_eq!(entries[0].line, 4);
    }
}
