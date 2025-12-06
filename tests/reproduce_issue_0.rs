use cs::{run_search, SearchQuery};
use std::fs;

use tempfile::TempDir;

#[test]
fn test_reproduce_issue_0() {
    // Setup a temporary directory simulating a project
    let temp_dir = TempDir::new().unwrap();
    let base_dir = temp_dir.path().to_path_buf();

    // 1. Create a translation file with deep nesting
    // Key: en.app.production_and_service.new_page.labels.add_new
    let locales_dir = base_dir.join("locales");
    fs::create_dir(&locales_dir).unwrap();
    fs::write(
        locales_dir.join("en.yml"),
        r#"
en-US:
  app:
    production_and_service:
      new_page:
        labels:
          add_new: "Add New"
"#,
    )
    .unwrap();

    // 2. Create a source file with usage that strips 'en.app'
    // Usage: production_and_service.new_page.labels.add_new
    let src_dir = base_dir.join("src");
    fs::create_dir(&src_dir).unwrap();
    fs::write(
        src_dir.join("index.js"),
        "console.log(i18n.t('production_and_service.new_page.labels.add_new'));",
    )
    .unwrap();

    // 4. Run the search
    let query = SearchQuery::new("Add New".to_string()).with_base_dir(base_dir.clone());

    let result = run_search(query).unwrap();

    // 5. Verify results
    println!(
        "Found {} translation entries",
        result.translation_entries.len()
    );
    for entry in &result.translation_entries {
        println!("Entry: {} = {}", entry.key, entry.value);
    }

    println!("Found {} code references", result.code_references.len());
    for reference in &result.code_references {
        println!("Ref: {:?} -> {}", reference.file, reference.key_path);
    }

    // We expect to find the translation entry
    assert_eq!(
        result.translation_entries.len(),
        1,
        "Should find translation entry"
    );
    assert_eq!(
        result.translation_entries[0].key,
        "en-US.app.production_and_service.new_page.labels.add_new"
    );

    // We expect to find the code reference
    // This will fail if generate_partial_keys doesn't handle stripping 'en.app'
    assert!(
        !result.code_references.is_empty(),
        "Should find code reference"
    );
    assert_eq!(
        result.code_references[0].key_path,
        "production_and_service.new_page.labels.add_new"
    );
}
