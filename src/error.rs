use std::path::PathBuf;
use thiserror::Error;

/// Custom error type for code search operations
#[derive(Debug, Error)]
pub enum SearchError {
    /// No translation files found containing the search text
    #[error("No translation files found containing '{text}'.\n\nSearched in: {searched_paths}\n\nTip: Check your project structure or verify translation files exist")]
    NoTranslationFiles {
        text: String,
        searched_paths: String,
    },

    /// Failed to parse YAML file
    #[error("Failed to parse YAML file {file}:\n{reason}\n\nTip: Verify the YAML syntax is correct")]
    YamlParseError { file: PathBuf, reason: String },

    /// Translation key found but no code references detected
    #[error("Translation key '{key}' found in {file} but no code references detected.\n\nTip: Check if the key is actually used in the codebase")]
    NoCodeReferences { key: String, file: PathBuf },

    /// IO error occurred
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Failed to execute ripgrep command
    #[error("Failed to execute ripgrep command: {0}")]
    RipgrepExecutionFailed(String),

    /// Invalid UTF-8 in ripgrep output
    #[error("ripgrep output is not valid UTF-8: {0}")]
    InvalidUtf8(#[from] std::string::FromUtf8Error),

    /// Failed to parse file path
    #[error("Failed to parse file path: {0}")]
    InvalidPath(String),

    /// Generic search error with context
    #[error("{0}")]
    Generic(String),
}

impl SearchError {
    /// Create a NoTranslationFiles error with default searched paths
    pub fn no_translation_files(text: impl Into<String>) -> Self {
        Self::NoTranslationFiles {
            text: text.into(),
            searched_paths: "config/locales, src/i18n, locales, i18n".to_string(),
        }
    }

    /// Create a NoTranslationFiles error with custom searched paths
    pub fn no_translation_files_with_paths(
        text: impl Into<String>,
        paths: impl Into<String>,
    ) -> Self {
        Self::NoTranslationFiles {
            text: text.into(),
            searched_paths: paths.into(),
        }
    }

    /// Create a YamlParseError from a file path and error
    pub fn yaml_parse_error(file: impl Into<PathBuf>, reason: impl Into<String>) -> Self {
        Self::YamlParseError {
            file: file.into(),
            reason: reason.into(),
        }
    }

    /// Create a NoCodeReferences error
    pub fn no_code_references(key: impl Into<String>, file: impl Into<PathBuf>) -> Self {
        Self::NoCodeReferences {
            key: key.into(),
            file: file.into(),
        }
    }
}

/// Result type alias for SearchError
pub type Result<T> = std::result::Result<T, SearchError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_no_translation_files_error() {
        let err = SearchError::no_translation_files("add new");
        let msg = err.to_string();
        assert!(msg.contains("add new"));
        assert!(msg.contains("config/locales"));
        assert!(msg.contains("Tip:"));
    }

    #[test]
    fn test_no_translation_files_with_custom_paths() {
        let err = SearchError::no_translation_files_with_paths("test", "custom/path1, custom/path2");
        let msg = err.to_string();
        assert!(msg.contains("test"));
        assert!(msg.contains("custom/path1"));
        assert!(msg.contains("custom/path2"));
    }

    #[test]
    fn test_yaml_parse_error() {
        let err = SearchError::yaml_parse_error("config/en.yml", "unexpected character");
        let msg = err.to_string();
        assert!(msg.contains("config/en.yml"));
        assert!(msg.contains("unexpected character"));
        assert!(msg.contains("YAML syntax"));
    }

    #[test]
    fn test_no_code_references_error() {
        let err = SearchError::no_code_references("invoice.labels.add_new", "config/en.yml");
        let msg = err.to_string();
        assert!(msg.contains("invoice.labels.add_new"));
        assert!(msg.contains("config/en.yml"));
        assert!(msg.contains("Tip:"));
    }

    #[test]
    fn test_io_error_conversion() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let search_err: SearchError = io_err.into();
        let msg = search_err.to_string();
        assert!(msg.contains("IO error"));
        assert!(msg.contains("file not found"));
    }

    #[test]
    fn test_ripgrep_execution_failed() {
        let err = SearchError::RipgrepExecutionFailed("command failed".to_string());
        let msg = err.to_string();
        assert!(msg.contains("Failed to execute ripgrep"));
        assert!(msg.contains("command failed"));
    }
}
