use anyhow::{Context, Result};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use yaml_rust::scanner::{Scanner, TokenType, TScalarStyle};
use yaml_rust::Yaml;

use super::translation::TranslationEntry;

/// Parser for YAML translation files
pub struct YamlParser;

impl YamlParser {
    pub fn parse_file(path: &Path) -> Result<Vec<TranslationEntry>> {
        let content = fs::read_to_string(path)
            .with_context(|| format!("Failed to read file: {}", path.display()))?;

        // First, build a map of scalar values to their line numbers using Scanner
        let mut value_to_line: HashMap<String, usize> = HashMap::new();
        let mut scanner = Scanner::new(content.chars());
        
        loop {
            match scanner.next_token() {
                Ok(Some(token)) => {
                    if let TokenType::Scalar(_, value) = token.1 {
                        // Store the line number for this scalar value
                        value_to_line.insert(value, token.0.line());
                    }
                }
                Ok(None) => break, // End of tokens
                Err(_) => break,   // Error, stop scanning
            }
        }

        // Then, use YamlLoader to parse the structure
        let docs = yaml_rust::YamlLoader::load_from_str(&content)
            .with_context(|| format!("Failed to parse YAML file: {}", path.display()))?;

        let mut entries = Vec::new();

        for doc in docs {
            Self::flatten_yaml(doc, String::new(), path, &value_to_line, &mut entries);
        }

        Ok(entries)
    }

    fn flatten_yaml(
        yaml: Yaml,
        prefix: String,
        file_path: &Path,
        value_to_line: &HashMap<String, usize>,
        entries: &mut Vec<TranslationEntry>,
    ) {
        match yaml {
            Yaml::Hash(hash) => {
                for (key, value) in hash {
                    if let Some(key_str) = key.as_str() {
                        let new_prefix = if prefix.is_empty() {
                            key_str.to_string()
                        } else {
                            format!("{}.{}", prefix, key_str)
                        };

                        Self::flatten_yaml(value, new_prefix, file_path, value_to_line, entries);
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
            _ => {
                // Ignore arrays and other types for now
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
        write!(file, "
key1: value1
key2: value2
nested:
  key3: value3
").unwrap();
        
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
}
