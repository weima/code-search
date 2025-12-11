//! # Ownership and Borrowing - Rust Book Chapter 4
//!
//! This module demonstrates ownership, borrowing, and references from
//! [The Rust Book Chapter 4](https://doc.rust-lang.org/book/ch04-00-understanding-ownership.html).
//!
//! ## Key Concepts Demonstrated
//!
//! 1. **Borrowing with `&` References** (Chapter 4.2)
//!    - The `build()` method takes `&SearchResult` instead of `SearchResult`
//!    - Allows reading data without taking ownership
//!    - Multiple borrows can exist simultaneously for reading
//!
//! 2. **Ownership Transfer with `.clone()`** (Chapter 4.1)
//!    - `result.query.clone()` creates a new owned `String`
//!    - Necessary when building nodes that need to own their data
//!    - Trade-off: memory cost vs. ownership flexibility
//!
//! 3. **Iterators and Borrowing** (Chapter 4.2 + 13.2)
//!    - `.iter()` creates an iterator of references (`&T`)
//!    - Allows processing collections without taking ownership
//!    - The original collection remains usable after iteration
//!
//! 4. **References in Collections** (Chapter 4.2)
//!    - `Vec<_>` with `.iter()` creates `Vec<&T>`
//!    - Temporary collections of references avoid cloning
//!    - References must not outlive the data they point to
//!
//! ## Learning Notes
//!
//! **Why use `&SearchResult` instead of `SearchResult`?**
//! - Caller retains ownership and can reuse the data
//! - More efficient - no need to clone the entire structure
//! - Follows Rust convention: borrow when you don't need ownership
//!
//! **When to clone vs. when to borrow?**
//! - Clone: When you need to store data in a new structure (like TreeNode)
//! - Borrow: When you only need to read data temporarily
//!
//! **The `.iter()` pattern:**
//! ```rust,ignore
//! for entry in &result.translation_entries {  // Borrows each element
//!     // entry is &TranslationEntry
//! }
//! // vs
//! for entry in result.translation_entries {   // Takes ownership
//!     // entry is TranslationEntry (moved)
//! }
//! ```

use crate::tree::{Location, NodeType, ReferenceTree, TreeNode};
use crate::{CodeReference, SearchResult, TranslationEntry};

/// Builder for constructing reference trees from search results.
///
/// # Rust Book Reference
///
/// **Chapter 4: Understanding Ownership**
/// https://doc.rust-lang.org/book/ch04-00-understanding-ownership.html
///
/// This builder demonstrates how to work with borrowed data to construct
/// owned data structures efficiently.
pub struct ReferenceTreeBuilder;

impl ReferenceTreeBuilder {
    /// Build a reference tree from search results.
    ///
    /// Creates a hierarchical tree structure:
    /// - Root: search query text
    ///   - Translation: translation file entry
    ///     - KeyPath: full translation key
    ///       - CodeRef: code reference using the key
    ///
    /// # Rust Book Reference
    ///
    /// **Chapter 4.2: References and Borrowing**
    /// https://doc.rust-lang.org/book/ch04-02-references-and-borrowing.html
    ///
    /// # Educational Notes - Borrowing with `&SearchResult`
    ///
    /// This method signature demonstrates **immutable borrowing**:
    ///
    /// ```rust,ignore
    /// pub fn build(result: &SearchResult) -> ReferenceTree
    /// //                   ^
    /// //                   Borrows result, doesn't take ownership
    /// ```
    ///
    /// **Why borrow instead of taking ownership?**
    ///
    /// 1. **Caller keeps ownership:**
    ///    ```rust,ignore
    ///    let result = search_translations("add new")?;
    ///    let tree = ReferenceTreeBuilder::build(&result);  // Borrow
    ///    // result is still usable here!
    ///    println!("Found {} entries", result.translation_entries.len());
    ///    ```
    ///
    /// 2. **No unnecessary cloning:**
    ///    - If we took ownership: `build(result: SearchResult)`
    ///    - Caller would need: `build(result.clone())` - expensive!
    ///    - With borrowing: `build(&result)` - zero cost!
    ///
    /// 3. **Rust's borrowing rules ensure safety:**
    ///    - The reference `&result` is valid for the entire function
    ///    - We can read all fields: `result.query`, `result.translation_entries`
    ///    - We cannot modify the data (immutable borrow)
    ///    - The original data cannot be moved while borrowed
    ///
    /// **Inside the function:**
    /// - We borrow fields: `&result.translation_entries`
    /// - We clone when we need ownership: `result.query.clone()`
    /// - We iterate with `.iter()` to borrow elements: `for entry in &result.translation_entries`
    ///
    /// **Key Insight:** Borrowing is Rust's way of saying "I just need to look at
    /// this data temporarily, I don't need to own it."
    pub fn build(result: &SearchResult) -> ReferenceTree {
        // OWNERSHIP: Clone the query string because TreeNode needs to own its content
        // Chapter 4.1: The query is a String, which doesn't implement Copy
        // We must clone to create a new owned value for the TreeNode
        let mut root = TreeNode::new(NodeType::Root, result.query.clone());
        let mut used_code_refs = std::collections::HashSet::new();

        // BORROWING: Iterate over references to avoid moving the vector
        // Chapter 4.2: `&result.translation_entries` borrows the Vec
        // `for entry in &vec` is shorthand for `for entry in vec.iter()`
        // Each `entry` is a `&TranslationEntry` (reference)
        for entry in &result.translation_entries {
            let mut translation_node = Self::build_translation_node(entry);
            let mut key_node = Self::build_key_node(entry);

            // ITERATORS AND BORROWING: Build a collection of references
            // Chapter 13.2: `.iter()` creates an iterator of references
            // `.enumerate()` adds indices: (usize, &CodeReference)
            // `.filter()` borrows each element to check the condition
            // Result: Vec<(usize, &CodeReference)> - no cloning needed!
            let matching_refs: Vec<_> = result
                .code_references
                .iter()
                .enumerate()
                .filter(|(_, r)| r.key_path == entry.key)
                .collect();

            // Add code reference nodes as children of the key node
            for (idx, code_ref) in matching_refs {
                let code_node = Self::build_code_node(code_ref);
                key_node.add_child(code_node);
                used_code_refs.insert(idx);
            }

            // Only add the key node if it has code references
            if key_node.has_children() {
                translation_node.children.push(key_node);
            }

            root.add_child(translation_node);
        }

        // Handle direct matches (code references not associated with any translation entry)
        let direct_matches: Vec<_> = result
            .code_references
            .iter()
            .enumerate()
            .filter(|(idx, _)| !used_code_refs.contains(idx))
            .map(|(_, r)| r)
            .collect();

        if !direct_matches.is_empty() {
            // Create a virtual node for direct matches
            let mut direct_matches_node =
                TreeNode::new(NodeType::KeyPath, "Direct Matches".to_string());

            for code_ref in direct_matches {
                let code_node = Self::build_code_node(code_ref);
                direct_matches_node.add_child(code_node);
            }

            root.add_child(direct_matches_node);
        }

        ReferenceTree::new(root)
    }

