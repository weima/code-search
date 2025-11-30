// src/parse/key_extractor.rs

use crate::error::Result;
use std::path::Path;
use walkdir::WalkDir;

use super::json_parser::JsonParser;
use super::translation::TranslationEntry;
use super::yaml_parser::YamlParser;

/// `KeyExtractor` provides functionality to search translation entries across
/// multiple YAML translation files, returning the full dot‑notation key path,
/// associated file path and line number for each match.
pub struct KeyExtractor;

impl KeyExtractor {
    /// Create a new `KeyExtractor`.
    pub fn new() -> Self {
        Self
    }

    /// Recursively walk `base_dir` for `*.yml` (or `*.yaml`) files, parse each,
    /// and return entries whose **value** contains `query`.
    ///
    /// Matching is case‑insensitive by default.
    pub fn extract(&self, base_dir: &Path, query: &str) -> Result<Vec<TranslationEntry>> {
        let mut matches = Vec::new();
        let lowered = query.to_lowercase();

        for entry in WalkDir::new(base_dir)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
        {
            let path = entry.path();
            if let Some(ext) = path.extension() {
                let ext_str = ext.to_string_lossy();
                if ext_str == "yml" || ext_str == "yaml" {
                    let entries = YamlParser::parse_file(path)?;
                    for e in entries {
                        if e.value.to_lowercase().contains(&lowered) {
                            matches.push(e);
                        }
                    }
                } else if ext_str == "json" {
                    let entries = JsonParser::parse_file(path)?;
                    for e in entries {
                        if e.value.to_lowercase().contains(&lowered) {
                            matches.push(e);
                        }
                    }
                }
            }
        }
        Ok(matches)
    }
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
    fn test_key_extractor_supports_json_and_yaml() -> Result<()> {
        let dir = tempdir()?;
        let yaml_path = dir.path().join("test.yml");
        let txt_path = dir.path().join("test.txt");
        let json_path = dir.path().join("test.json");

        fs::write(&yaml_path, "key: \"test value\"")?;
        fs::write(&txt_path, "key: test value")?; // This should be ignored
        fs::write(&json_path, "{\"key\": \"test value\"}")?; // This should be ignored

        let extractor = KeyExtractor::new();
        let results = extractor.extract(dir.path(), "test")?;

        // Should find both YAML and JSON files
        assert_eq!(results.len(), 2);
        let extensions: Vec<_> = results
            .iter()
            .map(|e| e.file.extension().unwrap().to_string_lossy().to_string())
            .collect();
        assert!(extensions.contains(&"yml".to_string()));
        assert!(extensions.contains(&"json".to_string()));

        Ok(())
    }
}
