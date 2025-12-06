use cs::trace::{CallExtractor, FunctionDef, FunctionFinder};
use std::path::PathBuf;

#[test]
fn test_extract_calls_from_js_function() {
    let func = FunctionDef {
        name: "processData".to_string(),
        file: PathBuf::from("tests/fixtures/call-graph/sample.js"),
        line: 3,
        body: "".to_string(),
    };

    let extractor = CallExtractor::new(std::env::current_dir().unwrap());
    let calls = extractor.extract_calls(&func).unwrap();

    // Should find validateInput, transformData, and saveToDatabase
    assert!(calls.contains(&"validateInput".to_string()));
    assert!(calls.contains(&"transformData".to_string()));
    assert!(calls.contains(&"saveToDatabase".to_string()));

    // Should NOT include the function itself
    assert!(!calls.contains(&"processData".to_string()));
}

#[test]
fn test_extract_calls_filters_keywords() {
    let func = FunctionDef {
        name: "validateInput".to_string(),
        file: PathBuf::from("tests/fixtures/call-graph/sample.js"),
        line: 10,
        body: "".to_string(),
    };

    let extractor = CallExtractor::new(std::env::current_dir().unwrap());
    let calls = extractor.extract_calls(&func).unwrap();

    // Should filter out 'if' keyword
    assert!(!calls.contains(&"if".to_string()));

    // Should filter out 'throw' keyword
    assert!(!calls.contains(&"throw".to_string()));
}

#[test]
fn test_extract_calls_from_ruby_function() {
    let func = FunctionDef {
        name: "process_order".to_string(),
        file: PathBuf::from("tests/fixtures/call-graph/sample.rb"),
        line: 3,
        body: "".to_string(),
    };

    let extractor = CallExtractor::new(std::env::current_dir().unwrap());
    let calls = extractor.extract_calls(&func).unwrap();

    // Should find called functions
    assert!(calls.contains(&"validate_order".to_string()));
    assert!(calls.contains(&"calculate_total".to_string()));
    assert!(calls.contains(&"send_confirmation".to_string()));
}

#[test]
fn test_find_callers_of_function() {
    let extractor = CallExtractor::new(std::env::current_dir().unwrap());

    // Find all callers of processData function
    let callers = extractor.find_callers("processData").unwrap();

    // Should find at least handleClick calling processData
    assert!(!callers.is_empty());

    // Check that we found a caller
    let caller_names: Vec<String> = callers.iter().map(|c| c.caller_name.clone()).collect();
    assert!(caller_names.iter().any(|name| name == "handleClick"));
}

#[test]
fn test_find_callers_of_ruby_function() {
    let extractor = CallExtractor::new(std::env::current_dir().unwrap());

    // Find all callers of process_order function
    let callers = extractor.find_callers("process_order").unwrap();

    // Should find at least one caller
    assert!(!callers.is_empty());

    // Check that we found the main function
    let caller_names: Vec<String> = callers.iter().map(|c| c.caller_name.clone()).collect();
    assert!(caller_names.iter().any(|name| name == "main"));
}

#[test]
fn test_function_finder_js() {
    let mut finder = FunctionFinder::new(std::env::current_dir().unwrap());
    let defs = finder.find_definition("processData").unwrap();

    assert!(!defs.is_empty());
    assert_eq!(defs[0].name, "processData");
    assert_eq!(defs[0].line, 3);
}

#[test]
fn test_function_finder_ruby() {
    let mut finder = FunctionFinder::new(std::env::current_dir().unwrap());
    let defs = finder.find_definition("process_order").unwrap();

    assert!(!defs.is_empty());
    assert_eq!(defs[0].name, "process_order");
    assert_eq!(defs[0].line, 3);
}

#[test]
fn test_keywords_filtered_correctly() {
    let extractor = CallExtractor::new(std::env::current_dir().unwrap());

    // Verify common keywords are in the filter list
    assert!(extractor.keywords.contains("if"));
    assert!(extractor.keywords.contains("for"));
    assert!(extractor.keywords.contains("while"));
    assert!(extractor.keywords.contains("print"));
}

#[test]
fn test_caller_info_structure() {
    let extractor = CallExtractor::new(std::env::current_dir().unwrap());
    let callers = extractor.find_callers("validateInput").unwrap();

    // Verify CallerInfo contains required fields
    for caller in &callers {
        assert!(!caller.caller_name.is_empty());
        assert!(!caller.file.as_os_str().is_empty());
        assert!(caller.line > 0);
    }
}
