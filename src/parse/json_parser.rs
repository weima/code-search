use crate::error::{Result, SearchError};
use serde_json::Value;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use super::translation::TranslationEntry;

/// Parser for JSON translation files
pub struct JsonParser;

impl JsonParser {
    pub fn parse_file(path: &Path) -> Result<Vec<TranslationEntry>> {
        Self::parse_file_with_query(path, None)
    }

    /// Parse JSON file, optionally filtering by query for better performance.
    /// If query is provided, uses bottom-up approach: finds exact matches with grep,
    /// then traces keys upward WITHOUT parsing the entire JSON structure.
    pub fn parse_file_with_query(
        path: &Path,
        query: Option<&str>,
    ) -> Result<Vec<TranslationEntry>> {
        let content = fs::read_to_string(path).map_err(|e| {
            SearchError::json_parse_error(path, format!("Failed to read file: {}", e))
        })?;

        // Strip comments to support JSONC (JSON with Comments) format
        let cleaned_content = Self::strip_json_comments(&content);

        // If query is provided, use bottom-up approach
        if let Some(q) = query {
            return Self::parse_with_bottom_up_trace(path, &cleaned_content, q);
        }

        // No query - parse entire file
        let root: Value = serde_json::from_str(&cleaned_content).map_err(|e| {
            SearchError::json_parse_error(path, format!("Invalid JSON syntax: {}", e))
        })?;

        let mut entries = Vec::new();
        Self::flatten_json(&root, String::new(), path, &mut entries);

        Ok(entries)
    }

    /// Bottom-up approach: Find matching lines with grep, then trace keys upward.
    /// This avoids parsing the entire JSON structure.
    fn parse_with_bottom_up_trace(
        path: &Path,
        content: &str,
        query: &str,
    ) -> Result<Vec<TranslationEntry>> {
        use grep_regex::RegexMatcherBuilder;
        use grep_searcher::sinks::UTF8;
        use grep_searcher::SearcherBuilder;

        // Use grep to find exact line numbers with matches
        let matcher = RegexMatcherBuilder::new()
            .case_insensitive(true)
            .fixed_strings(true)
            .build(query)
            .map_err(|e| SearchError::json_parse_error(path, format!("Matcher error: {}", e)))?;

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
            .map_err(|e| SearchError::json_parse_error(path, format!("Search error: {}", e)))?;

        if matched_lines.is_empty() {
            return Ok(Vec::new());
        }

        // For each matched line, trace the key path bottom-up
        let lines: Vec<&str> = content.lines().collect();
        let mut entries = Vec::new();

        // Optimization: tree is non-tangled, later matches appear after earlier ones.
        let mut cutoff_line: usize = 0;
        let mut ancestor_cache: HashMap<usize, Vec<String>> = HashMap::new();

        for (line_num, _line_content) in matched_lines {
            if let Some(trace) =
                Self::trace_key_from_line(&lines, line_num, path, cutoff_line, &ancestor_cache)
            {
                for (line_idx, prefix) in trace.parent_prefixes {
                    ancestor_cache.entry(line_idx).or_insert(prefix);
                }

                entries.push(trace.entry);
            }

            cutoff_line = line_num;
        }

        Ok(entries)
    }

    /// Binary search for parent opening brace in JSON.
    /// Returns (line_index, key) if found.
    fn binary_search_parent_brace(
        lines: &[&str],
        end_line: usize,
        cutoff_line: usize,
        _ancestor_cache: &HashMap<usize, Vec<String>>,
    ) -> Option<(usize, String)> {
        let mut brace_depth = 0;

        // First, calculate the brace depth at end_line
        for i in ((end_line + 1)..lines.len()).take(1) {
            for ch in lines[i].chars() {
                match ch {
                    '{' => brace_depth += 1,
                    '}' => brace_depth -= 1,
                    _ => {}
                }
            }
        }

        // Count braces from end_line backwards to know target depth
        for i in (0..=end_line).rev() {
            let line = lines[i];
            for ch in line.chars() {
                match ch {
                    '}' => brace_depth += 1,
                    '{' => brace_depth -= 1,
                    _ => {}
                }
            }

            // Found opening brace that increases nesting
            if brace_depth > 0 && line.contains('{') {
                let line_idx = i + 1;
                if line_idx <= cutoff_line {
                    return None; // Hit cutoff boundary
                }

                // Find the key before this brace
                if let Some((key_line, key)) = Self::find_key_before_brace(&lines[..=i]) {
                    return Some((key_line - 1, key)); // Return 0-indexed
                }
                return None;
            }
        }

        None
    }

    /// Trace the JSON key path from a specific line number bottom-up.
    /// Uses binary search to find parent braces efficiently.
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
        // JSON format: "key": "value" or "key": value
        let colon_pos = target_line.find(':')?;
        let key_part = target_line[..colon_pos].trim().trim_matches('"');
        let value_part = target_line[colon_pos + 1..].trim();

        // Extract value, handling trailing commas
        let value = value_part
            .trim_end_matches(',')
            .trim()
            .trim_matches('"')
            .to_string();

        // Build the key path by walking up the JSON structure using binary search
        let mut key_parts = vec![key_part.to_string()];
        let mut parent_lines: Vec<usize> = Vec::new();
        let mut search_end = line_num - 1;

