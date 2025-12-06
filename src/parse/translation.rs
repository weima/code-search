use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Represents a single translation entry found in a file
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TranslationEntry {
    /// The full dot-notation key (e.g., "invoice.labels.add_new")
    pub key: String,
    /// The translation value (e.g., "Add New")
    pub value: String,
    /// The line number where the key is defined (1-indexed)
    pub line: usize,
    /// The file path where this entry was found
    pub file: PathBuf,
}
