use crate::tree::{Location, NodeType, ReferenceTree, TreeNode};
use crate::{CodeReference, SearchResult, TranslationEntry};

/// Builder for constructing reference trees from search results
pub struct ReferenceTreeBuilder;

impl ReferenceTreeBuilder {
    /// Build a reference tree from search results
    ///
    /// Creates a hierarchical tree structure:
    /// - Root: search query text
    ///   - Translation: translation file entry
    ///     - KeyPath: full translation key
    ///       - CodeRef: code reference using the key
    pub fn build(result: &SearchResult) -> ReferenceTree {
        let mut root = TreeNode::new(NodeType::Root, result.query.clone());

        // Group code references by translation entry key
        for entry in &result.translation_entries {
            let mut translation_node = Self::build_translation_node(entry);
            let mut key_node = Self::build_key_node(entry);

            // Find all code references for this translation key
            let matching_refs: Vec<_> = result
                .code_references
                .iter()
                .filter(|r| r.key_path == entry.key)
                .collect();

            // Add code reference nodes as children of the key node
            for code_ref in matching_refs {
                let code_node = Self::build_code_node(code_ref);
                key_node.add_child(code_node);
            }

            // Only add the key node if it has code references
            if key_node.has_children() {
                translation_node.children.push(key_node);
            }

            root.add_child(translation_node);
        }

        ReferenceTree::new(root)
    }

    /// Build a translation node from a translation entry
    fn build_translation_node(entry: &TranslationEntry) -> TreeNode {
        let content = format!("{}: '{}'", entry.key, entry.value);
        let location = Location::new(entry.file.clone(), entry.line);

        TreeNode::with_location(NodeType::Translation, content, location)
    }

    /// Build a key path node from a translation entry
    fn build_key_node(entry: &TranslationEntry) -> TreeNode {
        TreeNode::new(NodeType::KeyPath, entry.key.clone())
    }

