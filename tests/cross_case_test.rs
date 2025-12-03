use cs::{run_trace, TraceDirection, TraceQuery};
use std::fs::File;
use std::io::Write;
use tempfile::TempDir;

#[test]
fn test_cross_case_function_search() {
    // Setup fixtures
    let temp_dir = TempDir::new().unwrap();
    let base_dir = temp_dir.path().to_path_buf();

    // 1. Ruby file defining snake_case function
    let ruby_file = base_dir.join("user_service.rb");
    let mut f = File::create(&ruby_file).unwrap();
    writeln!(f, "def process_user_data(user)").unwrap();
    writeln!(f, "  # implementation").unwrap();
    writeln!(f, "end").unwrap();

    // 2. JS file calling camelCase version
    let js_file = base_dir.join("api_client.js");
    let mut f = File::create(&js_file).unwrap();
    writeln!(f, "function syncUsers() {{").unwrap();
    writeln!(f, "  api.processUserData(user);").unwrap();
    writeln!(f, "}}").unwrap();

    // 3. Python file calling snake_case version
    let py_file = base_dir.join("script.py");
    let mut f = File::create(&py_file).unwrap();
    writeln!(f, "def main():").unwrap();
    writeln!(f, "    service.process_user_data(u)").unwrap();

    // Case 1: Search for camelCase input "processUserData"
    // Should find definition "process_user_data" and both callers
    let query = TraceQuery::new("processUserData".to_string(), TraceDirection::Backward, 1)
        .with_base_dir(base_dir.clone());

    let result = run_trace(query).expect("Trace failed");
    assert!(result.is_some(), "Should find function definition");

    let tree = result.unwrap();
    let root = &tree.root;

    // Verify definition found (original name in file)
    assert_eq!(root.def.name, "process_user_data");
    assert_eq!(root.def.file, ruby_file);

    // Verify callers found
    assert_eq!(root.children.len(), 2, "Should find 2 callers");

    let caller_names: Vec<String> = root.children.iter().map(|c| c.def.name.clone()).collect();

    assert!(
        caller_names.contains(&"syncUsers".to_string()),
        "Should find JS caller"
    );
    assert!(
        caller_names.contains(&"main".to_string()),
        "Should find Python caller"
    );

    // Case 2: Search for snake_case input "process_user_data"
    // Should find same results
    let query = TraceQuery::new("process_user_data".to_string(), TraceDirection::Backward, 1)
        .with_base_dir(base_dir);

    let result = run_trace(query).expect("Trace failed");
    assert!(result.is_some(), "Should find function definition");
    let tree = result.unwrap();
    assert_eq!(tree.root.def.name, "process_user_data");
    assert_eq!(tree.root.children.len(), 2);
}
