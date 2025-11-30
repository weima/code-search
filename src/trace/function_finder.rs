use crate::error::{Result, SearchError};
use crate::search::TextSearcher;
use regex::Regex;
use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;

/// Represents a function definition found in code
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FunctionDef {
    pub name: String,
    pub file: PathBuf,
    pub line: usize,
    pub body: String,
}

/// Finds function definitions in code using pattern matching
pub struct FunctionFinder {
    searcher: TextSearcher,
    patterns: Vec<Regex>,
    base_dir: PathBuf,
}

impl FunctionFinder {
    pub fn new(base_dir: PathBuf) -> Self {
        Self {
            searcher: TextSearcher::new(base_dir.clone()),
            patterns: Self::default_patterns(),
            base_dir,
        }
    }

    /// Default patterns for finding function definitions across languages
    fn default_patterns() -> Vec<Regex> {
        vec![
            // JavaScript/TypeScript - function declarations
            Regex::new(r"function\s+(\w+)\s*\(").unwrap(),
            // JavaScript/TypeScript - arrow functions
            Regex::new(r"(?:const|let|var)\s+(\w+)\s*=\s*(?:async\s+)?\([^)]*\)\s*=>").unwrap(),
            // JavaScript/TypeScript - method definitions
            Regex::new(r"^\s*(?:async\s+)?(\w+)\s*\([^)]*\)\s*\{").unwrap(),
            // JavaScript/TypeScript - export functions
            Regex::new(r"export\s+function\s+(\w+)").unwrap(),
            // JavaScript/TypeScript - class methods with modifiers
            Regex::new(r"^\s*(?:public|private|protected|static|async)\s+(\w+)\s*\(").unwrap(),
            // Ruby - method definitions
            Regex::new(r"def\s+(\w+)").unwrap(),
            // Ruby - class methods
            Regex::new(r"def\s+self\.(\w+)").unwrap(),
            // Python - function definitions
            Regex::new(r"def\s+(\w+)\s*\(").unwrap(),
            // Rust - function definitions
            Regex::new(r"fn\s+(\w+)\s*[<(]").unwrap(),
        ]
    }

    /// Generate case variants of a function name for cross-case searching
    ///
    /// For input "createUser" generates: ["createUser", "create_user", "CreateUser"]
    /// For input "user_profile" generates: ["user_profile", "userProfile", "UserProfile"]
    fn generate_case_variants(func_name: &str) -> Vec<String> {
        let mut variants = HashSet::new();

        // Always include the original
        variants.insert(func_name.to_string());

        // Generate snake_case variant
        let snake_case = Self::to_snake_case(func_name);
        variants.insert(snake_case.clone());

        // Generate camelCase variant
        let camel_case = Self::to_camel_case(&snake_case);
        variants.insert(camel_case.clone());

        // Generate PascalCase variant
        let pascal_case = Self::to_pascal_case(&snake_case);
        variants.insert(pascal_case);

        variants.into_iter().collect()
    }

    /// Convert to snake_case
    fn to_snake_case(input: &str) -> String {
        let mut result = String::new();

        for (i, ch) in input.chars().enumerate() {
            if ch.is_uppercase() && i > 0 {
                result.push('_');
            }
            result.push(ch.to_lowercase().next().unwrap());
        }

        result
    }

    /// Convert snake_case to camelCase
    fn to_camel_case(input: &str) -> String {
        let parts: Vec<&str> = input.split('_').collect();
        if parts.is_empty() {
            return String::new();
        }

        let mut result = parts[0].to_lowercase();
        for part in parts.iter().skip(1) {
            if !part.is_empty() {
                let mut chars = part.chars();
                if let Some(first) = chars.next() {
                    result.push(first.to_uppercase().next().unwrap());
                    result.push_str(&chars.as_str().to_lowercase());
                }
            }
        }

        result
    }

