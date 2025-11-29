use regex::Regex;
use std::path::PathBuf;
use crate::search::TextSearcher;

/// Represents a function definition found in code
#[derive(Debug, Clone)]
pub struct FunctionDef {
    pub name: String,
    pub file: PathBuf,
    pub line: usize,
}

/// Finds function definitions in code using pattern matching
pub struct FunctionFinder {
    searcher: TextSearcher,
    patterns: Vec<Regex>,
}

impl FunctionFinder {
    pub fn new() -> Self {
        Self {
            searcher: TextSearcher::new(),
            patterns: Self::default_patterns(),
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
            Regex::new(r"^\s*(\w+)\s*\([^)]*\)\s*\{").unwrap(),
            // Ruby - method definitions
            Regex::new(r"def\s+(\w+)").unwrap(),
            // Python - function definitions
            Regex::new(r"def\s+(\w+)\s*\(").unwrap(),
            // Rust - function definitions
            Regex::new(r"fn\s+(\w+)\s*[<(]").unwrap(),
        ]
    }

    /// Find all definitions of a function by name
    pub fn find_definition(&self, func_name: &str) -> Result<Vec<FunctionDef>, String> {
        let mut results = Vec::new();

        // Search for the function name in code
        let matches = self.searcher
            .search(func_name)
            .map_err(|e| format!("Search failed: {}", e))?;

        // Filter matches that look like function definitions
        for m in matches {
            let content = &m.content;
            
            // Check if this line matches any function definition pattern
            for pattern in &self.patterns {
                if let Some(captures) = pattern.captures(content) {
                    if let Some(name_match) = captures.get(1) {
                        let name = name_match.as_str();
                        if name == func_name {
                            results.push(FunctionDef {
                                name: name.to_string(),
                                file: m.file.clone(),
                                line: m.line,
                            });
                            break; // Found a match for this line
                        }
                    }
                }
            }
        }

        if results.is_empty() {
            Err(format!("Function '{}' not found in codebase", func_name))
        } else {
            Ok(results)
        }
    }
}

impl Default for FunctionFinder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_function_finder_creation() {
        let finder = FunctionFinder::new();
        assert!(!finder.patterns.is_empty());
    }

    #[test]
    fn test_patterns_compile() {
        let patterns = FunctionFinder::default_patterns();
        assert_eq!(patterns.len(), 6);
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
        let ruby_pattern = &patterns[3];
        
        assert!(ruby_pattern.is_match("def process_order"));
        assert!(ruby_pattern.is_match("  def calculate_total"));
    }

    #[test]
    fn test_python_pattern() {
        let patterns = FunctionFinder::default_patterns();
        let python_pattern = &patterns[4];
        
        assert!(python_pattern.is_match("def process_data(x):"));
        assert!(python_pattern.is_match("    def helper():"));
    }

    #[test]
    fn test_rust_pattern() {
        let patterns = FunctionFinder::default_patterns();
        let rust_pattern = &patterns[5];
        
        assert!(rust_pattern.is_match("fn main() {"));
        assert!(rust_pattern.is_match("fn process<T>(x: T) {"));
        assert!(rust_pattern.is_match("pub fn calculate("));
    }
}
