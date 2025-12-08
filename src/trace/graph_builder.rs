//! # Lifetimes - Rust Book Chapter 10.3
//!
//! This module demonstrates lifetime annotations from
//! [The Rust Book Chapter 10.3](https://doc.rust-lang.org/book/ch10-03-lifetime-syntax.html).
//!
//! ## Key Concepts Demonstrated
//!
//! 1. **Lifetime Parameters on Structs** (Chapter 10.3)
//!    - `CallGraphBuilder<'a>` holds references with lifetime `'a`
//!    - Prevents the struct from outliving its referenced data
//!    - Zero-cost abstraction - no runtime overhead
//!
//! 2. **Why Lifetimes Matter**
//!    - Prevent dangling references at compile time
//!    - Enable borrowing without ownership transfer
//!    - Alternative to heap allocation (`Box`) or reference counting (`Rc`)
//!
//! 3. **When You Need Explicit Lifetimes**
//!    - Structs that store references
//!    - Functions returning references
//!    - Multiple references with different lifetimes
//!
//! ## Learning Notes
//!
//! **Most code doesn't need explicit lifetimes!**
//! - Rust has "lifetime elision" rules that infer lifetimes
//! - You only write lifetimes when Rust can't figure them out
//! - This module is a good example of when they're necessary
//!
//! **The lifetime syntax:**
//! - `'a` - A lifetime parameter (can be any name: `'b`, `'lifetime`, etc.)
//! - `&'a T` - A reference to `T` that lives for lifetime `'a`
//! - `&'a mut T` - A mutable reference to `T` that lives for lifetime `'a`
//!
//! **Common lifetime names:**
//! - `'a`, `'b`, `'c` - Generic lifetime parameters
//! - `'static` - Special lifetime for the entire program duration

use crate::error::Result;
use crate::trace::{CallExtractor, FunctionDef, FunctionFinder};
use std::collections::HashSet;

/// Direction of the call graph trace
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TraceDirection {
    /// Trace forward: which functions does this function call?
    Forward,
    /// Trace backward: which functions call this function?
    Backward,
}

/// A node in the call graph tree
#[derive(Debug, Clone)]
pub struct CallNode {
    /// The function definition for this node
    pub def: FunctionDef,
    /// Child nodes (called functions or callers, depending on direction)
    pub children: Vec<CallNode>,
    /// Whether the tree was truncated at this node due to depth limit
    pub truncated: bool,
}

/// Represents a complete call graph tree
#[derive(Debug, Clone)]
pub struct CallTree {
    /// The root node of the tree (the starting function)
    pub root: CallNode,
}

/// Builds a call graph by recursively tracing function calls.
///
/// # Rust Book Reference
///
/// **Chapter 10.3: Validating References with Lifetimes**
/// https://doc.rust-lang.org/book/ch10-03-lifetime-syntax.html
///
/// # Educational Notes - Lifetime Parameters
///
/// This struct demonstrates explicit lifetime annotations:
///
/// ```rust,ignore
/// pub struct CallGraphBuilder<'a> {
///     finder: &'a mut FunctionFinder,
///     extractor: &'a CallExtractor,
/// }
/// ```
///
/// **What does `<'a>` mean?**
/// - `'a` is a **lifetime parameter** (read as "lifetime a")
/// - It says: "This struct holds references that live for lifetime 'a"
/// - The struct cannot outlive the data it references
///
/// **Why do we need lifetimes here?**
/// - `CallGraphBuilder` stores references to `FunctionFinder` and `CallExtractor`
/// - Without lifetimes, Rust doesn't know how long these references are valid
/// - The lifetime `'a` connects the struct's lifetime to its references
///
/// **What does this prevent?**
/// ```rust,ignore
/// let builder = {
///     let finder = FunctionFinder::new(...);
///     let extractor = CallExtractor::new(...);
///     CallGraphBuilder::new(..., &mut finder, &extractor)
/// }; // ERROR! finder and extractor are dropped here
/// // builder would have dangling references!
/// ```
///
/// **The lifetime constraint:**
/// - `CallGraphBuilder<'a>` can only exist while `finder` and `extractor` exist
/// - Rust enforces this at compile time
/// - No runtime overhead - pure compile-time safety
///
/// **Alternative without lifetimes:**
/// - Could use `Box<FunctionFinder>` (heap allocation + ownership)
/// - Could use `Rc<RefCell<FunctionFinder>>` (reference counting + runtime checks)
/// - Lifetimes are zero-cost: no allocation, no runtime checks
pub struct CallGraphBuilder<'a> {
    direction: TraceDirection,
    max_depth: usize,
    finder: &'a mut FunctionFinder,
    extractor: &'a CallExtractor,
}

