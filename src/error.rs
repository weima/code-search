//! # Error Handling - Rust Book Chapter 9
//!
//! This module demonstrates custom error types and error handling patterns from
//! [The Rust Book Chapter 9](https://doc.rust-lang.org/book/ch09-00-error-handling.html).
//!
//! ## Key Concepts Demonstrated
//!
//! 1. **Custom Error Types with `thiserror`** (Chapter 9.2)
//!    - Using enums to represent different error conditions
//!    - Adding context to errors with struct variants
//!    - Automatic `Display` implementation via `#[error(...)]`
//!
//! 2. **Automatic Error Conversion with `#[from]`** (Chapter 9.2)
//!    - The `#[from]` attribute implements `From<OtherError>` automatically
//!    - Enables using `?` operator with different error types
//!
//! 3. **Type Alias for Results** (Chapter 9.2)
//!    - Using `type Result<T> = std::result::Result<T, SearchError>`
//!    - Makes function signatures cleaner
//!
//! 4. **Builder Methods with `impl Into<T>`** (Chapter 10.2)
//!    - Accepting flexible input types that can convert to the target type
//!    - Makes APIs more ergonomic
//!
//! ## Learning Notes
//!
//! The `thiserror` crate is industry standard for libraries because it:
//! - Derives `std::error::Error` automatically
//! - Generates helpful `Display` messages
//! - Integrates seamlessly with the `?` operator
//!
//! Compare this to the book's manual error implementations to see how
//! derive macros reduce boilerplate while maintaining full functionality.

use std::path::PathBuf;
use thiserror::Error;

/// Custom error type for code search operations.
///
/// # Rust Book Reference
///
/// **Chapter 9.2: Recoverable Errors with Result**
/// https://doc.rust-lang.org/book/ch09-02-recoverable-errors-with-result.html
///
/// This demonstrates defining custom error types using an enum, which allows
/// representing multiple kinds of errors with different associated data.
///
/// # Educational Notes
///
/// ## Why use an enum for errors?
/// - Each variant can carry different data (strings, paths, numbers)
/// - Pattern matching ensures all error cases are handled
/// - Type system prevents mixing up different error kinds
///
/// ## The `#[derive(Debug, Error)]` attributes
/// - `Debug`: Required by `std::error::Error` trait
/// - `Error`: From `thiserror`, auto-implements `std::error::Error`
///
/// ## The `#[error("...")]` attribute
/// - Automatically implements `Display` trait
/// - Use `{field}` to interpolate struct fields
/// - Creates user-friendly error messages
///
/// Compare this to the book's manual `Display` implementation in Chapter 9.2
/// to see how `thiserror` reduces boilerplate.
#[derive(Debug, Error)]
pub enum SearchError {
    /// No translation files found containing the search text
    #[error("No translation files found containing '{text}'.\n\nSearched in: {searched_paths}\n\nTip: Check your project structure or verify translation files exist")]
    NoTranslationFiles {
        text: String,
        searched_paths: String,
    },

