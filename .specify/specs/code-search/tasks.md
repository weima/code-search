# Implementation Tasks: Code Search CLI Tool

## Metadata

- **Feature**: Code Search Core Functionality
- **Spec**: `.specify/specs/code-search/spec.md`
- **Plan**: `.specify/specs/code-search/plan.md`
- **Generated**: 2025-11-28

## Task Breakdown by User Story

### US-1: Basic Text-to-Code Trace

#### Task 1.1: Project Setup & Environment
**Priority**: P1 | **Estimate**: 4 hours | **Dependencies**: None

**Acceptance Criteria**:
- [ ] Cargo project initialized
- [ ] Cargo.toml configured with all dependencies
- [ ] README.md has basic structure
- [ ] LICENSE file present (Apache 2.0)
- [ ] .gitignore configured for Rust

**Subtasks**:
1. Run `cargo init --bin code-search`
2. Add dependencies to Cargo.toml:
   ```toml
   [dependencies]
   clap = { version = "4.4", features = ["derive"] }
   serde = { version = "1.0", features = ["derive"] }
   serde_yaml = "0.9"
   regex = "1.10"
   anyhow = "1.0"
   thiserror = "1.0"
   walkdir = "2.4"

   [dev-dependencies]
   assert_cmd = "2.0"
   predicates = "3.0"
   tempfile = "3.8"
   criterion = "0.5"
   ```
3. Create basic directory structure (src/search, src/parse, src/tree, src/output)
4. Add .gitignore with Rust entries
5. Verify `cargo build` succeeds

**Test Plan**:
- `cargo build` completes without errors
- `cargo test` runs (even with no tests)
- Project structure matches plan.md

---

#### Task 1.2: Test Fixtures Creation
**Priority**: P1 | **Estimate**: 3 hours | **Dependencies**: 1.1

**Acceptance Criteria**:
- [ ] Rails test project with YAML i18n files
- [ ] React test project with JSON i18n files
- [ ] Vue test project with JSON i18n files
- [ ] Each fixture has clear search targets

**Subtasks**:
1. Create `tests/fixtures/rails-app/` directory
2. Add `config/locales/en.yml`:
   ```yaml
   en:
     invoice:
       labels:
         add_new: 'add new'
         edit: 'edit invoice'
   ```
3. Add `app/components/invoices.ts` with `I18n.t('invoice.labels.add_new')`
4. Repeat for React project (react-app/) with `i18n.t()` pattern
5. Repeat for Vue project (vue-app/) with `$t()` pattern

**Test Plan**:
- Manual search verifies fixtures contain expected patterns
- Each fixture has at least 2 searchable text strings

---

#### Task 1.3: CLI Argument Parsing
**Priority**: P1 | **Estimate**: 4 hours | **Dependencies**: 1.1

**Acceptance Criteria**:
- [ ] CLI accepts search text as argument
- [ ] `--help` displays usage information
- [ ] `--version` displays version
- [ ] Empty search text shows error

**Subtasks**:
1. Create `src/main.rs` with clap derive:
   ```rust
   use clap::Parser;

   #[derive(Parser)]
   #[command(name = "cs")]
   #[command(about = "Code search tool for i18n tracing", long_about = None)]
   struct Cli {
       /// Text to search for
       search_text: String,

       /// Case-sensitive search
       #[arg(short, long)]
       case_sensitive: bool,
   }
   ```
2. Implement main() to parse args
3. Validate search_text is non-empty
4. Add `--version` with version from Cargo.toml

**Test Plan**:
```rust
#[test]
fn test_cli_help() {
    Command::cargo_bin("cs").unwrap()
        .arg("--help")
        .assert()
        .success();
}

#[test]
fn test_cli_requires_search_text() {
    Command::cargo_bin("cs").unwrap()
        .assert()
        .failure();
}
```

---

#### Task 1.4: Ripgrep Wrapper Implementation
**Priority**: P1 | **Estimate**: 6 hours | **Dependencies**: 1.1, 1.3

**Acceptance Criteria**:
- [ ] Can execute ripgrep via std::process::Command
- [ ] Parses ripgrep output into Match structs
- [ ] Handles ripgrep not installed gracefully
- [ ] Respects .gitignore by default

**Subtasks**:
1. Create `src/search/mod.rs` and `src/search/text_search.rs`
2. Define Match struct:
   ```rust
   pub struct Match {
       pub file: PathBuf,
       pub line: usize,
       pub content: String,
   }
   ```
3. Implement TextSearcher:
   ```rust
   pub struct TextSearcher;

   impl TextSearcher {
       pub fn search(&self, text: &str) -> Result<Vec<Match>> {
           // Check if ripgrep exists in PATH
           // Execute: rg --line-number --no-heading "text"
           // Parse output (format: file:line:content)
           // Return Vec<Match>
       }
   }
   ```
4. Add error handling for ripgrep not found
5. Write unit tests

**Test Plan**:
```rust
#[test]
fn test_ripgrep_finds_matches() {
    let searcher = TextSearcher;
    let matches = searcher.search("add new").unwrap();
    assert!(!matches.is_empty());
}

#[test]
fn test_ripgrep_not_found() {
    // Mock PATH without ripgrep
    // Verify error type is SearchError::RipgrepNotFound
}
```

