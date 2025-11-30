use cs::output::formatter::TreeFormatter;
use cs::trace::{CallNode, CallTree, FunctionDef, TraceDirection};
use regex::Regex;
use std::path::PathBuf;

fn create_test_function(name: &str, file: &str, line: usize) -> FunctionDef {
    FunctionDef {
        name: name.to_string(),
        file: PathBuf::from(file),
        line,
        body: String::new(),
    }
}

fn strip_ansi_codes(s: &str) -> String {
    let re = Regex::new(r"\x1B\[[0-?]*[ -/]*[@-~]").unwrap();
    re.replace_all(s, "").to_string()
}

#[test]
fn test_format_forward_tree() {
    let root_func = create_test_function("root", "root.rs", 1);
    let child_func = create_test_function("child", "child.rs", 10);

    let child_node = CallNode {
        def: child_func,
        children: vec![],
        truncated: false,
    };

    let root_node = CallNode {
        def: root_func,
        children: vec![child_node],
        truncated: false,
    };

    let tree = CallTree { root: root_node };
    let formatter = TreeFormatter::new();
    let output = formatter.format_trace_tree(&tree, TraceDirection::Forward);

    assert!(output.contains("root"));
    assert!(output.contains("child"));
    assert!(output.contains("└─>"));
}

#[test]
fn test_format_backward_tree() {
    // Structure: caller -> callee -> target
    // Tree: target (root) <- callee (child) <- caller (grandchild)

    let target_func = create_test_function("target", "target.rs", 1);
    let callee_func = create_test_function("callee", "callee.rs", 10);
    let caller_func = create_test_function("caller", "caller.rs", 20);

    let caller_node = CallNode {
        def: caller_func,
        children: vec![],
        truncated: false,
    };

    let callee_node = CallNode {
        def: callee_func,
        children: vec![caller_node],
        truncated: false,
    };

    let root_node = CallNode {
        def: target_func,
        children: vec![callee_node],
        truncated: false,
    };

    let tree = CallTree { root: root_node };
    let formatter = TreeFormatter::new();
    let output = formatter.format_trace_tree(&tree, TraceDirection::Backward);

    // Should show chain: caller (file:line) -> callee (file:line) -> target (file:line)
    let stripped_output = strip_ansi_codes(&output);
    assert!(stripped_output
        .contains("caller (caller.rs:20) -> callee (callee.rs:10) -> target (target.rs:1)"));
}

#[test]
fn test_format_truncated_node() {
    let root_func = create_test_function("root", "root.rs", 1);

    let root_node = CallNode {
        def: root_func,
        children: vec![],
        truncated: true,
    };

    let tree = CallTree { root: root_node };
    let formatter = TreeFormatter::new();
    let output = formatter.format_trace_tree(&tree, TraceDirection::Forward);

    assert!(output.contains("[depth limit reached]"));
}
