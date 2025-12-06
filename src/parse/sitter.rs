use anyhow::{Context, Result};
use std::collections::HashMap;
use std::path::Path;
use tree_sitter::{Language, Parser, Query, QueryCursor};

/// Supported languages for Tree-sitter parsing
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SupportedLanguage {
    Rust,
    Python,
    JavaScript,
    TypeScript,
    Ruby,
    CSharp,
    // Erb, // Temporarily disabled due to tree-sitter version conflict
}

impl SupportedLanguage {
    pub fn from_path(path: &Path) -> Option<Self> {
        match path.extension()?.to_str()? {
            "rs" => Some(Self::Rust),
            "py" => Some(Self::Python),
            "js" | "jsx" => Some(Self::JavaScript),
            "ts" | "tsx" => Some(Self::TypeScript),
            "rb" => Some(Self::Ruby),
            "cs" => Some(Self::CSharp),
            // "erb" => Some(Self::Erb), // Temporarily disabled
            _ => None,
        }
    }

    pub fn language(&self) -> Language {
        match self {
            Self::Rust => tree_sitter_rust::language(),
            Self::Python => tree_sitter_python::language(),
            Self::JavaScript => tree_sitter_javascript::language(),
            Self::TypeScript => tree_sitter_typescript::language_typescript(),
            Self::Ruby => tree_sitter_ruby::language(),
            Self::CSharp => tree_sitter_c_sharp::language(),
            // Self::Erb => tree_sitter_embedded_template::language(),
        }
    }
}

/// Sitter handles Tree-sitter parsing for multiple languages
pub struct Sitter {
    parsers: HashMap<SupportedLanguage, Parser>,
    queries: HashMap<SupportedLanguage, Query>,
}

impl Default for Sitter {
    fn default() -> Self {
        Self::new()
    }
}

impl Sitter {
    pub fn new() -> Self {
        Self {
            parsers: HashMap::new(),
            queries: HashMap::new(),
        }
    }

    /// Check if the file at the given path is supported by Tree-sitter
    pub fn is_supported(&self, path: &Path) -> bool {
        SupportedLanguage::from_path(path).is_some()
    }

    /// Get or create a parser for the given language
    fn get_parser(&mut self, lang: SupportedLanguage) -> Result<&mut Parser> {
        if let std::collections::hash_map::Entry::Vacant(e) = self.parsers.entry(lang) {
            let mut parser = Parser::new();
            parser
                .set_language(lang.language())
                .context("Failed to set parser language")?;
            e.insert(parser);
        }
        Ok(self.parsers.get_mut(&lang).unwrap())
    }

    /// Get or create a query for the given language
    fn get_query(&mut self, lang: SupportedLanguage) -> Result<&Query> {
        if let std::collections::hash_map::Entry::Vacant(e) = self.queries.entry(lang) {
            let query_str = match lang {
                SupportedLanguage::Rust => {
                    r#"
                    (function_item name: (identifier) @name)
                    (function_signature_item name: (identifier) @name)
                "#
                }
                SupportedLanguage::Python => {
                    r#"
                    (function_definition name: (identifier) @name)
                "#
                }
                SupportedLanguage::JavaScript | SupportedLanguage::TypeScript => {
                    r#"
                    (function_declaration name: (identifier) @name)
                    (export_statement (function_declaration name: (identifier) @name))
                    (method_definition name: (property_identifier) @name)
                    (arrow_function) @arrow
                    (variable_declarator
                        name: (identifier) @name
                        value: (arrow_function))
                "#
                }
                SupportedLanguage::Ruby => {
                    r#"
                    (method name: (identifier) @name)
                    (singleton_method name: (identifier) @name)
                "#
                }
                SupportedLanguage::CSharp => {
                    r#"
                    (method_declaration name: (identifier) @name)
                    (local_function_statement name: (identifier) @name)
                "#
                } // SupportedLanguage::Erb => "", // ERB usually doesn't define functions
            };

            let query = Query::new(lang.language(), query_str)
                .map_err(|e| anyhow::anyhow!("Failed to create query: {:?}", e))?;
            e.insert(query);
        }
        Ok(self.queries.get(&lang).unwrap())
    }

    /// Find function definitions in the given file
    pub fn find_functions(&mut self, path: &Path, code: &str) -> Result<Vec<FunctionMatch>> {
        let lang = match SupportedLanguage::from_path(path) {
            Some(l) => l,
            None => return Ok(Vec::new()), // Unsupported language
        };

        let parser = self.get_parser(lang)?;
        let tree = parser.parse(code, None).context("Failed to parse code")?;

        let query = self.get_query(lang)?;
        let mut cursor = QueryCursor::new();
        let matches = cursor.matches(query, tree.root_node(), code.as_bytes());

        let mut functions = Vec::new();
        // Capture index for @name is usually 0 if it's the first capture
        let name_idx = query.capture_index_for_name("name").unwrap_or(0);

        for m in matches {
            for capture in m.captures {
                if capture.index == name_idx {
                    let range = capture.node.range();
                    let start_line = range.start_point.row + 1; // 1-based
                    let end_line = range.end_point.row + 1;

                    let name = capture.node.utf8_text(code.as_bytes())?.to_string();

                    functions.push(FunctionMatch {
                        name,
                        start_line,
                        end_line,
                    });
                }
            }
        }

        Ok(functions)
    }
}

#[derive(Debug)]
pub struct FunctionMatch {
    pub name: String,
    pub start_line: usize,
    pub end_line: usize,
}