impl<'a> CallGraphBuilder<'a> {
    /// Create a new CallGraphBuilder.
    ///
    /// # Rust Book Reference
    ///
    /// **Chapter 10.3: Lifetime Annotations in Function Signatures**
    /// https://doc.rust-lang.org/book/ch10-03-lifetime-syntax.html#lifetime-annotations-in-function-signatures
    ///
    /// # Educational Notes - Lifetime in Function Signatures
    ///
    /// This function signature shows how lifetimes flow through constructors:
    ///
    /// ```rust,ignore
    /// pub fn new(
    ///     finder: &'a mut FunctionFinder,
    ///     extractor: &'a CallExtractor,
    /// ) -> Self
    /// ```
    ///
    /// **What this means:**
    /// - The returned `CallGraphBuilder<'a>` contains references with lifetime `'a`
    /// - Those references come from the input parameters
    /// - The builder cannot outlive `finder` or `extractor`
    ///
    /// **Lifetime elision:**
    /// Without the struct's `<'a>`, we'd need to write:
    /// ```rust,ignore
    /// pub fn new<'a>(
    ///     finder: &'a mut FunctionFinder,
    ///     extractor: &'a CallExtractor,
    /// ) -> CallGraphBuilder<'a>
    /// ```
    /// But since the struct already has `<'a>`, we can use `Self`.
    ///
    /// # Arguments
    /// * `direction` - Whether to trace forward or backward
    /// * `max_depth` - Maximum depth of the call tree
    /// * `finder` - Service to find function definitions
    /// * `extractor` - Service to extract calls from code
    pub fn new(
        direction: TraceDirection,
        max_depth: usize,
        finder: &'a mut FunctionFinder,
        extractor: &'a CallExtractor,
    ) -> Self {
        Self {
            direction,
            max_depth,
            finder,
            extractor,
        }
    }

    /// Build a call trace tree starting from the given function
    pub fn build_trace(&mut self, start_fn: &FunctionDef) -> Result<Option<CallTree>> {
        let mut current_path = HashSet::new();

        match self.build_node(start_fn, 0, &mut current_path) {
            Some(root) => Ok(Some(CallTree { root })),
            None => Ok(None),
        }
    }

    /// Recursively build a call tree node
    ///
    /// Uses proper cycle detection with current_path to prevent infinite recursion
    /// while still allowing the same function to appear in different branches.
    fn build_node(
        &mut self,
        func: &FunctionDef,
        depth: usize,
        current_path: &mut HashSet<FunctionDef>,
    ) -> Option<CallNode> {
        // Check depth limit
        if depth >= self.max_depth {
            return Some(CallNode {
                def: func.clone(),
                children: vec![],
                truncated: true,
            });
        }

        // Check for cycles in current path (prevents infinite recursion)
        if current_path.contains(func) {
            return Some(CallNode {
                def: func.clone(),
                children: vec![],
                truncated: false, // Not truncated by depth, but by cycle
            });
        }

        // Add to current path for cycle detection
        current_path.insert(func.clone());

        let children = match self.direction {
            TraceDirection::Forward => self.build_forward_children(func, depth, current_path),
            TraceDirection::Backward => self.build_backward_children(func, depth, current_path),
        };

        // Remove from current path (allows same function in different branches)
        current_path.remove(func);

        Some(CallNode {
            def: func.clone(),
            children,
            truncated: false,
        })
    }