    /// Build a code reference node
    fn build_code_node(code_ref: &CodeReference) -> TreeNode {
        let location = Location::new(code_ref.file.clone(), code_ref.line);
        TreeNode::with_location(NodeType::CodeRef, code_ref.context.clone(), location)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn create_test_translation_entry() -> TranslationEntry {
        TranslationEntry {
            key: "invoice.labels.add_new".to_string(),
            value: "add new".to_string(),
            line: 4,
            file: PathBuf::from("en.yml"),
        }
    }

    fn create_test_code_reference() -> CodeReference {
        CodeReference {
            file: PathBuf::from("invoices.ts"),
            line: 14,
            pattern: r#"I18n\.t\(['"]([^'"]+)['"]\)"#.to_string(),
            context: "I18n.t('invoice.labels.add_new')".to_string(),
            key_path: "invoice.labels.add_new".to_string(),
        }
    }

    #[test]
    fn test_build_translation_node() {
        let entry = create_test_translation_entry();
        let node = ReferenceTreeBuilder::build_translation_node(&entry);

        assert_eq!(node.node_type, NodeType::Translation);
        assert_eq!(node.content, "invoice.labels.add_new: 'add new'");
        assert!(node.location.is_some());
        assert_eq!(node.location.unwrap().line, 4);
    }

    #[test]
    fn test_build_key_node() {
        let entry = create_test_translation_entry();
        let node = ReferenceTreeBuilder::build_key_node(&entry);

        assert_eq!(node.node_type, NodeType::KeyPath);
        assert_eq!(node.content, "invoice.labels.add_new");
        assert!(node.location.is_none());
    }

    #[test]
    fn test_build_code_node() {
        let code_ref = create_test_code_reference();
        let node = ReferenceTreeBuilder::build_code_node(&code_ref);

        assert_eq!(node.node_type, NodeType::CodeRef);
        assert_eq!(node.content, "I18n.t('invoice.labels.add_new')");
        assert!(node.location.is_some());
        assert_eq!(node.location.as_ref().unwrap().line, 14);
    }

    #[test]
    fn test_build_tree_single_match() {
        let result = SearchResult {
            query: "add new".to_string(),
            translation_entries: vec![create_test_translation_entry()],
            code_references: vec![create_test_code_reference()],
        };

        let tree = ReferenceTreeBuilder::build(&result);

        assert_eq!(tree.root.content, "add new");
        assert_eq!(tree.root.node_type, NodeType::Root);
        assert_eq!(tree.root.children.len(), 1);

        // Check translation node
        let translation = &tree.root.children[0];
        assert_eq!(translation.node_type, NodeType::Translation);
        assert!(translation.content.contains("invoice.labels.add_new"));
        assert_eq!(translation.children.len(), 1);

        // Check key path node
        let key_path = &translation.children[0];
        assert_eq!(key_path.node_type, NodeType::KeyPath);
        assert_eq!(key_path.content, "invoice.labels.add_new");
        assert_eq!(key_path.children.len(), 1);

        // Check code reference node
        let code_ref = &key_path.children[0];
        assert_eq!(code_ref.node_type, NodeType::CodeRef);
        assert!(code_ref.content.contains("I18n.t"));
    }

    #[test]
    fn test_build_tree_multiple_code_refs() {
        let entry = create_test_translation_entry();
        let code_ref1 = create_test_code_reference();
        let mut code_ref2 = create_test_code_reference();
        code_ref2.line = 20;
        code_ref2.context = "I18n.t('invoice.labels.add_new') // another usage".to_string();

        let result = SearchResult {
            query: "add new".to_string(),
            translation_entries: vec![entry],
            code_references: vec![code_ref1, code_ref2],
        };

        let tree = ReferenceTreeBuilder::build(&result);

        // Should have one translation node
        assert_eq!(tree.root.children.len(), 1);

        // Translation should have one key path node
        let translation = &tree.root.children[0];
        assert_eq!(translation.children.len(), 1);

        // Key path should have two code reference nodes
        let key_path = &translation.children[0];
        assert_eq!(key_path.children.len(), 2);
    }

    #[test]
    fn test_build_tree_multiple_translations() {
        let entry1 = create_test_translation_entry();
        let mut entry2 = create_test_translation_entry();
        entry2.key = "invoice.labels.edit".to_string();
        entry2.value = "edit invoice".to_string();

        let code_ref1 = create_test_code_reference();
        let mut code_ref2 = create_test_code_reference();
        code_ref2.key_path = "invoice.labels.edit".to_string();
        code_ref2.context = "I18n.t('invoice.labels.edit')".to_string();

        let result = SearchResult {
            query: "invoice".to_string(),
            translation_entries: vec![entry1, entry2],
            code_references: vec![code_ref1, code_ref2],
        };

        let tree = ReferenceTreeBuilder::build(&result);

        // Should have two translation nodes
        assert_eq!(tree.root.children.len(), 2);

        // Each translation should have one key path with one code ref
        for translation in &tree.root.children {
            assert_eq!(translation.children.len(), 1);
            assert_eq!(translation.children[0].children.len(), 1);
        }
    }

    #[test]
    fn test_build_tree_no_code_refs() {
        let result = SearchResult {
            query: "add new".to_string(),
            translation_entries: vec![create_test_translation_entry()],
            code_references: vec![],
        };

        let tree = ReferenceTreeBuilder::build(&result);

        // Should have one translation node
        assert_eq!(tree.root.children.len(), 1);

        // Translation should have no children (no key path without code refs)
        let translation = &tree.root.children[0];
        assert_eq!(translation.children.len(), 0);
    }

    #[test]
    fn test_build_tree_empty_result() {
        let result = SearchResult {
            query: "nonexistent".to_string(),
            translation_entries: vec![],
            code_references: vec![],
        };

        let tree = ReferenceTreeBuilder::build(&result);

        assert_eq!(tree.root.content, "nonexistent");
        assert_eq!(tree.root.children.len(), 0);
        assert!(!tree.has_results());
    }

    #[test]
    fn test_build_tree_structure() {
        let result = SearchResult {
            query: "add new".to_string(),
            translation_entries: vec![create_test_translation_entry()],
            code_references: vec![create_test_code_reference()],
        };

        let tree = ReferenceTreeBuilder::build(&result);

        // Verify tree structure
        assert_eq!(tree.node_count(), 4); // root + translation + key + code_ref
        assert_eq!(tree.max_depth(), 4);
        assert!(tree.has_results());
    }

    #[test]
    fn test_build_tree_filters_unmatched_code_refs() {
        let entry = create_test_translation_entry();
        let code_ref1 = create_test_code_reference();
        let mut code_ref2 = create_test_code_reference();
        code_ref2.key_path = "different.key".to_string();

        let result = SearchResult {
            query: "add new".to_string(),
            translation_entries: vec![entry],
            code_references: vec![code_ref1, code_ref2],
        };

        let tree = ReferenceTreeBuilder::build(&result);

        // Should only include the matching code ref
        let key_path = &tree.root.children[0].children[0];
        assert_eq!(key_path.children.len(), 1);
        assert!(key_path.children[0]
            .content
            .contains("invoice.labels.add_new"));
    }
}
