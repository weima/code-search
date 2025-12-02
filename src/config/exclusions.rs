use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProjectType {
    Node,
    Ruby,
    Python,
    Rust,
    Go,
    Generic,
}

pub fn detect_project_type(base_dir: &Path) -> ProjectType {
    if base_dir.join("package.json").exists() {
        ProjectType::Node
    } else if base_dir.join("Gemfile").exists() {
        ProjectType::Ruby
    } else if base_dir.join("requirements.txt").exists()
        || base_dir.join("pyproject.toml").exists()
        || base_dir.join("setup.py").exists()
    {
        ProjectType::Python
    } else if base_dir.join("Cargo.toml").exists() {
        ProjectType::Rust
    } else if base_dir.join("go.mod").exists() {
        ProjectType::Go
    } else {
        ProjectType::Generic
    }
}

pub fn get_default_exclusions(project_type: ProjectType) -> Vec<&'static str> {
    let mut exclusions = vec![".git", ".svn", ".hg", ".idea", ".vscode", ".DS_Store"];

    match project_type {
        ProjectType::Node => {
            exclusions.extend_from_slice(&[
                "node_modules",
                "dist",
                "build",
                "coverage",
                ".next",
                ".nuxt",
            ]);
        }
        ProjectType::Ruby => {
            exclusions.extend_from_slice(&["vendor", ".bundle", "log", "tmp", "coverage"]);
        }
        ProjectType::Python => {
            exclusions.extend_from_slice(&[
                "venv",
                ".venv",
                "env",
                ".env",
                "__pycache__",
                "*.egg-info",
                ".pytest_cache",
                ".mypy_cache",
            ]);
        }
        ProjectType::Rust => {
            exclusions.extend_from_slice(&["target"]);
        }
        ProjectType::Go => {
            exclusions.extend_from_slice(&["vendor"]);
        }
        ProjectType::Generic => {
            // Add common exclusions for generic projects if needed
            exclusions.extend_from_slice(&[
                "node_modules", // Common enough to include by default?
                "vendor",
                "dist",
                "build",
                "target",
            ]);
        }
    }

    exclusions
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_detect_node_project() {
        let dir = tempdir().unwrap();
        fs::write(dir.path().join("package.json"), "{}").unwrap();
        assert_eq!(detect_project_type(dir.path()), ProjectType::Node);
    }

    #[test]
    fn test_detect_rust_project() {
        let dir = tempdir().unwrap();
        fs::write(dir.path().join("Cargo.toml"), "").unwrap();
        assert_eq!(detect_project_type(dir.path()), ProjectType::Rust);
    }

    #[test]
    fn test_detect_generic_project() {
        let dir = tempdir().unwrap();
        assert_eq!(detect_project_type(dir.path()), ProjectType::Generic);
    }

    #[test]
    fn test_default_exclusions_node() {
        let exclusions = get_default_exclusions(ProjectType::Node);
        assert!(exclusions.contains(&"node_modules"));
        assert!(exclusions.contains(&".git"));
    }
}
