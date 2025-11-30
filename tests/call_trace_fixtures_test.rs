use cs::{run_trace, TraceDirection, TraceQuery};
use std::path::PathBuf;

fn get_fixtures_dir() -> PathBuf {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("tests");
    path.push("fixtures");
    path.push("call-trace");
    path
}

#[test]
fn test_js_nested_calls() {
    let base_dir = get_fixtures_dir();
    // Trace "a" -> "b" -> "c"
    let query = TraceQuery::new("a".to_string(), TraceDirection::Forward, 3)
        .with_base_dir(base_dir);

    let result = run_trace(query).expect("Trace failed");
    assert!(result.is_some());
    let tree = result.unwrap();

    // Verify root is "a"
    assert_eq!(tree.root.def.name, "a");
    assert_eq!(tree.root.children.len(), 1);
    
    // Verify child is "b"
    let b_node = &tree.root.children[0];
    assert_eq!(b_node.def.name, "b");
    assert_eq!(b_node.children.len(), 1);

    // Verify grandchild is "c"
    let c_node = &b_node.children[0];
    assert_eq!(c_node.def.name, "c");
}

#[test]
fn test_js_cycle() {
    let base_dir = get_fixtures_dir();
    // Trace "cycleA" -> "cycleB" -> "cycleA"
    let query = TraceQuery::new("cycleA".to_string(), TraceDirection::Forward, 5)
        .with_base_dir(base_dir);

    let result = run_trace(query).expect("Trace failed");
    assert!(result.is_some());
    let tree = result.unwrap();

    // root: cycleA
    assert_eq!(tree.root.def.name, "cycleA");
    assert_eq!(tree.root.children.len(), 1);

    // child: cycleB
    let cycle_b = &tree.root.children[0];
    assert_eq!(cycle_b.def.name, "cycleB");
    assert_eq!(cycle_b.children.len(), 1);

    // grandchild: cycleA (should be present but not recurse infinitely)
    let cycle_a_2 = &cycle_b.children[0];
    assert_eq!(cycle_a_2.def.name, "cycleA");
    
    // Depending on implementation, it might stop here or continue until max depth.
    // The current implementation stops recursion if node is in current path.
    // So cycleA -> cycleB -> cycleA (stop)
    assert_eq!(cycle_a_2.children.len(), 0);
    assert!(!cycle_a_2.truncated); // Not truncated by depth, but by cycle check
}

#[test]
fn test_js_multiple_callers() {
    let base_dir = get_fixtures_dir();
    // Trace backward from "target"
    let query = TraceQuery::new("target".to_string(), TraceDirection::Backward, 3)
        .with_base_dir(base_dir);

    let result = run_trace(query).expect("Trace failed");
    assert!(result.is_some());
    let tree = result.unwrap();

    assert_eq!(tree.root.def.name, "target");
    // Should have 2 callers: caller1 and caller2
    assert_eq!(tree.root.children.len(), 2);
    
    let caller_names: Vec<String> = tree.root.children.iter()
        .map(|n| n.def.name.clone())
        .collect();
        
    assert!(caller_names.contains(&"caller1".to_string()));
    assert!(caller_names.contains(&"caller2".to_string()));
}

#[test]
fn test_ruby_nested_calls() {
    let base_dir = get_fixtures_dir();
    let query = TraceQuery::new("rb_a".to_string(), TraceDirection::Forward, 3)
        .with_base_dir(base_dir);

    let result = run_trace(query).expect("Trace failed");
    assert!(result.is_some());
    let tree = result.unwrap();

    assert_eq!(tree.root.def.name, "rb_a");
    assert_eq!(tree.root.children[0].def.name, "rb_b");
    assert_eq!(tree.root.children[0].children[0].def.name, "rb_c");
}

#[test]
fn test_python_nested_calls() {
    let base_dir = get_fixtures_dir();
    let query = TraceQuery::new("py_a".to_string(), TraceDirection::Forward, 3)
        .with_base_dir(base_dir);

    let result = run_trace(query).expect("Trace failed");
    assert!(result.is_some());
    let tree = result.unwrap();

    assert_eq!(tree.root.def.name, "py_a");
    assert_eq!(tree.root.children[0].def.name, "py_b");
    assert_eq!(tree.root.children[0].children[0].def.name, "py_c");
}
