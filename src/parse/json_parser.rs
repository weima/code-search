use crate::error::{Result, SearchError};
use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};

use super::translation::TranslationEntry;

/// Parser for JSON translation files
pub struct JsonParser;

impl JsonParser {
    pub fn parse_file(path: &Path) -> Result<Vec<TranslationEntry>> {
        let content = fs::read_to_string(path).map_err(|e| {
            SearchError::yaml_parse_error(path, format!("Failed to read file: {}", e))
        })?;

        let root: Value = serde_json::from_str(&content).map_err(|e| {
            SearchError::yaml_parse_error(path, format!("Invalid JSON syntax: {}", e))
        })?;

        let mut entries = Vec::new();
        Self::flatten_json(&root, String::new(), path, &mut entries);

        Ok(entries)
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
                // Note: JSON doesn't easily give us line numbers with standard serde_json
                // For now, we'll use 0 as the line number or try to find it if we want to be fancy later
                // To keep it simple and consistent with the requirement, we'll just store the entry.
                // If line numbers are critical, we might need a different parser or a second pass.
                // Given the constraints, 0 is acceptable for MVP JSON support if line numbers are hard.
                // However, let's try to be better. We can't easily get line numbers from serde_json::Value.
                // We will accept 0 for now as a limitation of serde_json default deserialization.
                entries.push(TranslationEntry {
                    key: prefix,
                    value: s.clone(),
                    line: 0, // Placeholder
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
            _ => {
                // Ignore arrays and nulls for now
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
}