        // Find parents by binary searching for opening braces at decreasing depths
        while let Some((parent_idx, parent_key)) =
            Self::binary_search_parent_brace(lines, search_end, cutoff_line, ancestor_cache)
        {
            let line_idx = parent_idx + 1; // Convert to 1-based

            // Check if we hit cached ancestor
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

            key_parts.insert(0, parent_key);
            parent_lines.push(line_idx);
            search_end = parent_idx; // Next search ends at this parent

            if parent_idx == 0 {
                break; // Reached root
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

    /// Find the key name before an opening brace in JSON and return its line number (1-based)
    fn find_key_before_brace(lines: &[&str]) -> Option<(usize, String)> {
        // Walk backwards from the last line to find "key": {
        for (idx, line) in lines.iter().enumerate().rev() {
            let trimmed = line.trim();
            if let Some(colon_pos) = trimmed.find(':') {
                let key_part = trimmed[..colon_pos].trim().trim_matches('"');
                return Some((idx + 1, key_part.to_string()));
            }
        }
        None
    }

    /// Strip single-line (//) and multi-line (/* */) comments from JSON
    /// This enables parsing of JSONC (JSON with Comments) files
    fn strip_json_comments(content: &str) -> String {
        let mut result = String::with_capacity(content.len());
        let mut chars = content.chars().peekable();
        let mut in_string = false;
        let mut escape_next = false;

        while let Some(ch) = chars.next() {
            if escape_next {
                result.push(ch);
                escape_next = false;
                continue;
            }

            if ch == '\\' && in_string {
                result.push(ch);
                escape_next = true;
                continue;
            }

            if ch == '"' {
                in_string = !in_string;
                result.push(ch);
                continue;
            }

            if !in_string && ch == '/' {
                if let Some(&next_ch) = chars.peek() {
                    if next_ch == '/' {
                        // Single-line comment - skip until newline
                        chars.next(); // consume second '/'
                        for c in chars.by_ref() {
                            if c == '\n' {
                                result.push('\n'); // preserve newline for line counting
                                break;
                            }
                        }
                        continue;
                    } else if next_ch == '*' {
                        // Multi-line comment - skip until */
                        chars.next(); // consume '*'
                        let mut prev = ' ';
                        for c in chars.by_ref() {
                            if prev == '*' && c == '/' {
                                break;
                            }
                            if c == '\n' {
                                result.push('\n'); // preserve newlines
                            }
                            prev = c;
                        }
                        continue;
                    }
                }
            }

            result.push(ch);
        }

        result
    }

    fn flatten_json(
        value: &Value,
        prefix: String,
        file_path: &Path,
        entries: &mut Vec<TranslationEntry>,
    ) {
        match value {
            Value::Object(map) => {
                for (key, val) in map {
                    let new_prefix = if prefix.is_empty() {
                        key.clone()
                    } else {
                        format!("{}.{}", prefix, key)
                    };

                    Self::flatten_json(val, new_prefix, file_path, entries);
                }
            }
            Value::String(s) => {
                entries.push(TranslationEntry {
                    key: prefix,
                    value: s.clone(),
                    line: 0, // Placeholder - serde_json doesn't provide line numbers
                    file: PathBuf::from(file_path),
                });
            }
            Value::Number(n) => {
                entries.push(TranslationEntry {
                    key: prefix,
                    value: n.to_string(),
                    line: 0,
                    file: PathBuf::from(file_path),
                });
            }
            Value::Bool(b) => {
                entries.push(TranslationEntry {
                    key: prefix,
                    value: b.to_string(),
                    line: 0,
                    file: PathBuf::from(file_path),
                });
            }
            Value::Array(arr) => {
                for (index, val) in arr.iter().enumerate() {
                    let new_prefix = if prefix.is_empty() {
                        index.to_string()
                    } else {
                        format!("{}.{}", prefix, index)
                    };
                    Self::flatten_json(val, new_prefix, file_path, entries);
                }
            }
            _ => {
                // Ignore nulls for now
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

        let mut parent_prefixes = Vec::new();
        for (idx, line_idx) in parent_lines.iter().rev().enumerate() {
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
    fn test_parse_simple_json() {
        let mut file = NamedTempFile::new().unwrap();
        write!(file, r#"{{"key": "value"}}"#).unwrap();

        let entries = JsonParser::parse_file(file.path()).unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].key, "key");
        assert_eq!(entries[0].value, "value");
    }

    #[test]
    fn test_parse_nested_json() {
        let mut file = NamedTempFile::new().unwrap();
        write!(file, r#"{{"parent": {{"child": "value"}}}}"#).unwrap();

        let entries = JsonParser::parse_file(file.path()).unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].key, "parent.child");
        assert_eq!(entries[0].value, "value");
    }

    #[test]
    fn test_parse_json_array() {
        let mut file = NamedTempFile::new().unwrap();
        write!(file, r#"{{"list": ["item1", "item2"]}}"#).unwrap();

        let entries = JsonParser::parse_file(file.path()).unwrap();
        assert_eq!(entries.len(), 2);

        // Check first item
        let item1 = entries.iter().find(|e| e.value == "item1").unwrap();
        assert_eq!(item1.key, "list.0");

        // Check second item
        let item2 = entries.iter().find(|e| e.value == "item2").unwrap();
        assert_eq!(item2.key, "list.1");
    }

    #[test]
    fn test_bottom_up_trace_json() {
        let mut file = NamedTempFile::new().unwrap();
        write!(
            file,
            r#"{{
  "user": {{
    "authentication": {{
      "login": "Log In",
      "logout": "Log Out"
    }}
  }}
}}"#
        )
        .unwrap();

        let entries = JsonParser::parse_file_with_query(file.path(), Some("Log In")).unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].value, "Log In");
        // Key should be traced bottom-up
        assert!(entries[0].key.contains("login"));
    }
}
