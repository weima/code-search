use crate::error::Result;
use crate::search::TextSearcher;
use regex::Regex;
use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;

use super::FunctionDef;

/// Information about a function that calls another function
#[derive(Debug, Clone)]
pub struct CallerInfo {
    pub caller_name: String,
    pub file: PathBuf,
    pub line: usize,
}

/// Extracts function calls from code
pub struct CallExtractor {
    searcher: TextSearcher,
    call_patterns: Vec<Regex>,
    pub keywords: HashSet<String>,
}

impl CallExtractor {
    pub fn new(base_dir: PathBuf) -> Self {
        Self {
            searcher: TextSearcher::new(base_dir),
            call_patterns: Self::default_call_patterns(),
            keywords: Self::common_keywords(),
        }
    }

    /// Default patterns for finding function calls across languages
    fn default_call_patterns() -> Vec<Regex> {
        vec![
            // JavaScript/TypeScript - direct function calls
            Regex::new(r"\b(\w+)\s*\(").unwrap(),
            // JavaScript/TypeScript - method calls
            Regex::new(r"\.(\w+)\s*\(").unwrap(),
            // JavaScript/TypeScript - chained calls
            Regex::new(r"\.(\w+)\s*\([^)]*\)\.(\w+)").unwrap(),
            // Ruby - method calls
            Regex::new(r"\b(\w+)\s*\(").unwrap(),
            // Ruby - method calls without parentheses
            Regex::new(r"\b(\w+)\s+\w+").unwrap(),
        ]
    }

    /// Common language keywords to filter out (not function calls)
    fn common_keywords() -> HashSet<String> {
        let keywords = vec![
            // JavaScript/TypeScript keywords
            "if",
            "for",
            "while",
            "switch",
            "catch",
            "typeof",
            "instanceof",
            "const",
            "let",
            "var",
            "function",
            "class",
            "extends",
            "import",
            "export",
            "from",
            "async",
            "await",
            "try",
            "finally",
            "else",
            "break",
            "continue",
            "case",
            "default",
            "do",
            "in",
            "of",
            // JavaScript/TypeScript built-ins
            "console",
            "window",
            "document",
            "setTimeout",
            "setInterval",
            "parseInt",
            "parseFloat",
            "isNaN",
            "Object",
            "Array",
            "String",
            "Number",
            "Boolean",
            "Date",
            "Math",
            "JSON",
            "Promise",
            // TypeScript specific
            "interface",
            "type",
            "enum",
            "namespace",
            "declare",
            "abstract",
            "implements",
            "public",
            "private",
            "protected",
            "readonly",
            // Ruby keywords
            "if",
            "unless",
            "case",
            "when",
            "while",
            "until",
            "for",
            "in",
            "begin",
            "rescue",
            "ensure",
            "end",
            "class",
            "module",
            "def",
            "puts",
            "print",
            "p",
            "require",
            "include",
            "extend",
            "attr_reader",
            "attr_writer",
            "attr_accessor",
            "private",
            "protected",
            "public",
            // Ruby built-ins
            "Array",
            "Hash",
            "String",
            "Integer",
            "Float",
            "Numeric",
            "File",
            // Common programming constructs
            "return",
            "new",
            "delete",
            "throw",
            "raise",
            "yield",
            "super",
        ];
        keywords.into_iter().map(String::from).collect()
    }

    /// Extract function calls from a function body
    ///
    /// Reads the function definition and extracts all function calls within its body.
    /// Filters out language keywords and built-in functions.
    pub fn extract_calls(&self, func: &FunctionDef) -> Result<Vec<String>> {
        // Read the file
        let content = fs::read_to_string(&func.file)?;
        let lines: Vec<&str> = content.lines().collect();

        // Find the function body - be smarter about detecting function boundaries
        let start_line = func.line.saturating_sub(1);
        let end_line = self.find_function_end(&lines, start_line).min(lines.len());

        let mut calls = HashSet::new();

        for line in &lines[start_line..end_line] {
            // Skip comments and strings
            if self.is_comment_or_string(line) {
                continue;
            }

            // Find all function calls using multiple patterns
            for pattern in &self.call_patterns {
                for cap in pattern.captures_iter(line) {
                    // Try each capture group (patterns may have different group structures)
                    for i in 1..cap.len() {
                        if let Some(name_match) = cap.get(i) {
                            let name = name_match.as_str();

                            // Filter out invalid function names
                            if self.is_valid_function_name(name)
                                && !self.keywords.contains(name)
                                && name != func.name
                            {
                                calls.insert(name.to_string());
                            }
                        }
                    }
                }
            }
        }

        Ok(calls.into_iter().collect())
    }