    /// Convert snake_case to PascalCase
    fn to_pascal_case(input: &str) -> String {
        let parts: Vec<&str> = input.split('_').collect();
        let mut result = String::new();

        for part in parts {
            if !part.is_empty() {
                let mut chars = part.chars();
                if let Some(first) = chars.next() {
                    result.push(first.to_uppercase().next().unwrap());
                    result.push_str(&chars.as_str().to_lowercase());
                }
            }
        }

        result
    }

    pub fn find_function(&self, func_name: &str) -> Option<FunctionDef> {
        // Try exact match first
        if let Ok(mut defs) = self.find_definition(func_name) {
            if let Some(def) = defs.pop() {
                return Some(def);
            }
        }

        // Try case variants if exact match fails
        let variants = Self::generate_case_variants(func_name);

        for variant in variants {
            if variant != func_name {
                // Skip the exact match we already tried
                if let Ok(mut defs) = self.find_definition(&variant) {
                    if let Some(def) = defs.pop() {
                        return Some(def);
                    }
                }
            }
        }

        None
    }

    /// Find all definitions of a function by name
    ///
    /// Searches the codebase for function definitions matching the given name.
    /// Returns all found definitions with their file locations.
    pub fn find_definition(&self, func_name: &str) -> Result<Vec<FunctionDef>> {
        let mut results = Vec::new();

        // Search for the function name in code
        let matches = self.searcher.search(func_name)?;

        // Filter matches that look like function definitions
        for m in matches {
            // Filter out the tool's own source files
            // Convert absolute path to relative path for filtering
            let relative_path = match m.file.strip_prefix(&self.base_dir) {
                Ok(rel_path) => rel_path.to_string_lossy().to_lowercase(),
                Err(_) => m.file.to_string_lossy().to_lowercase(),
            };

            if relative_path.starts_with("src/")
                || (relative_path.starts_with("tests/")
                    && !relative_path.starts_with("tests/fixtures/"))
            {
                continue;
            }

            let content = &m.content;

            // Check if this line matches any function definition pattern
            for pattern in &self.patterns {
                if let Some(captures) = pattern.captures(content) {
                    if let Some(name_match) = captures.get(1) {
                        let name = name_match.as_str();
                        // Ensure exact match (not substring)
                        if name == func_name {
                            let file_content = fs::read_to_string(&m.file)?;
                            let body = file_content
                                .lines()
                                .skip(m.line - 1)
                                .collect::<Vec<_>>()
                                .join("\n");
                            results.push(FunctionDef {
                                name: name.to_string(),
                                file: m.file.clone(),
                                line: m.line,
                                body,
                            });
                            break; // Found a match for this line
                        }
                    }
                }
            }
        }

        if results.is_empty() {
            Err(SearchError::Generic(format!(
                "Function '{}' not found in codebase",
                func_name
            )))
        } else {
            // Sort results by file path then line number for deterministic ordering
            results.sort_by(|a, b| a.file.cmp(&b.file).then(a.line.cmp(&b.line)));
            Ok(results)
        }
    }
}