---

#### Task 1.5: YAML Parser Implementation
**Priority**: P1 | **Estimate**: 8 hours | **Dependencies**: 1.1

**Acceptance Criteria**:
- [ ] Parses YAML files with serde_yaml
- [ ] Flattens nested structures into dot-notation
- [ ] Handles malformed YAML gracefully
- [ ] Extracts line numbers for each key-value pair

**Subtasks**:
1. Create `src/parse/mod.rs`, `src/parse/yaml_parser.rs`, `src/parse/translation.rs`
2. Define TranslationEntry struct:
   ```rust
   pub struct TranslationEntry {
       pub file: PathBuf,
       pub line: usize,
       pub key_path: String,
       pub value: String,
       pub language: String,
   }
   ```
3. Implement YamlParser:
   ```rust
   pub struct YamlParser;

   impl YamlParser {
       pub fn parse_file(&self, path: &Path) -> Result<Vec<TranslationEntry>> {
           // Read file
           // Parse with serde_yaml::from_str
           // Flatten nested structures
           // Return entries
       }

       fn flatten(&self, value: &Value, prefix: &str) -> Vec<(String, String)> {
           // Recursive function to flatten YAML
           // Build dot-notation keys
       }
   }
   ```
4. Handle errors (file not found, invalid YAML)
5. Write unit tests for nested YAML structures

**Test Plan**:
```rust
#[test]
fn test_parse_nested_yaml() {
    let yaml = r#"
    en:
      invoice:
        labels:
          add_new: 'add new'
    "#;
    let entries = YamlParser.parse_str(yaml).unwrap();
    assert_eq!(entries[0].key_path, "en.invoice.labels.add_new");
    assert_eq!(entries[0].value, "add new");
}

#[test]
fn test_malformed_yaml_error() {
    let yaml = "invalid: [yaml";
    let result = YamlParser.parse_str(yaml);
    assert!(result.is_err());
}
```

---

#### Task 1.6: Key Path Extraction
**Priority**: P1 | **Estimate**: 4 hours | **Dependencies**: 1.5

**Acceptance Criteria**:
- [ ] Identifies which translation entries match search text
- [ ] Extracts full dot-notation key paths
- [ ] Handles multiple language files (en, fr, etc.)
- [ ] Associates file and line number with each key

**Subtasks**:
1. Create `src/parse/key_extractor.rs`
2. Implement KeyExtractor:
   ```rust
   pub struct KeyExtractor;

   impl KeyExtractor {
       pub fn extract(&self, entries: &[TranslationEntry], text: &str) -> Vec<TranslationEntry> {
           // Filter entries where value contains text
           // Return matching entries with key_path populated
       }
   }
   ```
3. Add case-insensitive matching
4. Write unit tests

**Test Plan**:
```rust
#[test]
fn test_extract_key_for_text() {
    let entries = vec![
        TranslationEntry { key_path: "invoice.labels.add_new".into(), value: "add new".into(), .. },
        TranslationEntry { key_path: "user.labels.delete".into(), value: "delete".into(), .. },
    ];
    let matches = KeyExtractor.extract(&entries, "add new");
    assert_eq!(matches.len(), 1);
    assert_eq!(matches[0].key_path, "invoice.labels.add_new");
}
```

---

#### Task 1.7: Pattern Matching Implementation
**Priority**: P1 | **Estimate**: 8 hours | **Dependencies**: 1.4, 1.6

**Acceptance Criteria**:
- [ ] Searches for translation key usage in code
- [ ] Supports Ruby patterns (I18n.t, t)
- [ ] Supports JavaScript patterns (i18n.t, t, $t)
- [ ] Extracts file, line, and surrounding context

**Subtasks**:
1. Create `src/search/pattern_match.rs`
2. Create `src/config/patterns.rs` with regex definitions:
   ```rust
   pub fn default_patterns() -> Vec<Regex> {
       vec![
           Regex::new(r#"I18n\.t\(['"]([^'"]+)['"]\)"#).unwrap(),
           Regex::new(r#"\bt\(['"]([^'"]+)['"]\)"#).unwrap(),
           Regex::new(r#"\$t\(['"]([^'"]+)['"]\)"#).unwrap(),
           Regex::new(r#"i18n\.t\(['"]([^'"]+)['"]\)"#).unwrap(),
       ]
   }
   ```
3. Implement PatternMatcher:
   ```rust
   pub struct PatternMatcher {
       searcher: TextSearcher,
       patterns: Vec<Regex>,
   }

   impl PatternMatcher {
       pub fn find_usages(&self, key_path: &str) -> Result<Vec<CodeReference>> {
           // Search for key_path using TextSearcher
           // Apply regex patterns to filter matches
           // Extract CodeReference structs
       }
   }
   ```
4. Define CodeReference struct
5. Write unit tests for each pattern

**Test Plan**:
```rust
#[test]
fn test_find_ruby_pattern() {
    let matcher = PatternMatcher::new();
    let refs = matcher.find_usages("invoice.labels.add_new").unwrap();
    assert!(refs.iter().any(|r| r.pattern.contains("I18n.t")));
}

#[test]
fn test_find_vue_pattern() {
    let matcher = PatternMatcher::new();
    let refs = matcher.find_usages("invoice.labels.add_new").unwrap();
    assert!(refs.iter().any(|r| r.pattern.contains("$t")));
}
```

