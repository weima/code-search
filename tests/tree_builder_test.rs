use cs::{run_search, ReferenceTreeBuilder, SearchQuery};
use std::path::PathBuf;

#[test]
fn test_build_tree_from_search_result() {
    let query = SearchQuery::new("add new".to_string())
        .with_base_dir(PathBuf::from("tests/fixtures/rails-app"));

    let result = run_search(query).expect("Search should succeed");
    let tree = ReferenceTreeBuilder::build(&result);

    // Verify tree structure
    assert_eq!(tree.root.content, "add new");
    assert!(tree.has_results(), "Tree should have results");

    println!("\n=== Tree Structure ===");
    println!("Root: {}", tree.root.content);
    println!("Total nodes: {}", tree.node_count());
    println!("Max depth: {}", tree.max_depth());
    println!("Translation entries: {}", tree.root.children.len());
}

#[test]
fn test_tree_builder_with_multiple_frameworks() {
    let query =
        SearchQuery::new("add new".to_string()).with_base_dir(PathBuf::from("tests/fixtures"));

    let result = run_search(query).expect("Search should succeed");
    let tree = ReferenceTreeBuilder::build(&result);

    println!("\n=== Multi-Framework Tree ===");
    println!("Query: {}", tree.root.content);
    println!("Translation entries found: {}", tree.root.children.len());

    for (i, translation) in tree.root.children.iter().enumerate() {
        println!("\nTranslation {}:", i + 1);
        println!("  Content: {}", translation.content);
        if let Some(loc) = &translation.location {
            println!("  Location: {}:{}", loc.file.display(), loc.line);
        }

        for key_path in &translation.children {
            println!("  Key: {}", key_path.content);
            println!("  Code references: {}", key_path.children.len());

            for (j, code_ref) in key_path.children.iter().take(3).enumerate() {
                if let Some(loc) = &code_ref.location {
                    println!("    {}. {}:{}", j + 1, loc.file.display(), loc.line);
                    println!("       {}", code_ref.content.trim());
                }
            }

            if key_path.children.len() > 3 {
                println!("    ... and {} more", key_path.children.len() - 3);
            }
        }
    }

    assert!(tree.has_results());
}

#[test]
fn test_tree_builder_hierarchy() {
    let query = SearchQuery::new("add new".to_string())
        .with_base_dir(PathBuf::from("tests/fixtures/rails-app"));

    let result = run_search(query).expect("Search should succeed");
    let tree = ReferenceTreeBuilder::build(&result);

    // Verify hierarchy: Root -> Translation -> KeyPath -> CodeRef
    assert!(
        !tree.root.children.is_empty(),
        "Should have translation nodes"
    );

    let translation = &tree.root.children[0];
    assert_eq!(translation.node_type, cs::NodeType::Translation);

    if !translation.children.is_empty() {
        let key_path = &translation.children[0];
        assert_eq!(key_path.node_type, cs::NodeType::KeyPath);

        if !key_path.children.is_empty() {
            let code_ref = &key_path.children[0];
            assert_eq!(code_ref.node_type, cs::NodeType::CodeRef);
        }
    }
}

#[test]
fn test_tree_builder_preserves_locations() {
    let query = SearchQuery::new("add new".to_string())
        .with_base_dir(PathBuf::from("tests/fixtures/rails-app"));

    let result = run_search(query).expect("Search should succeed");
    let tree = ReferenceTreeBuilder::build(&result);

    // Check that translation nodes have locations
    for translation in &tree.root.children {
        assert!(
            translation.location.is_some(),
            "Translation node should have location"
        );

        // Check that code reference nodes have locations
        for key_path in &translation.children {
            for code_ref in &key_path.children {
                assert!(
                    code_ref.location.is_some(),
                    "Code reference node should have location"
                );
            }
        }
    }
}

#[test]
fn test_tree_builder_with_no_results() {
    let query = SearchQuery::new("nonexistent xyz abc".to_string())
        .with_base_dir(PathBuf::from("tests/fixtures/rails-app"));

    let result = run_search(query).expect("Search should succeed");
    let tree = ReferenceTreeBuilder::build(&result);

    assert_eq!(tree.root.content, "nonexistent xyz abc");
    assert!(!tree.has_results(), "Tree should have no results");
    assert_eq!(tree.node_count(), 1); // Only root node
    assert_eq!(tree.max_depth(), 1);
}

#[test]
fn test_end_to_end_with_tree_builder() {
    println!("\n=== End-to-End with Tree Builder ===\n");

    let search_text = "add new";
    let query = SearchQuery::new(search_text.to_string())
        .with_base_dir(PathBuf::from("tests/fixtures/rails-app"));

    println!("1. Running search for: '{}'", search_text);
    let result = run_search(query).expect("Search should succeed");

    println!(
        "2. Found {} translation entries",
        result.translation_entries.len()
    );
    println!("3. Found {} code references", result.code_references.len());

    println!("4. Building reference tree...");
    let tree = ReferenceTreeBuilder::build(&result);

    println!("5. Tree statistics:");
    println!("   - Total nodes: {}", tree.node_count());
    println!("   - Max depth: {}", tree.max_depth());
    println!("   - Has results: {}", tree.has_results());

    println!("\n6. Tree structure:");
    print_tree_node(&tree.root, 0);

    println!("\n✅ End-to-end workflow with tree builder complete!");

    assert!(tree.has_results());
    assert!(tree.node_count() > 1);
}

fn print_tree_node(node: &cs::TreeNode, depth: usize) {
    let indent = "  ".repeat(depth);

    print!("{}", indent);
    match node.node_type {
        cs::NodeType::Root => print!("🔍 "),
        cs::NodeType::Translation => print!("📄 "),
        cs::NodeType::KeyPath => print!("🔑 "),
        cs::NodeType::CodeRef => print!("💻 "),
    }

    print!("{}", node.content);

    if let Some(loc) = &node.location {
        print!(" ({}:{})", loc.file.display(), loc.line);
    }

    println!();

    for child in &node.children {
        print_tree_node(child, depth + 1);
    }
}
