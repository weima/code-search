use crate::error::Result;
use ignore::WalkBuilder;
use std::path::PathBuf;

/// Result of a file search
#[derive(Debug, Clone)]
pub struct FileMatch {
    pub path: PathBuf,
}

/// File searcher that finds files by name pattern
pub struct FileSearcher {
    base_dir: PathBuf,
    case_sensitive: bool,
    exclusions: Vec<String>,
}

impl FileSearcher {
    pub fn new(base_dir: PathBuf) -> Self {
        Self {
            base_dir,
            case_sensitive: false,
            exclusions: Vec::new(),
        }
    }

    pub fn case_sensitive(mut self, value: bool) -> Self {
        self.case_sensitive = value;
        self
    }

    pub fn add_exclusions(mut self, exclusions: Vec<String>) -> Self {
        self.exclusions.extend(exclusions);
        self
    }

    /// Search for files matching the pattern
    pub fn search(&self, pattern: &str) -> Result<Vec<FileMatch>> {
        let mut matches = Vec::new();

        let pattern_lower = if self.case_sensitive {
            pattern.to_string()
        } else {
            pattern.to_lowercase()
        };

        let walker = WalkBuilder::new(&self.base_dir)
            .git_ignore(true)
            .git_global(true)
            .git_exclude(true)
            .hidden(false)
            .build();

        for entry in walker.filter_map(|e| e.ok()) {
            if !entry.file_type().map(|ft| ft.is_file()).unwrap_or(false) {
                continue;
            }

            let path = entry.path();

            // Apply exclusions
            let path_str = path.to_string_lossy();
            if self
                .exclusions
                .iter()
                .any(|ex| path_str.contains(ex.as_str()))
            {
                continue;
            }

            // Check if filename matches pattern
            if let Some(file_name) = path.file_name() {
                let file_name_str = file_name.to_string_lossy();
                let file_name_compare = if self.case_sensitive {
                    file_name_str.to_string()
                } else {
                    file_name_str.to_lowercase()
                };

                if file_name_compare.contains(&pattern_lower) {
                    matches.push(FileMatch {
                        path: path.to_path_buf(),
                    });
                }
            }
        }

        Ok(matches)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_file_search_basic() {
        let temp_dir = TempDir::new().unwrap();
        fs::write(temp_dir.path().join("test.txt"), "content").unwrap();
        fs::write(temp_dir.path().join("other.txt"), "content").unwrap();

        let searcher = FileSearcher::new(temp_dir.path().to_path_buf());
        let matches = searcher.search("test").unwrap();

        assert_eq!(matches.len(), 1);
        assert!(matches[0].path.to_string_lossy().contains("test.txt"));
    }

    #[test]
    fn test_file_search_case_insensitive() {
        let temp_dir = TempDir::new().unwrap();
        fs::write(temp_dir.path().join("Test.txt"), "content").unwrap();

        let searcher = FileSearcher::new(temp_dir.path().to_path_buf());
        let matches = searcher.search("test").unwrap();

        assert_eq!(matches.len(), 1);
    }

    #[test]
    fn test_file_search_case_sensitive() {
        let temp_dir = TempDir::new().unwrap();
        fs::write(temp_dir.path().join("Test.txt"), "content").unwrap();

        let searcher = FileSearcher::new(temp_dir.path().to_path_buf()).case_sensitive(true);
        let matches = searcher.search("test").unwrap();

        assert_eq!(matches.len(), 0);
    }
}
