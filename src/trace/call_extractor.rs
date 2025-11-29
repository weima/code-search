use regex::Regex;
use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;
use crate::search::TextSearcher;
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
    call_pattern: Regex,
    keywords: HashSet<String>,
}

impl CallExtractor {
    pub fn new() -> Self {
        Self {
            searcher: TextSearcher::new(),
            call_pattern: Regex::new(r"(\w+)\s*\(").unwrap(),
            keywords: Self::common_keywords(),
        }
    }

    /// Common language keywords to filter out (not function calls)
    fn common_keywords() -> HashSet<String> {
        let keywords = vec![
            // JavaScript/TypeScript
            "if", "for", "while", "switch", "catch", "typeof", "instanceof",
            // Ruby
            "puts", "print", "require", "include", "attr_accessor",
            // Python
            "print", "len", "range", "str", "int", "float", "list", "dict",
            // Rust
            "println", "print", "vec", "Some", "None", "Ok", "Err",
            // Common
            "return", "new", "delete", "throw",
        ];
        keywords.into_iter().map(String::from).collect()
    }

    /// Extract function calls from a function body
    pub fn extract_calls(&self, func: &FunctionDef) -> Result<Vec<String>, String> {
        // Read the file
        let content = fs::read_to_string(&func.file)
            .map_err(|e| format!("Failed to read file {:?}: {}", func.file, e))?;

        let lines: Vec<&str> = content.lines().collect();
        
        // Find the function body (simplified - just look at next 20 lines)
        let start_line = func.line.saturating_sub(1);
        let end_line = (start_line + 20).min(lines.len());
        
        let mut calls = HashSet::new();
        
        for line in &lines[start_line..end_line] {
            // Find all function calls in this line
            for cap in self.call_pattern.captures_iter(line) {
                if let Some(name_match) = cap.get(1) {
                    let name = name_match.as_str().to_string();
                    
                    // Filter out keywords and the function itself
                    if !self.keywords.contains(&name) && name != func.name {
                        calls.insert(name);
                    }
                }
            }
        }

        Ok(calls.into_iter().collect())
    }

    /// Find all functions that call the given function
    pub fn find_callers(&self, func_name: &str) -> Result<Vec<CallerInfo>, String> {
        let mut callers = Vec::new();

        // Search for function calls
        let call_pattern = format!(r"{}[\s]*\(", func_name);
        let matches = self.searcher
            .search(&call_pattern)
            .map_err(|e| format!("Search failed: {}", e))?;

        for m in matches {
            // Try to determine the containing function
            // This is simplified - in a real implementation, we'd parse the AST
            let caller_name = self.find_containing_function(&m.file, m.line)?;
            
            callers.push(CallerInfo {
                caller_name,
                file: m.file.clone(),
                line: m.line,
            });
        }

        if callers.is_empty() {
            Err(format!("No callers found for function '{}'", func_name))
        } else {
            Ok(callers)
        }
    }

    /// Find the function that contains a given line (simplified implementation)
    fn find_containing_function(&self, file: &PathBuf, line: usize) -> Result<String, String> {
        let content = fs::read_to_string(file)
            .map_err(|e| format!("Failed to read file {:?}: {}", file, e))?;

        let lines: Vec<&str> = content.lines().collect();
        
        // Search backwards from the line to find a function definition
        let function_patterns = vec![
            Regex::new(r"function\s+(\w+)").unwrap(),
            Regex::new(r"(?:const|let|var)\s+(\w+)\s*=").unwrap(),
            Regex::new(r"def\s+(\w+)").unwrap(),
            Regex::new(r"fn\s+(\w+)").unwrap(),
            Regex::new(r"^\s*(\w+)\s*\(").unwrap(),
        ];

        for i in (0..line.saturating_sub(1)).rev() {
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

        Ok("unknown".to_string())
    }
}

impl Default for CallExtractor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_call_extractor_creation() {
        let extractor = CallExtractor::new();
        assert!(!extractor.keywords.is_empty());
    }

    #[test]
    fn test_call_pattern() {
        let extractor = CallExtractor::new();
        let test_line = "result = processData(x, y);";
        
        let captures: Vec<_> = extractor.call_pattern.captures_iter(test_line).collect();
        assert!(!captures.is_empty());
    }

    #[test]
    fn test_keywords_filter() {
        let extractor = CallExtractor::new();
        assert!(extractor.keywords.contains("if"));
        assert!(extractor.keywords.contains("for"));
        assert!(extractor.keywords.contains("while"));
    }
}