    /// Build children for forward tracing (what does this function call?)
    fn build_forward_children(
        &mut self,
        func: &FunctionDef,
        depth: usize,
        current_path: &mut HashSet<FunctionDef>,
    ) -> Vec<CallNode> {
        // Extract function calls from this function's body
        let call_names = match self.extractor.extract_calls(func) {
            Ok(calls) => calls,
            Err(_) => return vec![], // If extraction fails, return empty children
        };

        let mut children = Vec::new();

        for call_name in call_names {
            // Find the definition of the called function
            if let Some(called_func) = self.finder.find_function(&call_name) {
                // Recursively build the child node
                if let Some(child_node) = self.build_node(&called_func, depth + 1, current_path) {
                    children.push(child_node);
                }
            }
            // If function not found, we simply don't include it (graceful handling)
        }

        children
    }

    /// Build children for backward tracing (who calls this function?)
    fn build_backward_children(
        &mut self,
        func: &FunctionDef,
        depth: usize,
        current_path: &mut HashSet<FunctionDef>,
    ) -> Vec<CallNode> {
        // Find all functions that call this function
        let callers = match self.extractor.find_callers(&func.name) {
            Ok(caller_infos) => caller_infos,
            Err(_) => return vec![], // If finding callers fails, return empty children
        };

        let mut children = Vec::new();

        for caller_info in callers {
            // Try to find the caller function definition
            if let Some(caller_func) = self.finder.find_function(&caller_info.caller_name) {
                // Avoid adding the same caller multiple times
                if !children.iter().any(|child: &CallNode| {
                    child.def.name == caller_func.name && child.def.file == caller_func.file
                }) {
                    // Recursively build the child node
                    if let Some(child_node) = self.build_node(&caller_func, depth + 1, current_path)
                    {
                        children.push(child_node);
                    }
                }
            }
            // If caller function not found, we simply don't include it (graceful handling)
        }

        children
    }
}

impl CallTree {
    /// Get the total number of nodes in the tree
    pub fn node_count(&self) -> usize {
        Self::count_nodes(&self.root)
    }

    /// Get the maximum depth of the tree
    pub fn max_depth(&self) -> usize {
        Self::calculate_depth(&self.root, 0)
    }

    /// Check if the tree contains cycles
    pub fn has_cycles(&self) -> bool {
        let mut visited = HashSet::new();
        let mut path = HashSet::new();
        Self::has_cycle_helper(&self.root, &mut visited, &mut path)
    }

    fn count_nodes(node: &CallNode) -> usize {
        1 + node.children.iter().map(Self::count_nodes).sum::<usize>()
    }

    fn calculate_depth(node: &CallNode, current_depth: usize) -> usize {
        if node.children.is_empty() {
            current_depth
        } else {
            node.children
                .iter()
                .map(|child| Self::calculate_depth(child, current_depth + 1))
                .max()
                .unwrap_or(current_depth)
        }
    }