    /// Failed to parse YAML file
    #[error(
        "Failed to parse YAML file {file}:\n{reason}\n\nTip: Verify the YAML syntax is correct"
    )]
    YamlParseError { file: PathBuf, reason: String },

    /// Failed to parse JSON file
    #[error(
        "Failed to parse JSON file {file}:\n{reason}\n\nTip: Verify the JSON syntax is correct"
    )]
    JsonParseError { file: PathBuf, reason: String },

    /// Translation key found but no code references detected
    #[error("Translation key '{key}' found in {file} but no code references detected.\n\nTip: Check if the key is actually used in the codebase")]
    NoCodeReferences { key: String, file: PathBuf },

    /// IO error occurred during file operations.
    ///
    /// # Rust Book Reference
    ///
    /// **Chapter 9.2: Propagating Errors**
    /// https://doc.rust-lang.org/book/ch09-02-recoverable-errors-with-result.html#propagating-errors
    ///
    /// # Educational Notes - The `#[from]` Attribute
    ///
    /// The `#[from]` attribute automatically implements:
    /// ```rust,ignore
    /// impl From<std::io::Error> for SearchError {
    ///     fn from(err: std::io::Error) -> Self {
    ///         SearchError::Io(err)
    ///     }
    /// }
    /// ```
    ///
    /// This enables the `?` operator to automatically convert IO errors:
    /// ```rust,ignore
    /// fn read_file(path: &Path) -> Result<String> {
    ///     let contents = std::fs::read_to_string(path)?;  // IO error auto-converts
    ///     Ok(contents)
    /// }
    /// ```
    ///
    /// Without `#[from]`, you would need manual conversion or `.map_err()`.
    ///
    /// **Key Point**: The `?` operator calls `From::from` automatically,
    /// making error propagation seamless across different error types.
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
    /// Create a NoTranslationFiles error with default searched paths.
    ///
    /// # Rust Book Reference
    ///
    /// **Chapter 10.2: Traits as Parameters**
    /// https://doc.rust-lang.org/book/ch10-02-traits.html#traits-as-parameters
    ///
    /// # Educational Notes - `impl Into<String>`
    ///
    /// Using `impl Into<String>` instead of `String` makes the API more flexible:
    ///
    /// ```rust,ignore
    /// // All of these work:
    /// SearchError::no_translation_files("add new");           // &str
    /// SearchError::no_translation_files(String::from("add new")); // String
    /// SearchError::no_translation_files(owned_string);        // String (moved)
    /// ```
    ///
    /// **How it works:**
    /// - `&str` implements `Into<String>` (converts by allocating)
    /// - `String` implements `Into<String>` (converts by identity/move)
    /// - The `.into()` call inside performs the conversion
    ///
    /// **Trade-off:**
    /// - Pro: Caller convenience - accepts multiple types
    /// - Pro: Follows Rust API guidelines
    /// - Con: Slightly less clear what conversions happen
    ///
    /// **Best Practice**: Use `impl Into<T>` for owned types in constructors/builders,
    /// use `&str` for borrowed parameters in regular methods.
    pub fn no_translation_files(text: impl Into<String>) -> Self {
        Self::NoTranslationFiles {
            text: text.into(),
            searched_paths: "config/locales, src/i18n, locales, i18n".to_string(),
        }
    }

    /// Create a NoTranslationFiles error with custom searched paths.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use code_search_cli::error::SearchError;
    ///
    /// let err = SearchError::no_translation_files_with_paths(
    ///     "Add New",
    ///     "src/locales, config/i18n"
    /// );
    /// ```
    ///
    /// Both parameters accept `&str` or `String` thanks to `impl Into<String>`.
    pub fn no_translation_files_with_paths(
        text: impl Into<String>,
        paths: impl Into<String>,
    ) -> Self {
        Self::NoTranslationFiles {
            text: text.into(),
            searched_paths: paths.into(),
        }
    }

    /// Create a YamlParseError from a file path and error.
    ///
    /// # Educational Notes - Multiple Generic Parameters
    ///
    /// This method shows using `impl Into<T>` with different types:
    /// - `impl Into<PathBuf>` accepts `&Path`, `PathBuf`, `&str`, `String`
    /// - `impl Into<String>` accepts `&str`, `String`
    ///
    /// Each parameter independently accepts its own set of convertible types.
    pub fn yaml_parse_error(file: impl Into<PathBuf>, reason: impl Into<String>) -> Self {
        Self::YamlParseError {
            file: file.into(),
            reason: reason.into(),
        }
    }

    /// Create a JsonParseError from a file path and error
    pub fn json_parse_error(file: impl Into<PathBuf>, reason: impl Into<String>) -> Self {
        Self::JsonParseError {
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
        let err =
            SearchError::no_translation_files_with_paths("test", "custom/path1, custom/path2");
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