    /// Find the end of a function definition
    fn find_function_end(&self, lines: &[&str], start_line: usize) -> usize {
        if start_line >= lines.len() {
            return lines.len();
        }

        let start_content = lines[start_line].trim();
        
        // Check for brace-based languages (JS, Rust, etc.)
        if start_content.contains('{') || (start_line + 1 < lines.len() && lines[start_line + 1].trim().contains('{')) {
            return self.find_brace_end(lines, start_line);
        }
        
        // Check for Ruby (def ... end)
        if start_content.starts_with("def ") {
            return self.find_ruby_end(lines, start_line);
        }
        
        // Check for Python (indentation)
        if start_content.starts_with("def ") && start_content.ends_with(':') {
            return self.find_python_end(lines, start_line);
        }

        // Default fallback
        (start_line + 30).min(lines.len())
    }

    fn find_brace_end(&self, lines: &[&str], start_line: usize) -> usize {
        let mut brace_count = 0;
        let mut found_opening = false;

        for (i, line) in lines.iter().enumerate().skip(start_line) {
            for ch in line.chars() {
                match ch {
                    '{' => {
                        brace_count += 1;
                        found_opening = true;
                    }
                    '}' => {
                        brace_count -= 1;
                        if found_opening && brace_count == 0 {
                            return i + 1;
                        }
                    }
                    _ => {}
                }
            }
        }
        (start_line + 30).min(lines.len())
    }

    fn find_ruby_end(&self, lines: &[&str], start_line: usize) -> usize {
        let mut depth = 0;
        let mut found_start = false;

        for (i, line) in lines.iter().enumerate().skip(start_line) {
            let trimmed = line.trim();
            // Simple heuristic for Ruby blocks
            if trimmed.starts_with("def ") || trimmed.starts_with("class ") || trimmed.starts_with("module ") || trimmed.starts_with("if ") || trimmed.starts_with("do ") || trimmed.starts_with("begin ") {
                depth += 1;
                found_start = true;
            }
            
            if trimmed == "end" || trimmed.starts_with("end ") {
                depth -= 1;
                if found_start && depth == 0 {
                    return i + 1;
                }
            }
        }
        (start_line + 30).min(lines.len())
    }

