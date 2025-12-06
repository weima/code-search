use crate::trace::{CallNode, CallTree, TraceDirection};
use crate::SearchResult;

/// Formatter for simple, machine-readable output (ripgrep-compatible)
pub struct SimpleFormatter;

impl SimpleFormatter {
    pub fn new() -> Self {
        Self
    }

    /// Format a search result in simple format: file:line:content
    pub fn format(&self, result: &SearchResult) -> String {
        let mut output = String::new();

        // Format translation entries
        for entry in &result.translation_entries {
            let line = format!(
                "{}:{}:{}: '{}'",
                entry.file.display(),
                entry.line,
                entry.key,
                entry.value
            );
            output.push_str(&line);
            output.push('\n');
        }

        // Format code references
        for code_ref in &result.code_references {
            let line = format!(
                "{}:{}:{}",
                code_ref.file.display(),
                code_ref.line,
                code_ref.context.trim()
            );
            output.push_str(&line);
            output.push('\n');
        }

        output
    }

    /// Format a trace tree in simple format
    pub fn format_trace_tree(&self, tree: &CallTree, _direction: TraceDirection) -> String {
        let mut output = String::new();
        Self::format_trace_node(&tree.root, &mut output, 0);
        output
    }

    /// Recursively format trace nodes
    fn format_trace_node(node: &CallNode, output: &mut String, depth: usize) {
        // Format current node
        let indent = "  ".repeat(depth);
        let line = format!(
            "{}{}:{}:{}",
            indent,
            node.def.file.display(),
            node.def.line,
            node.def.name
        );
        output.push_str(&line);
        output.push('\n');

        // Format children
        for child in &node.children {
            Self::format_trace_node(child, output, depth + 1);
        }
    }
}

impl Default for SimpleFormatter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse::TranslationEntry;
    use crate::CodeReference;
    use std::path::PathBuf;

    #[test]
    fn test_format_translation_entry() {
        let formatter = SimpleFormatter::new();
        let mut result = SearchResult {
            query: String::new(),
            translation_entries: Vec::new(),
            code_references: Vec::new(),
        };

        result.translation_entries.push(TranslationEntry {
            key: "app.title".to_string(),
            value: "My App".to_string(),
            file: PathBuf::from("locales/en.yml"),
            line: 5,
        });

        let output = formatter.format(&result);
        assert_eq!(output, "locales/en.yml:5:app.title: 'My App'\n");
    }

    #[test]
    fn test_format_code_reference() {
        let formatter = SimpleFormatter::new();
        let mut result = SearchResult {
            query: String::new(),
            translation_entries: Vec::new(),
            code_references: Vec::new(),
        };

        result.code_references.push(CodeReference {
            file: PathBuf::from("src/main.rs"),
            line: 42,
            context: "    println!(\"Hello, world!\");".to_string(),
            key_path: "Hello".to_string(),
            pattern: "Hello".to_string(),
        });

        let output = formatter.format(&result);
        assert_eq!(output, "src/main.rs:42:println!(\"Hello, world!\");\n");
    }

    #[test]
    fn test_format_multiple_results() {
        let formatter = SimpleFormatter::new();
        let mut result = SearchResult {
            query: String::new(),
            translation_entries: Vec::new(),
            code_references: Vec::new(),
        };

        result.translation_entries.push(TranslationEntry {
            key: "key1".to_string(),
            value: "value1".to_string(),
            file: PathBuf::from("file1.yml"),
            line: 1,
        });

        result.code_references.push(CodeReference {
            file: PathBuf::from("file2.rs"),
            line: 2,
            context: "content2".to_string(),
            key_path: "key".to_string(),
            pattern: "key".to_string(),
        });

        let output = formatter.format(&result);
        assert!(output.contains("file1.yml:1:key1: 'value1'"));
        assert!(output.contains("file2.rs:2:content2"));
    }
}
