//! # Traits and Derive Macros - Rust Book Chapter 10
//!
//! This module demonstrates traits and derive macros from
//! [The Rust Book Chapter 10](https://doc.rust-lang.org/book/ch10-00-generics.html).
//!
//! ## Key Concepts Demonstrated
//!
//! 1. **Derive Macros for Common Traits** (Chapter 10.2)
//!    - `#[derive(Debug)]` - Automatic debug formatting
//!    - `#[derive(Clone)]` - Automatic cloning implementation
//!    - `#[derive(PartialEq, Eq)]` - Equality comparisons
//!
//! 2. **Trait Bounds and Requirements** (Chapter 10.2)
//!    - Why certain traits require others (e.g., `Eq` requires `PartialEq`)
//!    - How derive macros generate implementations
//!
//! 3. **Enums with Derive** (Chapter 6.1 + 10.2)
//!    - Deriving traits for enums
//!    - All variants must support the derived trait
//!
//! ## Learning Notes
//!
//! **What are derive macros?**
//! - Compiler-generated trait implementations
//! - Reduce boilerplate code
//! - Only work for types where all fields implement the trait
//!
//! **Common derive traits:**
//! - `Debug` - For `{:?}` formatting and debugging
//! - `Clone` - For `.clone()` method
//! - `PartialEq` - For `==` and `!=` operators
//! - `Eq` - For full equality (requires `PartialEq`)
//! - `Copy` - For implicit copying (only for stack types)

use std::path::PathBuf;

/// Type of node in the reference tree.
///
/// # Rust Book Reference
///
/// **Chapter 10.2: Traits - Deriving Traits**
/// https://doc.rust-lang.org/book/ch10-02-traits.html#deriving-traits
///
/// **Chapter 6.1: Defining an Enum**
/// https://doc.rust-lang.org/book/ch06-01-defining-an-enum.html
///
/// # Educational Notes - Derive Macros on Enums
///
/// This enum demonstrates multiple derived traits:
///
/// ```rust,ignore
/// #[derive(Debug, Clone, PartialEq, Eq)]
/// pub enum NodeType { ... }
/// ```
///
/// **What each trait provides:**
///
/// 1. **`Debug`** - Enables `println!("{:?}", node_type)`
///    - Essential for debugging and error messages
///    - Automatically formats enum variants
///
/// 2. **`Clone`** - Enables `.clone()` method
///    - Creates a deep copy of the enum value
///    - For enums without data, this is trivial
///
/// 3. **`PartialEq`** - Enables `==` and `!=` operators
///    - Compares enum variants for equality
///    - `Root == Root` is true, `Root == Translation` is false
///
/// 4. **`Eq`** - Marker trait for full equality
///    - Requires `PartialEq` first
///    - Indicates equality is reflexive, symmetric, and transitive
///    - Needed for using types as HashMap keys
///
/// **Why derive instead of manual implementation?**
/// - Less code to write and maintain
/// - Compiler-generated code is correct and efficient
/// - Consistent behavior across the codebase
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NodeType {
    /// Root node containing the search text
    Root,
    /// Translation file entry
    Translation,
    /// Full key path (e.g., "invoice.labels.add_new")
    KeyPath,
    /// Code reference where the key is used
    CodeRef,
}

/// Location information for a node.
///
/// # Rust Book Reference
///
/// **Chapter 10.2: Traits - Deriving Traits**
/// https://doc.rust-lang.org/book/ch10-02-traits.html#deriving-traits
///
/// # Educational Notes - Derive on Structs
///
/// This struct demonstrates deriving traits on a struct with multiple fields:
///
/// ```rust,ignore
/// #[derive(Debug, Clone, PartialEq, Eq)]
/// pub struct Location {
///     pub file: PathBuf,
///     pub line: usize,
/// }
/// ```
///
/// **How derive works for structs:**
/// - Compiler checks that ALL fields implement the trait
/// - `PathBuf` implements `Debug, Clone, PartialEq, Eq` ✓
/// - `usize` implements `Debug, Clone, PartialEq, Eq` ✓
/// - Therefore, `Location` can derive these traits
///
/// **Generated `PartialEq` implementation:**
/// ```rust,ignore
/// impl PartialEq for Location {
///     fn eq(&self, other: &Self) -> bool {
///         self.file == other.file && self.line == other.line
///     }
/// }
/// ```
///
/// **Why this matters:**
/// - Locations can be compared: `loc1 == loc2`
/// - Locations can be cloned: `loc.clone()`
/// - Locations can be debugged: `println!("{:?}", loc)`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Location {
    pub file: PathBuf,
    pub line: usize,
}