    /// Build a translation node from a translation entry.
    ///
    /// # Rust Book Reference
    ///
    /// **Chapter 4.2: References and Borrowing**
    /// https://doc.rust-lang.org/book/ch04-02-references-and-borrowing.html
    ///
    /// # Educational Notes - Borrowing vs. Cloning
    ///
    /// This method takes `&TranslationEntry` (a reference) but returns an owned `TreeNode`.
    ///
    /// **The pattern:**
    /// ```text
    /// fn build_translation_node(entry: &TranslationEntry) -> TreeNode
    /// //                               ^                      ^
    /// //                               Borrow input          Own output
    /// ```
    ///
    /// **Inside the function:**
    /// - We borrow: `entry.file`, `entry.line` (read-only access)
    /// - We clone: `entry.key.clone()`, `entry.value.clone()` (need owned data)
    /// - We move: `location` into the TreeNode (transfer ownership)
    ///
    /// **Why this pattern?**
    /// - Input: Borrow because we don't need to consume the entry
    /// - Output: Own because TreeNode needs to live independently
    /// - Clone: Only the strings we need to store, not the entire entry
    ///
    /// **Memory perspective:**
    /// ```text
    /// entry (borrowed)          TreeNode (owned)
    /// ├─ key: "invoice.add"  --clone--> content: "invoice.add"
    /// ├─ value: "Add New"    --clone--> metadata: "Add New"
    /// ├─ file: "en.yml"      --clone--> location.file: "en.yml"
    /// └─ line: 42            --copy---> location.line: 42
    /// ```
    fn build_translation_node(entry: &TranslationEntry) -> TreeNode {
        // CLONE: PathBuf doesn't implement Copy, so we clone to create owned data
        // Chapter 4.1: Clone creates a deep copy on the heap
        let location = Location::new(entry.file.clone(), entry.line);

        // CLONE: String doesn't implement Copy, clone creates new heap allocation
        let mut node = TreeNode::with_location(NodeType::Translation, entry.key.clone(), location);

        // CLONE: Store owned copy of the translation value
        node.metadata = Some(entry.value.clone());

        // MOVE: Transfer ownership of node to caller
        // Chapter 4.1: The node is moved out of this function
        node
    }

    /// Build a key path node from a translation entry.
    ///
    /// # Educational Notes - Minimal Borrowing
    ///
    /// This is the simplest borrowing pattern:
    /// - Borrow the entry: `&TranslationEntry`
    /// - Clone only what we need: `entry.key.clone()`
    /// - Return owned result: `TreeNode`
    ///
    /// We could have taken ownership: `fn build_key_node(entry: TranslationEntry)`
    /// But then the caller would lose access to `entry` after calling this function.
    fn build_key_node(entry: &TranslationEntry) -> TreeNode {
        TreeNode::new(NodeType::KeyPath, entry.key.clone())
    }

    /// Build a code reference node.
    ///
    /// # Educational Notes - Borrowing Composite Types
    ///
    /// `CodeReference` contains multiple fields:
    /// - `file: PathBuf` - heap-allocated path
    /// - `line: usize` - stack-allocated number (Copy type)
    /// - `context: String` - heap-allocated string
    /// - `key_path: String` - heap-allocated string
    ///
    /// By borrowing `&CodeReference`, we can access all fields without cloning
    /// the entire struct. We only clone the specific fields we need to store.
    fn build_code_node(code_ref: &CodeReference) -> TreeNode {
        let location = Location::new(code_ref.file.clone(), code_ref.line);
        let mut node =
            TreeNode::with_location(NodeType::CodeRef, code_ref.context.clone(), location);
        // Store the key path (or search pattern) in metadata for highlighting
        node.metadata = Some(code_ref.key_path.clone());
        node
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
            context_before: vec![],
            context_after: vec![],
        }
    }

    #[test]
    fn test_build_translation_node() {
        let entry = create_test_translation_entry();
        let node = ReferenceTreeBuilder::build_translation_node(&entry);

        assert_eq!(node.node_type, NodeType::Translation);
        assert_eq!(node.content, "invoice.labels.add_new");
        assert_eq!(node.metadata.as_deref(), Some("add new"));
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
        assert_eq!(translation.content, "invoice.labels.add_new");
        assert_eq!(translation.metadata.as_deref(), Some("add new"));
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