---

#### Task 1.8: Search Orchestrator
**Priority**: P1 | **Estimate**: 6 hours | **Dependencies**: 1.4, 1.5, 1.6, 1.7

**Acceptance Criteria**:
- [ ] Coordinates all search steps
- [ ] Calls modules in correct order
- [ ] Handles errors at each step
- [ ] Returns complete ReferenceTree

**Subtasks**:
1. Create `src/lib.rs` with run_search function:
   ```rust
   pub fn run_search(query: SearchQuery) -> Result<ReferenceTree> {
       // 1. Search for text
       let matches = TextSearcher.search(&query.text)?;

       // 2. Filter translation files
       let translation_files = filter_translation_files(&matches);

       // 3. Parse translations
       let entries = YamlParser.parse_files(&translation_files)?;

       // 4. Extract keys
       let keys = KeyExtractor.extract(&entries, &query.text);

       // 5. Find code references
       let mut code_refs = vec![];
       for entry in &keys {
           let refs = PatternMatcher.find_usages(&entry.key_path)?;
           code_refs.extend(refs);
       }

       // 6. Build tree
       let tree = ReferenceTreeBuilder::build(&query, &keys, &code_refs);

       Ok(tree)
   }
   ```
2. Implement filter_translation_files helper
3. Add error handling for each step
4. Write integration test

**Test Plan**:
```rust
#[test]
fn test_end_to_end_search() {
    let query = SearchQuery { text: "add new".into(), .. };
    let tree = run_search(query).unwrap();
    assert!(!tree.root.children.is_empty());
}
```

---

### US-2: Clear Visual Output

#### Task 2.1: Tree Node Data Structures
**Priority**: P1 | **Estimate**: 3 hours | **Dependencies**: 1.8

**Acceptance Criteria**:
- [ ] TreeNode struct defined with type, content, location, children
- [ ] ReferenceTree struct wraps root node
- [ ] NodeType enum includes all node types

**Subtasks**:
1. Create `src/tree/mod.rs` and `src/tree/node.rs`
2. Define data structures:
   ```rust
   #[derive(Debug, Clone)]
   pub enum NodeType {
       Root,
       Translation,
       KeyPath,
       CodeRef,
   }

   #[derive(Debug, Clone)]
   pub struct Location {
       pub file: PathBuf,
       pub line: usize,
   }

   #[derive(Debug, Clone)]
   pub struct TreeNode {
       pub node_type: NodeType,
       pub content: String,
       pub location: Option<Location>,
       pub children: Vec<TreeNode>,
   }

   #[derive(Debug)]
   pub struct ReferenceTree {
       pub root: TreeNode,
   }
   ```
3. Add constructors and helper methods
4. Write unit tests

**Test Plan**:
```rust
#[test]
fn test_create_tree_node() {
    let node = TreeNode {
        node_type: NodeType::Root,
        content: "add new".into(),
        location: None,
        children: vec![],
    };
    assert_eq!(node.content, "add new");
}
```

---

#### Task 2.2: Reference Tree Builder
**Priority**: P1 | **Estimate**: 6 hours | **Dependencies**: 2.1, 1.8

**Acceptance Criteria**:
- [ ] Builds tree from search results
- [ ] Creates hierarchical structure (root → translation → key → code)
- [ ] Handles multiple branches (multiple matches)
- [ ] Correctly associates locations

**Subtasks**:
1. Create `src/tree/builder.rs`
2. Implement ReferenceTreeBuilder:
   ```rust
   pub struct ReferenceTreeBuilder;

   impl ReferenceTreeBuilder {
       pub fn build(
           query: &SearchQuery,
           translations: &[TranslationEntry],
           code_refs: &[CodeReference]
       ) -> ReferenceTree {
           // Create root node with search text
           let mut root = TreeNode {
               node_type: NodeType::Root,
               content: query.text.clone(),
               location: None,
               children: vec![],
           };

           // Group code_refs by translation entry
           for translation in translations {
               let translation_node = self.build_translation_node(translation);
               let key_node = self.build_key_node(translation);

               // Find code refs for this key
               let matching_refs: Vec<_> = code_refs
                   .iter()
                   .filter(|r| r.key_path == translation.key_path)
                   .collect();

               // Build code ref nodes
               for code_ref in matching_refs {
                   let code_node = self.build_code_node(code_ref);
                   key_node.children.push(code_node);
               }

               translation_node.children.push(key_node);
               root.children.push(translation_node);
           }

           ReferenceTree { root }
       }
   }
   ```
3. Implement helper methods (build_translation_node, etc.)
4. Write unit tests

**Test Plan**:
```rust
#[test]
fn test_build_tree_single_match() {
    let translations = vec![/* ... */];
    let code_refs = vec![/* ... */];
    let tree = ReferenceTreeBuilder::build(&query, &translations, &code_refs);

    assert_eq!(tree.root.children.len(), 1);
    assert_eq!(tree.root.children[0].node_type, NodeType::Translation);
}

#[test]
fn test_build_tree_multiple_code_refs() {
    // Test with 2+ code refs for same translation
    // Verify all code refs appear as children
}
```

