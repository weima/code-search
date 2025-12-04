use cs::{run_search, SearchQuery};
use std::fs::File;
use std::io::Write;
use tempfile::TempDir;

#[test]
fn test_json_translation_support() {
    // Setup fixtures
    let temp_dir = TempDir::new().unwrap();
    let base_dir = temp_dir.path().to_path_buf();

    // 1. JSON translation file
    let json_file = base_dir.join("en.json");
    let mut f = File::create(&json_file).unwrap();
    writeln!(
        f,
        r#"{{
        "common": {{
            "save": "Save Changes",
            "cancel": "Cancel"
        }},
        "errors": {{
            "not_found": "Item not found"
        }}
    }}"#
    )
    .unwrap();

    // 2. Code file using the key
    let code_file = base_dir.join("Button.js");
    let mut f = File::create(&code_file).unwrap();
    writeln!(f, "const label = t('common.save');").unwrap();

    // Search for "Save Changes"
    let query = SearchQuery::new("Save Changes".to_string()).with_base_dir(base_dir.clone());

    let result = run_search(query).expect("Search failed");

    // Verify translation entry found
    assert!(
        !result.translation_entries.is_empty(),
        "Should find translation entry"
    );
    let entry = &result.translation_entries[0];
    assert_eq!(entry.key, "common.save");
    assert_eq!(entry.value, "Save Changes");
    assert_eq!(entry.file, json_file);

    // Verify code reference found
    assert!(
        !result.code_references.is_empty(),
        "Should find code reference"
    );
    let code_ref = &result.code_references[0];
    assert_eq!(code_ref.key_path, "common.save");
    assert_eq!(code_ref.file, code_file);
}
