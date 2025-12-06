use cs::{run_search, SearchQuery};
use std::path::PathBuf;

#[test]
fn test_real_world_mastodon_fixture() {
    // Path to our fixture
    let mut base_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    base_dir.push("tests/fixtures/real_world/mastodon");

    // 1. Search for "Post" (mapped to 'compose_form.publish')
    let query = SearchQuery::new("Post".to_string()).with_base_dir(base_dir.clone());

    let result = run_search(query).unwrap();

    assert!(
        !result.translation_entries.is_empty(),
        "Should find at least 'compose_form.publish' translation"
    );

    // Check if we found the correct key
    let found_target_key = result
        .translation_entries
        .iter()
        .any(|e| e.key == "compose_form.publish");
    assert!(found_target_key, "Should find 'compose_form.publish' key");

    // Should find usage in compose_form.js
    assert!(
        !result.code_references.is_empty(),
        "Should find code reference for 'Post'"
    );

    // We might have direct matches (key_path="Post") and traced matches (key_path="compose_form.publish")
    // Find the traced match
    let traced_ref = result
        .code_references
        .iter()
        .find(|r| r.key_path == "compose_form.publish");
    assert!(
        traced_ref.is_some(),
        "Should find traced reference for 'compose_form.publish'"
    );

    let ref_path = traced_ref.unwrap().file.to_string_lossy();
    assert!(ref_path.contains("compose_form.js"));

    // 2. Search for "What is on your mind?" (mapped to 'compose_form.placeholder')
    let query_placeholder =
        SearchQuery::new("What is on your mind?".to_string()).with_base_dir(base_dir.clone());

    let result_placeholder = run_search(query_placeholder).unwrap();

    assert_eq!(result_placeholder.translation_entries.len(), 1);
    assert!(!result_placeholder.code_references.is_empty());
    assert_eq!(
        result_placeholder.code_references[0].key_path,
        "compose_form.placeholder"
    );
}

#[test]
fn test_real_world_rails_fixture() {
    // Path to our fixture
    let mut base_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    base_dir.push("tests/fixtures/real_world/rails");

    // 1. Search for "Your account has been activated" (mapped to 'activation.activated')
    let query = SearchQuery::new("Your account has been activated".to_string())
        .with_base_dir(base_dir.clone());

    let result = run_search(query).unwrap();

    // Should find the translation entry
    assert!(
        !result.translation_entries.is_empty(),
        "Should find 'activation.activated' translation"
    );

    // Check key
    let entry = result
        .translation_entries
        .iter()
        .find(|e| e.key == "activation.activated");
    assert!(entry.is_some(), "Should find 'activation.activated' key");

    // Should find usage in users_controller.rb
    // Note: In Rails, keys often omit 'en.' prefix in usage, but the file has 'en.activation.activated'
    // The tool should handle this via generate_partial_keys
    let traced_ref = result
        .code_references
        .iter()
        .find(|r| r.key_path == "activation.activated");
    assert!(
        traced_ref.is_some(),
        "Should find traced reference for 'activation.activated'"
    );

    let ref_path = traced_ref.unwrap().file.to_string_lossy();
    assert!(ref_path.contains("users_controller.rb"));
}