    fn has_cycle_helper(
        node: &CallNode,
        visited: &mut HashSet<FunctionDef>,
        path: &mut HashSet<FunctionDef>,
    ) -> bool {
        if path.contains(&node.def) {
            return true; // Found a cycle
        }

        if visited.contains(&node.def) {
            return false; // Already processed this node
        }

        visited.insert(node.def.clone());
        path.insert(node.def.clone());

        for child in &node.children {
            if Self::has_cycle_helper(child, visited, path) {
                return true;
            }
        }

        path.remove(&node.def);
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn create_test_function(name: &str, file: &str, line: usize) -> FunctionDef {
        FunctionDef {
            name: name.to_string(),
            file: PathBuf::from(file),
            line,
            body: format!("function {}() {{}}", name),
        }
    }

    #[test]
    fn test_trace_direction_equality() {
        assert_eq!(TraceDirection::Forward, TraceDirection::Forward);
        assert_eq!(TraceDirection::Backward, TraceDirection::Backward);
        assert_ne!(TraceDirection::Forward, TraceDirection::Backward);
    }

    #[test]
    fn test_call_node_creation() {
        let func = create_test_function("test_func", "test.js", 10);
        let node = CallNode {
            def: func.clone(),
            children: vec![],
            truncated: false,
        };

        assert_eq!(node.def.name, "test_func");
        assert_eq!(node.children.len(), 0);
        assert!(!node.truncated);
    }

    #[test]
    fn test_call_tree_creation() {
        let func = create_test_function("main", "main.js", 1);
        let root = CallNode {
            def: func,
            children: vec![],
            truncated: false,
        };
        let tree = CallTree { root };

        assert_eq!(tree.root.def.name, "main");
    }

    #[test]
    fn test_call_tree_node_count() {
        let main_func = create_test_function("main", "main.js", 1);
        let helper_func = create_test_function("helper", "utils.js", 5);

        let helper_node = CallNode {
            def: helper_func,
            children: vec![],
            truncated: false,
        };

        let root = CallNode {
            def: main_func,
            children: vec![helper_node],
            truncated: false,
        };

        let tree = CallTree { root };
        assert_eq!(tree.node_count(), 2);
    }

    #[test]
    fn test_call_tree_max_depth() {
        let func1 = create_test_function("func1", "test.js", 1);
        let func2 = create_test_function("func2", "test.js", 10);
        let func3 = create_test_function("func3", "test.js", 20);

        // Create a chain: func1 -> func2 -> func3
        let node3 = CallNode {
            def: func3,
            children: vec![],
            truncated: false,
        };

        let node2 = CallNode {
            def: func2,
            children: vec![node3],
            truncated: false,
        };

        let root = CallNode {
            def: func1,
            children: vec![node2],
            truncated: false,
        };

        let tree = CallTree { root };
        assert_eq!(tree.max_depth(), 2); // 0-indexed depth
    }

    #[test]
    fn test_call_graph_builder_creation() {
        use crate::trace::{CallExtractor, FunctionFinder};
        use std::env;

        let base_dir = env::current_dir().unwrap();
        let mut finder = FunctionFinder::new(base_dir.clone());
        let extractor = CallExtractor::new(base_dir);

        let builder = CallGraphBuilder::new(TraceDirection::Forward, 5, &mut finder, &extractor);

        assert_eq!(builder.direction, TraceDirection::Forward);
        assert_eq!(builder.max_depth, 5);
    }

    #[test]
    fn test_depth_limit_handling() {
        use crate::trace::{CallExtractor, FunctionFinder};
        use std::env;

        let base_dir = env::current_dir().unwrap();
        let mut finder = FunctionFinder::new(base_dir.clone());
        let extractor = CallExtractor::new(base_dir);

        let mut builder = CallGraphBuilder::new(
            TraceDirection::Forward,
            0, // Max depth of 0 should only return root
            &mut finder,
            &extractor,
        );

        let test_func = create_test_function("test", "test.js", 1);
        let mut path = HashSet::new();
        let result = builder.build_node(&test_func, 0, &mut path);

        assert!(result.is_some());
        let node = result.unwrap();
        assert_eq!(node.def.name, "test");
        assert_eq!(node.children.len(), 0); // Should have no children due to depth limit
        assert!(node.truncated); // Should be truncated
    }

    #[test]
    fn test_cycle_detection() {
        use crate::trace::{CallExtractor, FunctionFinder};
        use std::env;

        let base_dir = env::current_dir().unwrap();
        let mut finder = FunctionFinder::new(base_dir.clone());
        let extractor = CallExtractor::new(base_dir);

        let mut builder =
            CallGraphBuilder::new(TraceDirection::Forward, 10, &mut finder, &extractor);

        let test_func = create_test_function("recursive", "test.js", 1);
        let mut path = HashSet::new();

        // Add the function to path to simulate cycle detection
        path.insert(test_func.clone());

        let result = builder.build_node(&test_func, 0, &mut path);

        assert!(result.is_some());
        let node = result.unwrap();
        assert_eq!(node.children.len(), 0); // Should stop due to cycle detection
    }

    #[test]
    fn test_function_def_equality() {
        let func1 = create_test_function("test", "file.js", 10);
        let func2 = create_test_function("test", "file.js", 10);
        let func3 = create_test_function("test", "file.js", 20);

        assert_eq!(func1, func2);
        assert_ne!(func1, func3); // Different line numbers
    }
}