impl Location {
    pub fn new(file: PathBuf, line: usize) -> Self {
        Self { file, line }
    }
}

/// A node in the reference tree.
///
/// # Rust Book Reference
///
/// **Chapter 10.2: Traits**
/// https://doc.rust-lang.org/book/ch10-02-traits.html
///
/// # Educational Notes - Selective Trait Derivation
///
/// Notice this struct only derives `Debug` and `Clone`, not `PartialEq`:
///
/// ```rust,ignore
/// #[derive(Debug, Clone)]  // No PartialEq!
/// pub struct TreeNode { ... }
/// ```
///
/// **Why not derive `PartialEq`?**
/// - `TreeNode` contains `Vec<TreeNode>` (recursive structure)
/// - Comparing entire trees could be expensive
/// - We don't need to compare trees for equality in this application
/// - Omitting `PartialEq` prevents accidental expensive comparisons
///
/// **Why derive `Clone`?**
/// - We need to clone nodes when building trees
/// - `Clone` is explicit (`.clone()`) so we know when it happens
/// - All fields implement `Clone`:
///   - `NodeType: Clone` ✓
///   - `String: Clone` ✓
///   - `Option<Location>: Clone` ✓ (Location implements Clone)
///   - `Vec<TreeNode>: Clone` ✓ (recursive, but works)
///   - `Option<String>: Clone` ✓
///
/// **Design principle:**
/// Only derive traits you actually need. This prevents:
/// - Accidental expensive operations
/// - Unnecessary trait bound requirements
/// - Compilation errors when adding non-Clone fields later
#[derive(Debug, Clone)]
pub struct TreeNode {
    pub node_type: NodeType,
    pub content: String,
    pub location: Option<Location>,
    pub children: Vec<TreeNode>,
    pub metadata: Option<String>,
}

impl TreeNode {
    /// Create a new TreeNode
    pub fn new(node_type: NodeType, content: String) -> Self {
        Self {
            node_type,
            content,
            location: None,
            children: Vec::new(),
            metadata: None,
        }
    }

    /// Create a TreeNode with a location
    pub fn with_location(node_type: NodeType, content: String, location: Location) -> Self {
        Self {
            node_type,
            content,
            location: Some(location),
            children: Vec::new(),
            metadata: None,
        }
    }

    /// Add a child node
    pub fn add_child(&mut self, child: TreeNode) {
        self.children.push(child);
    }

    /// Check if this node has children
    pub fn has_children(&self) -> bool {
        !self.children.is_empty()
    }

    /// Get the number of children
    pub fn child_count(&self) -> usize {
        self.children.len()
    }

    /// Get the total number of nodes in the tree (including this node)
    pub fn node_count(&self) -> usize {
        1 + self.children.iter().map(|c| c.node_count()).sum::<usize>()
    }

    /// Get the maximum depth of the tree
    pub fn max_depth(&self) -> usize {
        if self.children.is_empty() {
            1
        } else {
            1 + self
                .children
                .iter()
                .map(|c| c.max_depth())
                .max()
                .unwrap_or(0)
        }
    }
}

/// A reference tree representing the search results
#[derive(Debug)]
pub struct ReferenceTree {
    pub root: TreeNode,
}

impl ReferenceTree {
    /// Create a new ReferenceTree with a root node
    pub fn new(root: TreeNode) -> Self {
        Self { root }
    }

    /// Create a ReferenceTree with a root containing the search text
    pub fn with_search_text(search_text: String) -> Self {
        Self {
            root: TreeNode::new(NodeType::Root, search_text),
        }
    }

    /// Get the total number of nodes in the tree
    pub fn node_count(&self) -> usize {
        self.root.node_count()
    }

    /// Get the maximum depth of the tree
    pub fn max_depth(&self) -> usize {
        self.root.max_depth()
    }

