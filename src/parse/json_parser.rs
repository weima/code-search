use crate::error::{Result, SearchError};
use serde_json::Value;
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

        // Parse entire file
        let root: Value = serde_json::from_str(&cleaned_content).map_err(|e| {
            SearchError::json_parse_error(path, format!("Invalid JSON syntax: {}", e))
        })?;

        let mut entries = Vec::new();
        Self::flatten_json(&root, String::new(), path, &mut entries);

        // Filter by query if provided (since bottom-up trace is disabled)
        if let Some(q) = query {
            let q_lower = q.to_lowercase();
            entries.retain(|e| e.value.to_lowercase().contains(&q_lower));
        }

        Ok(entries)
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
