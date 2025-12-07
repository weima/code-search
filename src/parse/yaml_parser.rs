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

        // Parse entire file
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

        // Filter by query if provided (since bottom-up trace is disabled)
        if let Some(q) = query {
            let q_lower = q.to_lowercase();
            entries.retain(|e| e.value.to_lowercase().contains(&q_lower));
        }

        Ok(entries)
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