impl Default for FunctionFinder {
    fn default() -> Self {
        Self::new(std::env::current_dir().unwrap())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_function_finder_creation() {
        let finder = FunctionFinder::new(std::env::current_dir().unwrap());
        assert!(!finder.patterns.is_empty());
    }

    #[test]
    fn test_patterns_compile() {
        let patterns = FunctionFinder::default_patterns();
        assert_eq!(patterns.len(), 9);
    }

    #[test]
    fn test_js_function_pattern() {
        let patterns = FunctionFinder::default_patterns();
        let js_pattern = &patterns[0];

        assert!(js_pattern.is_match("function handleClick() {"));
        assert!(js_pattern.is_match("function processData(x, y) {"));
        assert!(!js_pattern.is_match("const x = function() {"));
    }

    #[test]
    fn test_arrow_function_pattern() {
        let patterns = FunctionFinder::default_patterns();
        let arrow_pattern = &patterns[1];

        assert!(arrow_pattern.is_match("const handleClick = () => {"));
        assert!(arrow_pattern.is_match("let processData = async (x) => {"));
        assert!(arrow_pattern.is_match("var foo = (a, b) => {"));
    }

    #[test]
    fn test_ruby_pattern() {
        let patterns = FunctionFinder::default_patterns();
        let ruby_pattern = &patterns[5]; // Updated index for "def \w+" pattern

        assert!(ruby_pattern.is_match("def process_order"));
        assert!(ruby_pattern.is_match("  def calculate_total"));
    }

    #[test]
    fn test_python_pattern() {
        let patterns = FunctionFinder::default_patterns();
        let python_pattern = &patterns[7]; // Updated index for Python pattern

        assert!(python_pattern.is_match("def process_data(x):"));
        assert!(python_pattern.is_match("    def helper():"));
    }

    #[test]
    fn test_rust_pattern() {
        let patterns = FunctionFinder::default_patterns();
        let rust_pattern = &patterns[8]; // Updated index for Rust pattern

        assert!(rust_pattern.is_match("fn main() {"));
        assert!(rust_pattern.is_match("fn process<T>(x: T) {"));
        assert!(rust_pattern.is_match("pub fn calculate("));
    }

    #[test]
    fn test_javascript_export_patterns() {
        let patterns = FunctionFinder::default_patterns();
        let export_func_pattern = &patterns[3];

        assert!(export_func_pattern.is_match("export function processData"));
        assert!(export_func_pattern.is_match("export function calculate"));
    }

    #[test]
    fn test_javascript_method_patterns() {
        let patterns = FunctionFinder::default_patterns();
        let method_pattern = &patterns[2];

        assert!(method_pattern.is_match("  processData() {"));
        assert!(method_pattern.is_match("    handleClick() {"));
        assert!(method_pattern.is_match("  async methodName() {"));
    }

    #[test]
    fn test_ruby_class_methods() {
        let patterns = FunctionFinder::default_patterns();
        let ruby_class_method_pattern = &patterns[6];

        assert!(ruby_class_method_pattern.is_match("def self.create"));
        assert!(ruby_class_method_pattern.is_match("  def self.find_by_name"));
    }

    #[test]
    fn test_case_conversion() {
        // Test snake_case conversion
        assert_eq!(FunctionFinder::to_snake_case("createUser"), "create_user");
        assert_eq!(
            FunctionFinder::to_snake_case("validateEmailAddress"),
            "validate_email_address"
        );
        assert_eq!(
            FunctionFinder::to_snake_case("XMLHttpRequest"),
            "x_m_l_http_request"
        );
        assert_eq!(
            FunctionFinder::to_snake_case("already_snake"),
            "already_snake"
        );

        // Test camelCase conversion
        assert_eq!(FunctionFinder::to_camel_case("create_user"), "createUser");
        assert_eq!(
            FunctionFinder::to_camel_case("validate_email_address"),
            "validateEmailAddress"
        );
        assert_eq!(FunctionFinder::to_camel_case("single"), "single");

        // Test PascalCase conversion
        assert_eq!(FunctionFinder::to_pascal_case("create_user"), "CreateUser");
        assert_eq!(
            FunctionFinder::to_pascal_case("validate_email_address"),
            "ValidateEmailAddress"
        );
        assert_eq!(FunctionFinder::to_pascal_case("single"), "Single");
    }

    #[test]
    fn test_generate_case_variants() {
        // Test with camelCase input
        let variants = FunctionFinder::generate_case_variants("createUser");
        assert!(variants.contains(&"createUser".to_string()));
        assert!(variants.contains(&"create_user".to_string()));
        assert!(variants.contains(&"CreateUser".to_string()));

        // Test with snake_case input
        let variants = FunctionFinder::generate_case_variants("validate_email");
        assert!(variants.contains(&"validate_email".to_string()));
        assert!(variants.contains(&"validateEmail".to_string()));
        assert!(variants.contains(&"ValidateEmail".to_string()));

        // Test with PascalCase input
        let variants = FunctionFinder::generate_case_variants("ProcessUserData");
        assert!(variants.contains(&"ProcessUserData".to_string()));
        assert!(variants.contains(&"process_user_data".to_string()));
        assert!(variants.contains(&"processUserData".to_string()));
    }
}