---

#### Task 2.3: Tree Formatter Implementation
**Priority**: P1 | **Estimate**: 8 hours | **Dependencies**: 2.2

**Acceptance Criteria**:
- [ ] Formats tree with visual connectors (|, |->)
- [ ] Displays file paths and line numbers
- [ ] Fits within 80-column width
- [ ] Handles deep nesting (up to 10 levels)

**Subtasks**:
1. Create `src/output/mod.rs` and `src/output/formatter.rs`
2. Implement TreeFormatter:
   ```rust
   pub struct TreeFormatter {
       max_width: usize,
   }

   impl TreeFormatter {
       pub fn format(&self, tree: &ReferenceTree) -> String {
           let mut output = String::new();
           self.format_node(&tree.root, &mut output, "", true);
           output
       }

       fn format_node(&self, node: &TreeNode, output: &mut String, prefix: &str, is_last: bool) {
           // Render node with appropriate indentation and connectors
           // Format: 'content' at line X of file.ext
           // Recurse for children
       }
   }
   ```
3. Implement indentation logic
4. Add line truncation for long paths
5. Write formatting tests

**Test Plan**:
```rust
#[test]
fn test_format_tree_basic() {
    let tree = ReferenceTree { /* ... */ };
    let formatted = TreeFormatter::new(80).format(&tree);

    assert!(formatted.contains("'add new'"));
    assert!(formatted.contains("|->"));
    assert!(formatted.contains("at line"));
}

#[test]
fn test_format_fits_80_columns() {
    let tree = ReferenceTree { /* very long paths */ };
    let formatted = TreeFormatter::new(80).format(&tree);

    for line in formatted.lines() {
        assert!(line.len() <= 80, "Line too long: {}", line);
    }
}
```

---

#### Task 2.4: Output Display Integration
**Priority**: P1 | **Estimate**: 3 hours | **Dependencies**: 2.3

**Acceptance Criteria**:
- [ ] main.rs calls formatter and prints output
- [ ] Handles empty results gracefully
- [ ] Writes to stdout (not stderr)

**Subtasks**:
1. Update `src/main.rs`:
   ```rust
   fn main() -> Result<()> {
       let cli = Cli::parse();

       let query = SearchQuery {
           text: cli.search_text,
           case_sensitive: cli.case_sensitive,
           file_patterns: None,
       };

       let tree = run_search(query)?;
       let formatter = TreeFormatter::new(80);
       let output = formatter.format(&tree);

       println!("{}", output);

       Ok(())
   }
   ```
2. Handle errors with clear messages
3. Add exit codes (0 = success, 1 = error)

**Test Plan**:
```rust
#[test]
fn test_cli_displays_tree() {
    Command::cargo_bin("cs").unwrap()
        .arg("add new")
        .current_dir("tests/fixtures/rails-app")
        .assert()
        .success()
        .stdout(predicate::str::contains("invoice.labels.add_new"));
}
```

---

### US-3: Multiple Match Handling

#### Task 3.1: Multiple Translation File Support
**Priority**: P1 | **Estimate**: 4 hours | **Dependencies**: 1.5, 2.2

**Acceptance Criteria**:
- [ ] Searches across multiple language files (en.yml, fr.yml, etc.)
- [ ] Groups results by translation file
- [ ] Shows all matches in tree

**Subtasks**:
1. Update filter_translation_files to include all .yml files
2. Update tree builder to handle multiple translation nodes
3. Test with multi-language fixtures

**Test Plan**:
```rust
#[test]
fn test_multiple_language_files() {
    // Create fixture with en.yml and fr.yml
    let tree = run_search(query).unwrap();
    assert!(tree.root.children.len() >= 2); // One for each lang file
}
```

---

#### Task 3.2: Multiple Code Reference Display
**Priority**: P1 | **Estimate**: 3 hours | **Dependencies**: 2.3

**Acceptance Criteria**:
- [ ] Shows all code locations using a translation key
- [ ] Groups by translation entry
- [ ] Visual grouping is clear

**Subtasks**:
1. Update tree builder to attach all code refs to correct parent
2. Update formatter to handle multiple siblings
3. Test with fixture having 3+ code refs for same key

**Test Plan**:
```rust
#[test]
fn test_multiple_code_references() {
    // Fixture with same key used in 3 files
    let tree = run_search(query).unwrap();
    // Navigate to key node, verify it has 3 children
}
```

---

### US-4: Framework Pattern Detection

#### Task 4.1: React Pattern Support
**Priority**: P2 | **Estimate**: 4 hours | **Dependencies**: 1.7

**Acceptance Criteria**:
- [ ] Detects `useTranslation().t('key')`
- [ ] Detects `<Trans i18nKey="key" />`
- [ ] Works with React test fixture

**Subtasks**:
1. Add React patterns to patterns.rs:
   ```rust
   Regex::new(r#"useTranslation\(\)\.t\(['"]([^'"]+)['"]\)"#).unwrap(),
   Regex::new(r#"<Trans\s+i18nKey=['"]([^'"]+)['"]"#).unwrap(),
   ```
