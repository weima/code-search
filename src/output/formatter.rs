use crate::tree::{NodeType, ReferenceTree, TreeNode};

/// Formatter for rendering reference trees as text
pub struct TreeFormatter {
    max_width: usize,
}

impl TreeFormatter {
    /// Create a new TreeFormatter with default width (80 columns)
    pub fn new() -> Self {
        Self { max_width: 80 }
    }

    /// Create a TreeFormatter with custom width
    pub fn with_width(max_width: usize) -> Self {
        Self { max_width }
    }

    /// Format a reference tree as a string
    pub fn format(&self, tree: &ReferenceTree) -> String {
        let mut output = String::new();
        self.format_node(&tree.root, &mut output, "", true, true);
        output
    }

    /// Format a single node and its children
    fn format_node(
        &self,
        node: &TreeNode,
        output: &mut String,
        prefix: &str,
        is_last: bool,
        is_root: bool,
    ) {
        // Format the current node
        if !is_root {
            output.push_str(prefix);
            output.push_str(if is_last { "└─> " } else { "├─> " });
        }

        // Add node content
        let content = self.format_content(node);
        output.push_str(&content);

        // Add location if present
        if let Some(location) = &node.location {
            let location_str = format!(
                " ({}:{})",
                location.file.display(),
                location.line
            );
            output.push_str(&location_str);
        }

        output.push('\n');

        // Format children
        let child_count = node.children.len();
        for (i, child) in node.children.iter().enumerate() {
            let is_last_child = i == child_count - 1;
            let child_prefix = if is_root {
                String::new()
            } else {
                format!("{}{}   ", prefix, if is_last { " " } else { "│" })
            };

            self.format_node(child, output, &child_prefix, is_last_child, false);
        }
    }

    /// Format node content based on node type
    fn format_content(&self, node: &TreeNode) -> String {
        match node.node_type {
            NodeType::Root => {
                format!("'{}' (search query)", node.content)
            }
            NodeType::Translation => {
                // Truncate if too long
                self.truncate(&node.content, self.max_width - 20)
            }
            NodeType::KeyPath => {
                format!("Key: {}", node.content)
            }
            NodeType::CodeRef => {
                // Truncate code context
                let truncated = self.truncate(&node.content.trim(), self.max_width - 30);
                format!("Code: {}", truncated)
            }
        }
    }

    /// Truncate a string to fit within max length
    fn truncate(&self, s: &str, max_len: usize) -> String {
        if s.len() <= max_len {
            s.to_string()
        } else {
            format!("{}...", &s[..max_len.saturating_sub(3)])
        }
    }
}