    /// Check if the tree has any results (children of root)
    pub fn has_results(&self) -> bool {
        self.root.has_children()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_tree_node() {
        let node = TreeNode::new(NodeType::Root, "add new".to_string());
        assert_eq!(node.content, "add new");
        assert_eq!(node.node_type, NodeType::Root);
        assert!(node.location.is_none());
        assert!(node.children.is_empty());
    }

    #[test]
    fn test_create_tree_node_with_location() {
        let location = Location::new(PathBuf::from("test.yml"), 10);
        let node = TreeNode::with_location(
            NodeType::Translation,
            "add_new: 'add new'".to_string(),
            location.clone(),
        );

        assert_eq!(node.content, "add_new: 'add new'");
        assert_eq!(node.node_type, NodeType::Translation);
        assert!(node.location.is_some());
        assert_eq!(node.location.unwrap().line, 10);
    }

    #[test]
    fn test_add_child() {
        let mut parent = TreeNode::new(NodeType::Root, "root".to_string());
        let child = TreeNode::new(NodeType::Translation, "child".to_string());

        assert_eq!(parent.child_count(), 0);
        parent.add_child(child);
        assert_eq!(parent.child_count(), 1);
        assert!(parent.has_children());
    }

    #[test]
    fn test_node_count() {
        let mut root = TreeNode::new(NodeType::Root, "root".to_string());
        let mut child1 = TreeNode::new(NodeType::Translation, "child1".to_string());
        let child2 = TreeNode::new(NodeType::Translation, "child2".to_string());
        let grandchild = TreeNode::new(NodeType::KeyPath, "grandchild".to_string());

        child1.add_child(grandchild);
        root.add_child(child1);
        root.add_child(child2);

        // root + child1 + child2 + grandchild = 4
        assert_eq!(root.node_count(), 4);
    }

    #[test]
    fn test_max_depth() {
        let mut root = TreeNode::new(NodeType::Root, "root".to_string());
        let mut child = TreeNode::new(NodeType::Translation, "child".to_string());
        let grandchild = TreeNode::new(NodeType::KeyPath, "grandchild".to_string());

        // Depth 1: just root
        assert_eq!(root.max_depth(), 1);

        // Depth 2: root -> child
        root.add_child(child.clone());
        assert_eq!(root.max_depth(), 2);

        // Depth 3: root -> child -> grandchild
        child.add_child(grandchild);
        root.children[0] = child;
        assert_eq!(root.max_depth(), 3);
    }

    #[test]
    fn test_reference_tree_creation() {
        let tree = ReferenceTree::with_search_text("add new".to_string());
        assert_eq!(tree.root.content, "add new");
        assert_eq!(tree.root.node_type, NodeType::Root);
        assert!(!tree.has_results());
    }

    #[test]
    fn test_reference_tree_with_results() {
        let mut root = TreeNode::new(NodeType::Root, "add new".to_string());
        let child = TreeNode::new(NodeType::Translation, "translation".to_string());
        root.add_child(child);

        let tree = ReferenceTree::new(root);
        assert!(tree.has_results());
        assert_eq!(tree.node_count(), 2);
        assert_eq!(tree.max_depth(), 2);
    }

    #[test]
    fn test_location_creation() {
        let location = Location::new(PathBuf::from("test.yml"), 42);
        assert_eq!(location.file, PathBuf::from("test.yml"));
        assert_eq!(location.line, 42);
    }

    #[test]
    fn test_node_types() {
        let root = NodeType::Root;
        let translation = NodeType::Translation;
        let key_path = NodeType::KeyPath;
        let code_ref = NodeType::CodeRef;

        assert_eq!(root, NodeType::Root);
        assert_eq!(translation, NodeType::Translation);
        assert_eq!(key_path, NodeType::KeyPath);
        assert_eq!(code_ref, NodeType::CodeRef);
    }

    #[test]
    fn test_complex_tree_structure() {
        // Build a tree: root -> translation -> key_path -> code_ref
        let mut root = TreeNode::new(NodeType::Root, "add new".to_string());

        let mut translation = TreeNode::with_location(
            NodeType::Translation,
            "add_new: 'add new'".to_string(),
            Location::new(PathBuf::from("en.yml"), 4),
        );

        let mut key_path = TreeNode::new(NodeType::KeyPath, "invoice.labels.add_new".to_string());

        let code_ref = TreeNode::with_location(
            NodeType::CodeRef,
            "I18n.t('invoice.labels.add_new')".to_string(),
            Location::new(PathBuf::from("invoices.ts"), 14),
        );

        key_path.add_child(code_ref);
        translation.add_child(key_path);
        root.add_child(translation);

        let tree = ReferenceTree::new(root);

        assert_eq!(tree.node_count(), 4);
        assert_eq!(tree.max_depth(), 4);
        assert!(tree.has_results());
    }
}
