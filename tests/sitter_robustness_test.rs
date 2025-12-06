use cs::trace::FunctionFinder;
use std::fs;
use tempfile::TempDir;

#[test]
fn test_sitter_rust_comments() {
    let temp_dir = TempDir::new().unwrap();
    let base_dir = temp_dir.path().to_path_buf();

    // Create a Rust file with commented out function
    let file_path = base_dir.join("test.rs");
    fs::write(
        &file_path,
        r#"
        // fn fake_function() {}
        /* 
           fn another_fake() {} 
        */
        fn real_function() {}
        "#,
    )
    .unwrap();

    let mut finder = FunctionFinder::new(base_dir);

    // Should NOT find fake_function (regex would find it)
    let fake = finder.find_function("fake_function");
    assert!(
        fake.is_none(),
        "Should not find function in comment: fake_function"
    );

    // Should find real_function
    let real = finder.find_function("real_function");
    assert!(real.is_some(), "Should find real_function");
    let real = real.unwrap();
    assert_eq!(real.name, "real_function");
}

#[test]
fn test_sitter_python_comments() {
    let temp_dir = TempDir::new().unwrap();
    let base_dir = temp_dir.path().to_path_buf();

    let file_path = base_dir.join("test.py");
    fs::write(
        &file_path,
        r#"
# def fake_function(): pass
def real_function():
    pass
        "#,
    )
    .unwrap();

    let mut finder = FunctionFinder::new(base_dir);

    let fake = finder.find_function("fake_function");
    assert!(
        fake.is_none(),
        "Should not find function in comment: fake_function"
    );

    let real = finder.find_function("real_function");
    assert!(real.is_some(), "Should find real_function");
}

#[test]
fn test_sitter_js_comments() {
    let temp_dir = TempDir::new().unwrap();
    let base_dir = temp_dir.path().to_path_buf();

    let file_path = base_dir.join("test.js");
    fs::write(
        &file_path,
        r#"
// function fakeFunction() {}
function realFunction() {}
        "#,
    )
    .unwrap();

    let mut finder = FunctionFinder::new(base_dir);

    let fake = finder.find_function("fakeFunction");
    assert!(
        fake.is_none(),
        "Should not find function in comment: fakeFunction"
    );

    let real = finder.find_function("realFunction");
    assert!(real.is_some(), "Should find realFunction");
}