2. Update React test fixture with these patterns
3. Add integration test

**Test Plan**:
```rust
#[test]
fn test_react_usetranslation_pattern() {
    Command::cargo_bin("cs").unwrap()
        .arg("add new")
        .current_dir("tests/fixtures/react-app")
        .assert()
        .success()
        .stdout(predicate::str::contains("useTranslation"));
}
```

---

#### Task 4.2: Vue Pattern Support
**Priority**: P2 | **Estimate**: 4 hours | **Dependencies**: 1.7

**Acceptance Criteria**:
- [ ] Detects `{{ $t('key') }}`
- [ ] Detects `this.$t('key')`
- [ ] Works with Vue test fixture

**Subtasks**:
1. Add Vue patterns to patterns.rs:
   ```rust
   Regex::new(r#"\{\{\s*\$t\(['"]([^'"]+)['"]\)\s*\}\}"#).unwrap(),
   Regex::new(r#"this\.\$t\(['"]([^'"]+)['"]\)"#).unwrap(),
   ```
2. Update Vue test fixture
3. Add integration test

**Test Plan**:
```rust
#[test]
fn test_vue_template_pattern() {
    Command::cargo_bin("cs").unwrap()
        .arg("add new")
        .current_dir("tests/fixtures/vue-app")
        .assert()
        .success()
        .stdout(predicate::str::contains("$t"));
}
```

---

### US-5: Error Handling and Guidance

#### Task 5.1: Custom Error Types
**Priority**: P2 | **Estimate**: 4 hours | **Dependencies**: 1.1

**Acceptance Criteria**:
- [ ] All error types defined with thiserror
- [ ] Error messages are actionable
- [ ] Errors include context (file paths, etc.)

**Subtasks**:
1. Create `src/error.rs`:
   ```rust
   use thiserror::Error;

   #[derive(Debug, Error)]
   pub enum SearchError {
       #[error("ripgrep not found in PATH. Install from https://github.com/BurntSushi/ripgrep")]
       RipgrepNotFound,

       #[error("No translation files found containing '{0}'")]
       NoTranslationFiles(String),

       #[error("Failed to parse YAML file {file}: {reason}")]
       YamlParseError {
           file: PathBuf,
           reason: String,
       },

       #[error("Translation key '{key}' found but no code references detected")]
       NoCodeReferences { key: String },

       #[error("IO error: {0}")]
       Io(#[from] std::io::Error),
   }
   ```
2. Update all functions to return these error types
3. Add context with anyhow where appropriate

**Test Plan**:
```rust
#[test]
fn test_ripgrep_not_found_error() {
    // Mock environment without ripgrep
    let result = TextSearcher.search("test");
    assert!(matches!(result, Err(SearchError::RipgrepNotFound)));
}
```

---

#### Task 5.2: User-Friendly Error Messages
**Priority**: P2 | **Estimate**: 3 hours | **Dependencies**: 5.1

**Acceptance Criteria**:
- [ ] Errors display helpful suggestions
- [ ] Errors show searched directories
- [ ] Errors guide user to next step

**Subtasks**:
1. Update main.rs error handling:
   ```rust
   match run_search(query) {
       Ok(tree) => { /* display tree */ },
       Err(SearchError::NoTranslationFiles(text)) => {
           eprintln!("No translation files found containing '{}'", text);
           eprintln!("\nSearched in: config/locales, src/i18n");
           eprintln!("\nTip: Check your project structure or use --translations-dir");
           std::process::exit(1);
       },
       Err(e) => {
           eprintln!("Error: {}", e);
           std::process::exit(1);
       }
   }
   ```
2. Add suggestions for each error type
3. Test error message formatting

**Test Plan**:
```rust
#[test]
fn test_no_translation_files_message() {
    Command::cargo_bin("cs").unwrap()
        .arg("nonexistent")
        .assert()
        .failure()
        .stderr(predicate::str::contains("No translation files found"))
        .stderr(predicate::str::contains("Tip:"));
}
```

---

### US-6: Call Graph Tracing

#### Task 8.1: CLI Arguments for Tracing
**Priority**: P1 | **Estimate**: 2 hours | **Dependencies**: 1.3

**Acceptance Criteria**:
- [ ] `--trace` flag enables forward call tracing
- [ ] `--traceback` flag enables backward call tracing
- [ ] `--trace-all` flag enables both directions
- [ ] `--depth N` flag controls trace depth (default: 3, max: 10)

**Subtasks**:
1. Update CLI struct in `src/main.rs`:
   ```rust
   /// Trace forward call graph (what does this function call?)
   #[arg(long)]
   trace: bool,

   /// Trace backward call graph (who calls this function?)
   #[arg(long)]
   traceback: bool,

   /// Trace both directions
   #[arg(long)]
   trace_all: bool,

   /// Maximum depth for call tracing (default: 3, max: 10)
   #[arg(long, default_value = "3")]
   depth: usize,
   ```