impl Default for TreeFormatter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tree::{Location, TreeNode};
    use std::path::PathBuf;

    #[test]
    fn test_formatter_creation() {
        let formatter = TreeFormatter::new();
        assert_eq!(formatter.max_width, 80);
    }

    #[test]
    fn test_formatter_with_custom_width() {
        let formatter = TreeFormatter::with_width(120);
        assert_eq!(formatter.max_width, 120);
    }

    #[test]
    fn test_format_empty_tree() {
        let tree = ReferenceTree::with_search_text("test".to_string());
        let formatter = TreeFormatter::new();
        let output = formatter.format(&tree);

        assert!(output.contains("'test'"));
        assert!(output.contains("search query"));
    }

    #[test]
    fn test_format_tree_with_translation() {
        let mut root = TreeNode::new(NodeType::Root, "add new".to_string());
        let translation = TreeNode::with_location(
            NodeType::Translation,
            "invoice.labels.add_new: 'add new'".to_string(),
            Location::new(PathBuf::from("en.yml"), 4),
        );
        root.add_child(translation);

        let tree = ReferenceTree::new(root);
        let formatter = TreeFormatter::new();
        let output = formatter.format(&tree);

        assert!(output.contains("'add new'"));
        assert!(output.contains("invoice.labels.add_new"));
        assert!(output.contains("en.yml:4"));
        assert!(output.contains("└─>") || output.contains("├─>"));
    }

    #[test]
    fn test_format_complete_tree() {
        let mut root = TreeNode::new(NodeType::Root, "add new".to_string());

        let mut translation = TreeNode::with_location(
            NodeType::Translation,
            "invoice.labels.add_new: 'add new'".to_string(),
            Location::new(PathBuf::from("en.yml"), 4),
        );

        let mut key_path = TreeNode::new(
            NodeType::KeyPath,
            "invoice.labels.add_new".to_string(),
        );

        let code_ref = TreeNode::with_location(
            NodeType::CodeRef,
            "I18n.t('invoice.labels.add_new')".to_string(),
            Location::new(PathBuf::from("invoices.ts"), 14),
        );

        key_path.add_child(code_ref);
        translation.add_child(key_path);
        root.add_child(translation);

        let tree = ReferenceTree::new(root);
        let formatter = TreeFormatter::new();
        let output = formatter.format(&tree);

        // Verify all parts are present
        assert!(output.contains("'add new'"));
        assert!(output.contains("invoice.labels.add_new"));
        assert!(output.contains("Key:"));
        assert!(output.contains("Code:"));
        assert!(output.contains("I18n.t"));
        assert!(output.contains("en.yml:4"));
        assert!(output.contains("invoices.ts:14"));
    }

    #[test]
    fn test_format_multiple_children() {
        let mut root = TreeNode::new(NodeType::Root, "test".to_string());

        let child1 = TreeNode::with_location(
            NodeType::Translation,
            "key1: 'value1'".to_string(),
            Location::new(PathBuf::from("file1.yml"), 1),
        );

        let child2 = TreeNode::with_location(
            NodeType::Translation,
            "key2: 'value2'".to_string(),
            Location::new(PathBuf::from("file2.yml"), 2),
        );

        root.add_child(child1);
        root.add_child(child2);

        let tree = ReferenceTree::new(root);
        let formatter = TreeFormatter::new();
        let output = formatter.format(&tree);

        // Should have both children
        assert!(output.contains("key1"));
        assert!(output.contains("key2"));
        assert!(output.contains("file1.yml:1"));
        assert!(output.contains("file2.yml:2"));

        // Should have proper tree connectors
        assert!(output.contains("├─>"));
        assert!(output.contains("└─>"));
    }

    #[test]
    fn test_truncate_long_content() {
        let formatter = TreeFormatter::with_width(50);
        let long_string = "a".repeat(100);
        let truncated = formatter.truncate(&long_string, 20);

        assert!(truncated.len() <= 20);
        assert!(truncated.ends_with("..."));
    }

    #[test]
    fn test_truncate_short_content() {
        let formatter = TreeFormatter::new();
        let short_string = "short";
        let result = formatter.truncate(short_string, 20);

        assert_eq!(result, "short");
    }

    #[test]
    fn test_format_content_root() {
        let formatter = TreeFormatter::new();
        let node = TreeNode::new(NodeType::Root, "test query".to_string());
        let content = formatter.format_content(&node);

        assert!(content.contains("test query"));
        assert!(content.contains("search query"));
    }

    #[test]
    fn test_format_content_key_path() {
        let formatter = TreeFormatter::new();
        let node = TreeNode::new(NodeType::KeyPath, "invoice.labels.add_new".to_string());
        let content = formatter.format_content(&node);

        assert!(content.contains("Key:"));
        assert!(content.contains("invoice.labels.add_new"));
    }

    #[test]
    fn test_format_content_code_ref() {
        let formatter = TreeFormatter::new();
        let node = TreeNode::new(
            NodeType::CodeRef,
            "  I18n.t('invoice.labels.add_new')  ".to_string(),
        );
        let content = formatter.format_content(&node);

        assert!(content.contains("Code:"));
        assert!(content.contains("I18n.t"));
        // Should trim whitespace
        assert!(!content.starts_with("  "));
    }

    #[test]
    fn test_format_deep_nesting() {
        let mut root = TreeNode::new(NodeType::Root, "test".to_string());
        let mut level1 = TreeNode::new(NodeType::Translation, "level1".to_string());
        let mut level2 = TreeNode::new(NodeType::KeyPath, "level2".to_string());
        let level3 = TreeNode::new(NodeType::CodeRef, "level3".to_string());

        level2.add_child(level3);
        level1.add_child(level2);
        root.add_child(level1);

        let tree = ReferenceTree::new(root);
        let formatter = TreeFormatter::new();
        let output = formatter.format(&tree);

        // Should have proper indentation
        let lines: Vec<&str> = output.lines().collect();
        assert!(lines.len() >= 4);

        // Check that deeper levels have more indentation
        assert!(lines[2].starts_with(' ') || lines[2].starts_with('│'));
    }
}