    fn find_python_end(&self, lines: &[&str], start_line: usize) -> usize {
        // Get indentation of the function definition
        let def_indent = lines[start_line].chars().take_while(|c| c.is_whitespace()).count();
        
        for (i, line) in lines.iter().enumerate().skip(start_line + 1) {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }
            
            let current_indent = line.chars().take_while(|c| c.is_whitespace()).count();
            if current_indent <= def_indent {
                return i;
            }
        }
        lines.len()
    }

    /// Check if a line is a comment or inside a string literal
    fn is_comment_or_string(&self, line: &str) -> bool {
        let trimmed = line.trim();
        // JavaScript/TypeScript comments
        trimmed.starts_with("//") || trimmed.starts_with("/*") ||
        // Ruby/Python comments
        trimmed.starts_with("#")
    }

    /// Check if a string is a valid function name
    fn is_valid_function_name(&self, name: &str) -> bool {
        // Must be a valid identifier
        !name.is_empty()
            && name.chars().all(|c| c.is_alphanumeric() || c == '_')
            && !name.chars().next().unwrap().is_numeric()
    }

    /// Find all functions that call the given function
    ///
    /// Searches the codebase for all calls to `func_name` and identifies
    /// the calling function for each occurrence. Uses case variants for cross-language support.
    pub fn find_callers(&self, func_name: &str) -> Result<Vec<CallerInfo>> {
        let mut callers = Vec::new();

        // Generate case variants for cross-case searching
        let variants = Self::generate_case_variants(func_name);

        // Search for each variant
        for variant in variants {
            let matches = self.searcher.search(&variant)?;

            for m in matches {
                // Skip comment lines (JavaScript //, Ruby/Python #)
                let trimmed = m.content.trim();
                if trimmed.starts_with("//") || trimmed.starts_with("#") {
                    continue;
                }

                // Ensure it's a function call (variant followed by '(') with word boundary
                let call_regex = Regex::new(&format!(r"\b{}\s*\(", regex::escape(&variant))).unwrap();
                if !call_regex.is_match(&m.content) {
                    continue;
                }

                // Skip function definition lines where the variant is being defined
                if trimmed.starts_with("function ") || trimmed.starts_with("def ") || trimmed.starts_with("fn ") {
                    if trimmed.contains(&variant) {
                        continue;
                    }
                }

                // Determine the calling function
                let caller_name = self.find_containing_function(&m.file, m.line)?;

                // Avoid duplicates (same caller, file, line)
                if !callers.iter().any(|existing: &CallerInfo| {
                    existing.caller_name == caller_name && existing.file == m.file && existing.line == m.line
                }) {
                    callers.push(CallerInfo {
                        caller_name,
                        file: m.file.clone(),
                        line: m.line,
                    });
                }
            }
        }

        Ok(callers)
    }

    /// Find the function that contains a given line (simplified implementation)
    ///
    /// Searches backwards from the given line to find the most recent function definition.
    fn find_containing_function(&self, file: &PathBuf, line: usize) -> Result<String> {
        let content = fs::read_to_string(file)?;

        let lines: Vec<&str> = content.lines().collect();

        // Search backwards from the line to find a function definition
        let function_patterns = vec![
            // JavaScript/TypeScript patterns
            Regex::new(r"function\s+(\w+)").unwrap(),
            Regex::new(r"(?:const|let|var)\s+(\w+)\s*=\s*(?:async\s+)?\([^)]*\)\s*=>").unwrap(),
            Regex::new(r"export\s+(?:const|let|var)\s+(\w+)\s*=\s*(?:async\s+)?\([^)]*\)\s*=>")
                .unwrap(),
            Regex::new(r"export\s+function\s+(\w+)").unwrap(),
            Regex::new(
                r"^\s*(?:public|private|protected|static)?\s*(?:async\s+)?(\w+)\s*\([^)]*\)\s*[:{]",
            )
            .unwrap(),
            // Ruby patterns
            Regex::new(r"def\s+(\w+)").unwrap(),
            Regex::new(r"def\s+self\.(\w+)").unwrap(),
            // Generic method pattern
            Regex::new(r"^\s*(\w+)\s*\([^)]*\)\s*\{").unwrap(),
            // Rust pattern (for completeness)
            Regex::new(r"fn\s+(\w+)").unwrap(),
        ];

        // Search backwards up to 100 lines or start of file
        let start = line.saturating_sub(100);
        for i in (start..line.saturating_sub(1)).rev() {
            if i >= lines.len() {
                continue;
            }

            let line_content = lines[i];
            for pattern in &function_patterns {
                if let Some(captures) = pattern.captures(line_content) {
                    if let Some(name_match) = captures.get(1) {
                        return Ok(name_match.as_str().to_string());
                    }
                }
            }
        }

        // If no containing function found, it might be top-level code
        Ok("<top-level>".to_string())
    }

    /// Generate case variants of a function name for cross-case searching
    ///
    /// For input "createUser" generates: ["createUser", "create_user", "CreateUser"]
    /// For input "user_profile" generates: ["user_profile", "userProfile", "UserProfile"]
    fn generate_case_variants(func_name: &str) -> Vec<String> {
        let mut variants = std::collections::HashSet::new();

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
}

impl Default for CallExtractor {
    fn default() -> Self {
        Self::new(std::env::current_dir().unwrap())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_call_extractor_creation() {
        let extractor = CallExtractor::new(std::env::current_dir().unwrap());
        assert!(!extractor.keywords.is_empty());
    }

    #[test]
    fn test_call_patterns() {
        let extractor = CallExtractor::new(std::env::current_dir().unwrap());
        let test_line = "result = processData(x, y);";

        let mut found_calls = false;
        for pattern in &extractor.call_patterns {
            if pattern.is_match(test_line) {
                found_calls = true;
                break;
            }
        }
        assert!(found_calls);
    }

    #[test]
    fn test_keywords_filter() {
        let extractor = CallExtractor::new(std::env::current_dir().unwrap());
        assert!(extractor.keywords.contains("if"));
        assert!(extractor.keywords.contains("for"));
        assert!(extractor.keywords.contains("while"));
    }
}