2. Add validation: depth must be 1-10
3. Add mutual exclusivity check (can't use --trace with default i18n mode)

**Test Plan**:
```rust
#[test]
fn test_trace_flag_accepted() {
    Command::cargo_bin("cs").unwrap()
        .args(["bar", "--trace"])
        .assert()
        .success();
}

#[test]
fn test_depth_validation() {
    Command::cargo_bin("cs").unwrap()
        .args(["bar", "--trace", "--depth", "15"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("depth"));
}
```

---

#### Task 8.2: Function Definition Finder
**Priority**: P1 | **Estimate**: 6 hours | **Dependencies**: 1.4, 8.1

**Acceptance Criteria**:
- [ ] Finds function definitions using regex patterns
- [ ] Supports JavaScript/TypeScript, Ruby, Python, Rust patterns
- [ ] Returns file path, line number, and function name
- [ ] Handles multiple definitions with same name (different files)

**Subtasks**:
1. Create `src/trace/mod.rs` and `src/trace/function_finder.rs`
2. Define FunctionDef struct:
   ```rust
   pub struct FunctionDef {
       pub name: String,
       pub file: PathBuf,
       pub line: usize,
   }
   ```
3. Implement FunctionFinder with language patterns:
   ```rust
   fn default_patterns() -> Vec<Regex> {
       vec![
           Regex::new(r"function\s+(\w+)\s*\(").unwrap(),      // JS
           Regex::new(r"(\w+)\s*=\s*(?:async\s+)?\(").unwrap(), // JS arrow
           Regex::new(r"def\s+(\w+)").unwrap(),                // Ruby/Python
           Regex::new(r"fn\s+(\w+)").unwrap(),                 // Rust
       ]
   }
   ```
4. Use ripgrep to search for patterns, filter by function name
5. Write unit tests

**Test Plan**:
```rust
#[test]
fn test_find_js_function() {
    let finder = FunctionFinder::new();
    let defs = finder.find_definition("handleClick").unwrap();
    assert!(!defs.is_empty());
}

#[test]
fn test_find_ruby_method() {
    let finder = FunctionFinder::new();
    let defs = finder.find_definition("process_order").unwrap();
    assert!(!defs.is_empty());
}
```

---

#### Task 8.3: Call Extractor Implementation
**Priority**: P1 | **Estimate**: 6 hours | **Dependencies**: 8.2

**Acceptance Criteria**:
- [ ] Extracts function calls from a function body
- [ ] Filters out language keywords and built-ins
- [ ] Finds all callers of a given function
- [ ] Returns caller function name and location

**Subtasks**:
1. Create `src/trace/call_extractor.rs`
2. Implement extract_calls (forward tracing):
   ```rust
   pub fn extract_calls(&self, func: &FunctionDef) -> Result<Vec<String>> {
       // Read file content around function definition
       // Apply regex to find function calls: (\w+)\s*\(
       // Filter out keywords (if, for, while, etc.)
       // Return unique function names
   }
   ```
3. Implement find_callers (backward tracing):
   ```rust
   pub fn find_callers(&self, func_name: &str) -> Result<Vec<CallerInfo>> {
       // Use ripgrep to find all occurrences of func_name(
       // For each match, determine containing function
       // Return CallerInfo with function name and location
   }
   ```
4. Add keyword filter list for common languages
5. Write unit tests

**Test Plan**:
```rust
#[test]
fn test_extract_calls_from_function() {
    let func = FunctionDef { name: "foo".into(), /* ... */ };
    let calls = CallExtractor::new().extract_calls(&func).unwrap();
    assert!(calls.contains(&"bar".to_string()));
}

#[test]
fn test_find_callers() {
    let callers = CallExtractor::new().find_callers("processData").unwrap();
    assert!(!callers.is_empty());
}
```

---

#### Task 8.4: Call Graph Builder
**Priority**: P1 | **Estimate**: 8 hours | **Dependencies**: 8.2, 8.3

**Acceptance Criteria**:
- [ ] Builds forward call tree (--trace)
- [ ] Builds backward call tree (--traceback)
- [ ] Respects depth limit
- [ ] Detects and handles cycles (no infinite loops)
- [ ] Handles functions not found gracefully

**Subtasks**:
1. Create `src/trace/graph_builder.rs`
2. Implement CallGraphBuilder:
   ```rust
   pub fn build_trace(
       &self,
       start_func: &str,
       direction: TraceDirection,
       depth: usize,
   ) -> Result<CallTree> {
       let mut visited = HashSet::new();
       let mut queue = VecDeque::new();
       // BFS traversal with depth limit
       // Track visited to prevent cycles
   }
   ```
3. Implement CallTree data structure
4. Add cycle detection with visited set
5. Write unit tests for various scenarios

**Test Plan**:
```rust
#[test]
fn test_forward_trace_depth_limit() {
    let builder = CallGraphBuilder::new(3);
    let tree = builder.build_trace("main", TraceDirection::Forward, 3).unwrap();
    assert!(tree.max_depth() <= 3);
}

#[test]
fn test_cycle_detection() {
    // Create fixture with a -> b -> a cycle
    let tree = builder.build_trace("a", TraceDirection::Forward, 10).unwrap();
    // Should not hang, should stop at cycle
}

#[test]
fn test_backward_trace() {
    let tree = builder.build_trace("helper", TraceDirection::Backward, 3).unwrap();
    assert!(!tree.root.children.is_empty());
}
```

---

#### Task 8.5: Call Tree Formatter
**Priority**: P1 | **Estimate**: 4 hours | **Dependencies**: 8.4, 2.3

**Acceptance Criteria**:
- [ ] Formats forward trace as tree (function -> callees)
- [ ] Formats backward trace as chains (callers -> function)
- [ ] Shows file and line for each function
- [ ] Indicates when depth limit reached

**Subtasks**:
1. Extend `src/output/formatter.rs` for call trees
2. Implement forward trace format:
   ```
   bar
   |-> zoo1 (utils.ts:45)
   |-> zoo2 (helpers.ts:23)
   |-> zoo3 (api.ts:89)
   ```
3. Implement backward trace format:
   ```
   blah1 -> foo1 -> bar
   blah2 -> foo2 -> bar
   ```
4. Add "[depth limit reached]" indicator
5. Write formatting tests

**Test Plan**:
```rust
#[test]
fn test_format_forward_trace() {
    let tree = /* ... */;
    let output = formatter.format_call_tree(&tree, TraceDirection::Forward);
    assert!(output.contains("|->"));
}

#[test]
fn test_format_backward_trace() {
    let tree = /* ... */;
    let output = formatter.format_call_tree(&tree, TraceDirection::Backward);
    assert!(output.contains(" -> "));
}
```

---

#### Task 8.6: Test Fixtures for Call Tracing
**Priority**: P1 | **Estimate**: 3 hours | **Dependencies**: 1.2

**Acceptance Criteria**:
- [ ] JavaScript fixture with nested function calls
- [ ] Fixture with circular call pattern (a -> b -> a)
- [ ] Fixture with multiple callers of same function

**Subtasks**:
1. Create `tests/fixtures/call-trace/` directory
2. Add `functions.js`:
   ```javascript
   function main() { foo(); bar(); }
   function foo() { helper1(); helper2(); }
   function bar() { helper1(); baz(); }
   function helper1() { /* leaf */ }
   function helper2() { helper1(); }
   function baz() { foo(); }  // Creates cycle: bar -> baz -> foo
   ```
3. Add similar fixtures for Ruby and Python
4. Document expected trace outputs

**Test Plan**:
- Manual verification of fixture structure
- Integration tests use these fixtures

---

#### Task 8.7: Integration with Main Orchestrator
**Priority**: P1 | **Estimate**: 4 hours | **Dependencies**: 8.4, 8.5, 1.8

**Acceptance Criteria**:
- [ ] main.rs routes to call tracing when --trace/--traceback used
- [ ] Default mode (no flags) still does i18n tracing
- [ ] Error messages specific to call tracing mode

**Subtasks**:
1. Update `src/lib.rs` to add trace mode:
   ```rust
   pub fn run_trace(
       func_name: &str,
       direction: TraceDirection,
       depth: usize,
   ) -> Result<CallTree> {
       let builder = CallGraphBuilder::new(depth);
       builder.build_trace(func_name, direction, depth)
   }
   ```
2. Update `src/main.rs` to dispatch based on flags:
   ```rust
   if cli.trace || cli.traceback || cli.trace_all {
       let direction = if cli.trace_all { TraceDirection::Both }
           else if cli.traceback { TraceDirection::Backward }
           else { TraceDirection::Forward };
       let tree = run_trace(&cli.search_text, direction, cli.depth)?;
       // Format and display
   } else {
       // Existing i18n search
   }
   ```
3. Add call-tracing specific error handling
4. Write integration tests

**Test Plan**:
```rust
#[test]
fn test_cli_trace_mode() {
    Command::cargo_bin("cs").unwrap()
        .args(["foo", "--trace"])
        .current_dir("tests/fixtures/call-trace")
        .assert()
        .success()
        .stdout(predicate::str::contains("helper1"));
}

#[test]
fn test_cli_traceback_mode() {
    Command::cargo_bin("cs").unwrap()
        .args(["helper1", "--traceback"])
        .current_dir("tests/fixtures/call-trace")
        .assert()
        .success()
        .stdout(predicate::str::contains("foo"));
}
```

---

## Performance & Quality Tasks

### Task 6.1: Performance Benchmarks
**Priority**: P1 | **Estimate**: 6 hours | **Dependencies**: 1.8, 2.4

**Acceptance Criteria**:
- [ ] Criterion benchmarks for all phases
- [ ] Benchmarks for small/medium/large projects
- [ ] Results documented in CHANGELOG

**Subtasks**:
1. Create `benches/search_benchmark.rs`
2. Implement benchmarks:
   ```rust
   fn benchmark_text_search(c: &mut Criterion) {
       c.bench_function("text search small project", |b| {
           b.iter(|| TextSearcher.search(black_box("add new")))
       });
   }

   fn benchmark_end_to_end(c: &mut Criterion) {
       c.bench_function("end-to-end rails app", |b| {
           b.iter(|| run_search(black_box(query)))
       });
   }
   ```
3. Run benchmarks, verify performance targets
4. Document results

**Test Plan**:
- `cargo bench` runs successfully
- Small project: < 100ms
- Medium project: < 500ms

---

### Task 6.2: Integration Test Suite
**Priority**: P1 | **Estimate**: 6 hours | **Dependencies**: 2.4, 3.2, 4.2

**Acceptance Criteria**:
- [ ] Integration tests for all user stories
- [ ] Tests for error scenarios
- [ ] Tests for edge cases (empty files, malformed YAML)

**Subtasks**:
1. Create `tests/integration/basic_search.rs`
2. Create `tests/integration/multi_match.rs`
3. Create `tests/integration/error_cases.rs`
4. Implement comprehensive test cases
5. Verify 80% coverage with cargo-tarpaulin

**Test Plan**:
```bash
cargo test --test integration
cargo tarpaulin --out Html
# Verify coverage ≥ 80%
```

---

### Task 6.3: Documentation
**Priority**: P1 | **Estimate**: 6 hours | **Dependencies**: All implementation tasks

**Acceptance Criteria**:
- [ ] README.md complete with examples
- [ ] API documentation with rustdoc
- [ ] CONTRIBUTING.md present
- [ ] CHANGELOG.md initialized

**Subtasks**:
1. Complete README.md:
   - Installation instructions
   - Usage examples
   - Supported frameworks
   - Configuration options
2. Add rustdoc comments to all public APIs
3. Create CONTRIBUTING.md with development setup
4. Initialize CHANGELOG.md with v0.1.0 entries

**Test Plan**:
- `cargo doc --no-deps --open` generates docs
- README examples work when followed

---

## Release Tasks

### Task 7.1: CI/CD Setup
**Priority**: P1 | **Estimate**: 4 hours | **Dependencies**: 1.1

**Acceptance Criteria**:
- [ ] GitHub Actions workflow for tests
- [ ] GitHub Actions workflow for linting
- [ ] GitHub Actions workflow for benchmarks
- [ ] GitHub Actions workflow for releases

**Subtasks**:
1. Create `.github/workflows/ci.yml`:
   ```yaml
   name: CI
   on: [push, pull_request]
   jobs:
     test:
       runs-on: ubuntu-latest
       steps:
         - uses: actions/checkout@v3
         - uses: actions-rs/toolchain@v1
         - run: cargo test
         - run: cargo clippy
     bench:
       runs-on: ubuntu-latest
       steps:
         - run: cargo bench
   ```
2. Create release workflow for cross-compilation
3. Test workflows locally with act

---

### Task 7.2: Release v0.1.0
**Priority**: P1 | **Estimate**: 4 hours | **Dependencies**: All tasks

**Acceptance Criteria**:
- [ ] Version bumped to 0.1.0
- [ ] Git tag created
- [ ] Binaries built for Linux, macOS, Windows
- [ ] Published to crates.io
- [ ] GitHub release created

**Subtasks**:
1. Update Cargo.toml version to 0.1.0
2. Update CHANGELOG.md with all changes
3. Create git tag: `git tag v0.1.0`
4. Run release workflow (or cross-compile locally)
5. Publish to crates.io: `cargo publish`
6. Create GitHub release with binaries

---

## Task Dependencies Graph

```
1.1 (Setup)
  ├─> 1.2 (Fixtures)
  ├─> 1.3 (CLI) ─> 1.4 (Ripgrep) ─┐
  ├─> 1.5 (YAML) ─> 1.6 (KeyExt) ─┤
  └─> 1.7 (Patterns) ─────────────┴─> 1.8 (Orchestrator)
                                          │
                      2.1 (TreeNode) ─> 2.2 (Builder) ─> 2.3 (Formatter) ─> 2.4 (Display)
                                          │                                      │
                      3.1 (MultiTrans) ───┤                                      │
                      3.2 (MultiCode) ────┤                                      │
                      4.1 (React) ────────┤                                      │
                      4.2 (Vue) ──────────┤                                      │
                      5.1 (Errors) ───────┴─> 5.2 (Messages) ──────────────────┘
                                                                                  │
                      6.1 (Bench) ───────────────────────────────────────────────┤
                      6.2 (IntTests) ────────────────────────────────────────────┤
                      6.3 (Docs) ────────────────────────────────────────────────┤
                                                                                  │
                      7.1 (CI/CD) ───────────────────────────────────────────────┤
                      7.2 (Release) ─────────────────────────────────────────────┘
```

## Execution Timeline

**Week 1** (35 hours):
- Day 1: Tasks 1.1, 1.2, 1.3 (11 hours)
- Day 2: Tasks 1.4, 1.5 (14 hours)
- Day 3: Tasks 1.6, 1.7 (12 hours)

**Week 2** (38 hours):
- Day 1: Tasks 1.8, 2.1, 2.2 (15 hours)
- Day 2: Tasks 2.3, 2.4, 3.1 (15 hours)
- Day 3: Tasks 3.2, 4.1, 4.2 (11 hours)

**Week 3** (29 hours):
- Day 1: Tasks 5.1, 5.2, 6.1 (13 hours)
- Day 2: Tasks 6.2, 6.3 (12 hours)
- Day 3: Tasks 7.1, 7.2 (8 hours)

**Total Estimate**: ~102 hours (12-13 days of full-time work)

---

**Status Tracking**: Mark tasks complete as you finish them. Update this document if dependencies change or new tasks are discovered during implementation.
