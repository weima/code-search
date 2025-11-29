use crate::error::Result;
use crate::trace::{CallExtractor, FunctionDef, FunctionFinder};
use std::collections::HashSet;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TraceDirection {
    Forward,
    Backward,
}

#[derive(Debug, Clone)]
pub struct CallNode {
    pub def: FunctionDef,
    pub children: Vec<CallNode>,
}

#[derive(Debug, Clone)]
pub struct CallTree {
    pub root: CallNode,
}

pub struct CallGraphBuilder<'a> {
    direction: TraceDirection,
    max_depth: usize,
    finder: &'a FunctionFinder,
    extractor: &'a CallExtractor,
}

impl<'a> CallGraphBuilder<'a> {
    pub fn new(
        direction: TraceDirection,
        max_depth: usize,
        finder: &'a FunctionFinder,
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
    pub fn build_trace(&self, start_fn: &FunctionDef) -> Result<Option<CallTree>> {
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
        &self,
        func: &FunctionDef,
        depth: usize,
        current_path: &mut HashSet<FunctionDef>,
    ) -> Option<CallNode> {
        // Check depth limit
        if depth >= self.max_depth {
            return Some(CallNode {
                def: func.clone(),
                children: vec![],
            });
        }

        // Check for cycles in current path (prevents infinite recursion)
        if current_path.contains(func) {
            return Some(CallNode {
                def: func.clone(),
                children: vec![],
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
        })
    }

    /// Build children for forward tracing (what does this function call?)
    fn build_forward_children(
        &self,
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
        &self,
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
        self.count_nodes(&self.root)
    }

    /// Get the maximum depth of the tree
    pub fn max_depth(&self) -> usize {
        self.calculate_depth(&self.root, 0)
    }

    /// Check if the tree contains cycles
    pub fn has_cycles(&self) -> bool {
        let mut visited = HashSet::new();
        let mut path = HashSet::new();
        self.has_cycle_helper(&self.root, &mut visited, &mut path)
    }

    fn count_nodes(&self, node: &CallNode) -> usize {
        1 + node
            .children
            .iter()
            .map(|child| self.count_nodes(child))
            .sum::<usize>()
    }

    fn calculate_depth(&self, node: &CallNode, current_depth: usize) -> usize {
        if node.children.is_empty() {
            current_depth
        } else {
            node.children
                .iter()
                .map(|child| self.calculate_depth(child, current_depth + 1))
                .max()
                .unwrap_or(current_depth)
        }
    }

    fn has_cycle_helper(
        &self,
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
            if self.has_cycle_helper(child, visited, path) {
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
        };

        assert_eq!(node.def.name, "test_func");
        assert_eq!(node.children.len(), 0);
    }

    #[test]
    fn test_call_tree_creation() {
        let func = create_test_function("main", "main.js", 1);
        let root = CallNode {
            def: func,
            children: vec![],
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
        };

        let root = CallNode {
            def: main_func,
            children: vec![helper_node],
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
        };

        let node2 = CallNode {
            def: func2,
            children: vec![node3],
        };

        let root = CallNode {
            def: func1,
            children: vec![node2],
        };

        let tree = CallTree { root };
        assert_eq!(tree.max_depth(), 2); // 0-indexed depth
    }

    #[test]
    fn test_call_graph_builder_creation() {
        use crate::trace::{CallExtractor, FunctionFinder};
        use std::env;

        let base_dir = env::current_dir().unwrap();
        let finder = FunctionFinder::new(base_dir.clone());
        let extractor = CallExtractor::new(base_dir);

        let builder = CallGraphBuilder::new(TraceDirection::Forward, 5, &finder, &extractor);

        assert_eq!(builder.direction, TraceDirection::Forward);
        assert_eq!(builder.max_depth, 5);
    }

    #[test]
    fn test_depth_limit_handling() {
        use crate::trace::{CallExtractor, FunctionFinder};
        use std::env;

        let base_dir = env::current_dir().unwrap();
        let finder = FunctionFinder::new(base_dir.clone());
        let extractor = CallExtractor::new(base_dir);

        let builder = CallGraphBuilder::new(
            TraceDirection::Forward,
            0, // Max depth of 0 should only return root
            &finder,
            &extractor,
        );

        let test_func = create_test_function("test", "test.js", 1);
        let mut path = HashSet::new();
        let result = builder.build_node(&test_func, 0, &mut path);

        assert!(result.is_some());
        let node = result.unwrap();
        assert_eq!(node.def.name, "test");
        assert_eq!(node.children.len(), 0); // Should have no children due to depth limit
    }

    #[test]
    fn test_cycle_detection() {
        use crate::trace::{CallExtractor, FunctionFinder};
        use std::env;

        let base_dir = env::current_dir().unwrap();
        let finder = FunctionFinder::new(base_dir.clone());
        let extractor = CallExtractor::new(base_dir);

        let builder = CallGraphBuilder::new(TraceDirection::Forward, 10, &finder, &extractor);

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
