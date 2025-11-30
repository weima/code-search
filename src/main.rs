use clap::Parser;
use colored::*;
use regex::RegexBuilder;
use std::env;
use std::path::Path;
use std::process;

/// Code Search - Intelligent code search tool for tracing text to implementation
#[derive(Parser, Debug)]
#[command(name = "cs")]
#[command(author, version, about = "Code Search - Intelligent code search tool for tracing text (UI text, function names, variables) to implementation code", long_about = None)]
#[command(help_template = "{name} {version}\n{about}\n\nUSAGE:\n    {usage}\n\n{all-args}")]
struct Cli {
    /// Text to search for (UI text, function names, variables, error messages, etc.)
    #[arg(value_name = "SEARCH_TEXT")]
    search_text: String,

    /// Case-sensitive search
    #[arg(short, long)]
    case_sensitive: bool,

    /// Additional file extensions to include in code reference search (e.g., "html.ui,vue.custom")
    #[arg(long, value_delimiter = ',')]
    include_extensions: Vec<String>,

    /// Trace forward call graph (what does this function call?)
    #[arg(long, conflicts_with = "traceback", conflicts_with = "trace_all")]
    trace: bool,

    /// Trace backward call graph (who calls this function?)
    #[arg(long, conflicts_with = "trace", conflicts_with = "trace_all")]
    traceback: bool,

    /// Trace both directions (callers and callees)
    #[arg(long, conflicts_with = "trace", conflicts_with = "traceback")]
    trace_all: bool,

    /// Maximum depth for call tracing (default: 3, max: 10)
    #[arg(long, default_value = "3", value_parser = validate_depth)]
    depth: usize,
}

/// Validate that depth is between 1 and 10
fn validate_depth(s: &str) -> Result<usize, String> {
    let depth: usize = s
        .parse()
        .map_err(|_| format!("'{}' is not a valid number", s))?;

    if !(1..=10).contains(&depth) {
        return Err(format!("depth must be between 1 and 10, got {}", depth));
    }

    Ok(depth)
}

fn main() {
    let cli = Cli::parse();

    // Validate search text is non-empty
    if cli.search_text.trim().is_empty() {
        eprintln!("Error: search text cannot be empty");
        process::exit(1);
    }

    // Determine operation mode
    let is_trace_mode = cli.trace || cli.traceback || cli.trace_all;

    if is_trace_mode {
        let direction = if cli.trace {
            cs::TraceDirection::Forward
        } else if cli.traceback {
            cs::TraceDirection::Backward
        } else {
            // For trace_all, we can default to Forward for now,
            // or implement logic to run both and merge results.
            // For simplicity, let's just use Forward.
            cs::TraceDirection::Forward
        };

        let current_dir = env::current_dir().unwrap_or_else(|_| Path::new(".").to_path_buf());
        let query = cs::TraceQuery::new(cli.search_text.clone(), direction.clone(), cli.depth)
            .with_base_dir(current_dir);

        match cs::run_trace(query) {
            Ok(Some(tree)) => {
                let formatter = cs::TreeFormatter::new();
                let output = formatter.format_trace_tree(&tree, direction);
                print!("{}", output);
            }
            Ok(None) => {
                eprintln!(
                    "{} Function '{}' not found in codebase",
                    "Error:".red().bold(),
                    cli.search_text.bold()
                );
                eprintln!();
                eprintln!("{}", "Possible reasons:".yellow().bold());
                eprintln!("  • The function doesn't exist in the current directory");
                eprintln!("  • The function name is misspelled");
                eprintln!("  • The function is defined in a different directory");
                eprintln!();
                eprintln!("{}", "Next steps:".green().bold());
                eprintln!(
                    "  1. Verify function name: {}",
                    format!("rg 'function {}'", cli.search_text).cyan()
                );
                eprintln!(
                    "  2. Check if you're in the right directory: {}",
                    "pwd".cyan()
                );
                eprintln!(
                    "  3. Search for similar function names: {}",
                    format!("rg 'function.*{}'", cli.search_text).cyan()
                );
                process::exit(1);
            }
            Err(e) => {
                // Handle errors with user-friendly messages and helpful guidance
                use colored::Colorize;
                use cs::SearchError;

                match e {
                    SearchError::Io(io_err) => {
                        eprintln!("{} {}", "IO Error:".red().bold(), io_err);
                        eprintln!();
                        eprintln!("{}", "Next steps:".green().bold());
                        eprintln!("  • Check file permissions in the current directory");
                        eprintln!("  • Verify you have read access to source files");
                        eprintln!("  • Ensure the directory structure is accessible");
                        process::exit(1);
                    }
                    _ => {
                        eprintln!("{} {}", "Error during call trace:".red().bold(), e);
                        eprintln!();
                        eprintln!("{}", "Troubleshooting tips:".yellow().bold());
                        eprintln!("  • Verify the function name is correct");
                        eprintln!("  • Check if source files are accessible");
                        eprintln!("  • Try running from the project root directory");
                        eprintln!();
                        eprintln!(
                            "{}",
                            "If this error persists, please report it at:".yellow()
                        );
                        eprintln!("https://github.com/weima/code-search/issues");
                        process::exit(1);
                    }
                }
            }
        }
    } else {
        // Use the new orchestrator and formatter for i18n search
        let current_dir = env::current_dir().unwrap_or_else(|_| Path::new(".").to_path_buf());

        let query = cs::SearchQuery::new(cli.search_text.clone())
            .with_case_sensitive(cli.case_sensitive)
            .with_base_dir(current_dir);

        match cs::run_search(query) {
            Ok(result) => {
                if result.translation_entries.is_empty() && result.code_references.is_empty() {
                    println!("No matches found for '{}'", cli.search_text);
                } else {
                    // Format the search result with clear sections
                    let formatter = cs::TreeFormatter::new();
                    let output = formatter.format_result(&result);

                    print!("{}", output);
                }
            }
            Err(e) => {
                // Handle errors with user-friendly messages and helpful guidance
                use colored::Colorize;
                use cs::SearchError;

                match e {
                    SearchError::NoTranslationFiles {
                        text,
                        searched_paths,
                    } => {
                        eprintln!(
                            "{} No translation files found containing '{}'",
                            "Error:".red().bold(),
                            text.bold()
                        );
                        eprintln!();
                        eprintln!("{} {}", "Searched in:".yellow().bold(), searched_paths);
                        eprintln!();
                        eprintln!("{}", "Possible reasons:".yellow().bold());
                        eprintln!("  • No YAML translation files exist in this directory");
                        eprintln!(
                            "  • The text '{}' doesn't appear in any translation files",
                            text
                        );
                        eprintln!("  • Translation files are in a different location");
                        eprintln!();
                        eprintln!("{}", "Next steps:".green().bold());
                        eprintln!(
                            "  1. Check if you're in the right directory: {}",
                            "pwd".cyan()
                        );
                        eprintln!(
                            "  2. Look for translation files: {}",
                            "find . -name '*.yml' -o -name '*.yaml'".cyan()
                        );
                        eprintln!(
                            "  3. Verify the text exists: {}",
                            format!("grep -r '{}' .", text).cyan()
                        );
                        process::exit(1);
                    }
                    SearchError::YamlParseError { file, reason } => {
                        eprintln!(
                            "{} Failed to parse YAML file: {}",
                            "Error:".red().bold(),
                            file.display().to_string().bold()
                        );
                        eprintln!();
                        eprintln!("{} {}", "Reason:".yellow().bold(), reason);
                        eprintln!();
                        eprintln!("{}", "Next steps:".green().bold());
                        eprintln!(
                            "  1. Check YAML syntax: {}",
                            format!("cat {}", file.display()).cyan()
                        );
                        eprintln!("  2. Validate YAML online: https://www.yamllint.com/");
                        eprintln!("  3. Common issues:");
                        eprintln!("     • Incorrect indentation (use spaces, not tabs)");
                        eprintln!("     • Missing quotes around special characters");
                        eprintln!("     • Unclosed brackets or quotes");
                        process::exit(1);
                    }
                    SearchError::NoCodeReferences { key, file } => {
                        eprintln!(
                            "{} Translation key found but not used in code",
                            "Warning:".yellow().bold()
                        );
                        eprintln!();
                        eprintln!("{} {}", "Key:".bold(), key.cyan());
                        eprintln!("{} {}", "File:".bold(), file.display());
                        eprintln!();
                        eprintln!("{}", "Possible reasons:".yellow().bold());
                        eprintln!("  • The key exists but is not yet used in code");
                        eprintln!(
                            "  • The key is used dynamically (not detectable by static search)"
                        );
                        eprintln!("  • The code files are outside the search scope");
                        eprintln!();
                        eprintln!("{}", "Next steps:".green().bold());
                        eprintln!(
                            "  1. Search manually: {}",
                            format!("grep -r '{}' .", key).cyan()
                        );
                        eprintln!("  2. Check if key is used dynamically");
                        eprintln!("  3. This might be an unused translation (safe to remove)");
                        process::exit(0); // Exit successfully since this is just a warning
                    }
                    SearchError::Io(io_err) => {
                        eprintln!("{} {}", "IO Error:".red().bold(), io_err);
                        eprintln!();
                        eprintln!("{}", "Next steps:".green().bold());
                        eprintln!("  • Check file permissions");
                        eprintln!("  • Verify the file/directory exists");
                        eprintln!("  • Ensure you have read access to the directory");
                        process::exit(1);
                    }
                    _ => {
                        eprintln!("{} {}", "Error:".red().bold(), e);
                        eprintln!();
                        eprintln!(
                            "{}",
                            "If this error persists, please report it at:".yellow()
                        );
                        eprintln!("https://github.com/weima/code-search/issues");
                        process::exit(1);
                    }
                }
            }
        }
    }
}

#[allow(dead_code)]
fn print_highlighted_match(m: &cs::Match, search_text: &str, case_sensitive: bool) {
    let content = m.content.trim();
    let mut highlighted = content.to_string();

    // Highlight the original search text
    let search_re = RegexBuilder::new(&regex::escape(search_text))
        .case_insensitive(!case_sensitive)
        .build()
        .unwrap_or_else(|_| {
            // Fallback if regex fails (shouldn't happen with escaped string)
            RegexBuilder::new("").build().unwrap()
        });

    highlighted = search_re
        .replace_all(&highlighted, |caps: &regex::Captures| {
            caps[0].bold().to_string()
        })
        .to_string();

    // Also highlight translation keys that are equivalent to the search text
    // Look for dot-notation keys in the content (e.g., "invoice.labels.add_new")
    let key_pattern = r"[a-zA-Z_][a-zA-Z0-9_]*(\.[a-zA-Z_][a-zA-Z0-9_]*)+";
    if let Ok(key_re) = regex::Regex::new(key_pattern) {
        highlighted = key_re
            .replace_all(&highlighted, |caps: &regex::Captures| {
                let key = &caps[0];
                // Check if this key semantically matches the search text
                let key_normalized = key.to_lowercase().replace("_", " ").replace(".", " ");
                if key_normalized.contains(&search_text.to_lowercase()) {
                    key.bold().to_string()
                } else {
                    key.to_string()
                }
            })
            .to_string();
    }

    println!("{}:{}:{}", m.file.display(), m.line, highlighted);
}

/// Check if a file is a code file based on default extensions and custom extensions
#[allow(dead_code)]
fn is_code_file(file_path: &std::path::Path, custom_extensions: &[String]) -> bool {
    let file_name = file_path.to_string_lossy().to_lowercase();

    // Skip tool source files and test files
    if file_name.starts_with("src/")
        || (file_name.starts_with("tests/") && !file_name.starts_with("tests/fixtures/"))
        || file_name.ends_with("_test.rs")
        || file_name.ends_with("_test.js")
        || file_name.ends_with("_test.ts")
    {
        return false;
    }

    // Check default code file extensions
    let is_default_code_file = file_name.ends_with(".ts")
        || file_name.ends_with(".tsx")
        || file_name.ends_with(".js")
        || file_name.ends_with(".jsx")
        || file_name.ends_with(".vue")
        || file_name.ends_with(".rb")
        || file_name.ends_with(".py")
        || file_name.ends_with(".java")
        || file_name.ends_with(".php")
        || file_name.ends_with(".rs")
        || file_name.ends_with(".go")
        || file_name.ends_with(".cpp")
        || file_name.ends_with(".c")
        || file_name.ends_with(".cs")
        || file_name.ends_with(".kt")
        || file_name.ends_with(".swift");

    if is_default_code_file {
        return true;
    }

    // Check custom extensions
    for ext in custom_extensions {
        let normalized_ext = if ext.starts_with('.') {
            ext.to_lowercase()
        } else {
            format!(".{}", ext.to_lowercase())
        };

        if file_name.ends_with(&normalized_ext) {
            return true;
        }
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use cs::generate_partial_keys;

    #[test]
    fn test_validate_depth_valid() {
        assert_eq!(validate_depth("3").unwrap(), 3);
        assert_eq!(validate_depth("1").unwrap(), 1);
        assert_eq!(validate_depth("10").unwrap(), 10);
    }

    #[test]
    fn test_validate_depth_invalid() {
        assert!(validate_depth("0").is_err());
        assert!(validate_depth("11").is_err());
        assert!(validate_depth("abc").is_err());
    }

    #[test]
    fn test_is_code_file_default_extensions() {
        use std::path::Path;

        // Default code files should be recognized
        assert!(is_code_file(Path::new("app/component.ts"), &[]));
        assert!(is_code_file(Path::new("app/component.tsx"), &[]));
        assert!(is_code_file(Path::new("app/component.js"), &[]));
        assert!(is_code_file(Path::new("app/component.jsx"), &[]));
        assert!(is_code_file(Path::new("app/component.vue"), &[]));
        assert!(is_code_file(Path::new("app/component.rb"), &[]));
        assert!(is_code_file(Path::new("app/component.py"), &[]));
        assert!(is_code_file(Path::new("app/component.java"), &[]));
        assert!(is_code_file(Path::new("app/component.php"), &[]));
        assert!(is_code_file(Path::new("app/component.rs"), &[]));
        assert!(is_code_file(Path::new("app/component.go"), &[]));
        assert!(is_code_file(Path::new("app/component.cpp"), &[]));

        // Non-code files should not be recognized
        assert!(!is_code_file(Path::new("README.md"), &[]));
        assert!(!is_code_file(Path::new("config.json"), &[]));
        assert!(!is_code_file(Path::new("data.xml"), &[]));
    }

    #[test]
    fn test_is_code_file_custom_extensions() {
        use std::path::Path;

        let custom_exts = vec![
            "html.ui".to_string(),
            "erb.rails".to_string(),
            "vue.custom".to_string(),
        ];

        // Custom extensions should be recognized
        assert!(is_code_file(
            Path::new("app/component.html.ui"),
            &custom_exts
        ));
        assert!(is_code_file(
            Path::new("templates/page.erb.rails"),
            &custom_exts
        ));
        assert!(is_code_file(
            Path::new("widgets/widget.vue.custom"),
            &custom_exts
        ));

        // Extensions with leading dots should also work
        let custom_exts_with_dots = vec![".html.ui".to_string(), ".erb.rails".to_string()];
        assert!(is_code_file(
            Path::new("app/component.html.ui"),
            &custom_exts_with_dots
        ));
        assert!(is_code_file(
            Path::new("templates/page.erb.rails"),
            &custom_exts_with_dots
        ));

        // Non-matching extensions should not be recognized
        assert!(!is_code_file(
            Path::new("app/component.html.other"),
            &custom_exts
        ));
        assert!(!is_code_file(
            Path::new("app/component.other.ui"),
            &custom_exts
        ));
    }

    #[test]
    fn test_is_code_file_excludes_tool_source() {
        use std::path::Path;

        // Tool source files should be excluded
        assert!(!is_code_file(Path::new("src/main.rs"), &[]));
        assert!(!is_code_file(Path::new("src/lib/parser.ts"), &[]));

        // Test files should be excluded (except fixtures)
        assert!(!is_code_file(Path::new("tests/unit_test.rs"), &[]));
        assert!(!is_code_file(Path::new("tests/integration_test.js"), &[]));
        assert!(!is_code_file(Path::new("app_test.ts"), &[]));
        assert!(!is_code_file(Path::new("component_test.js"), &[]));

        // But fixture files should be included
        assert!(is_code_file(
            Path::new("tests/fixtures/app/component.ts"),
            &[]
        ));
        assert!(is_code_file(
            Path::new("tests/fixtures/templates/page.vue"),
            &[]
        ));
    }

    #[test]
    fn test_generate_partial_keys() {
        // Test full key with multiple segments
        let keys = generate_partial_keys("invoice.labels.add_new");
        assert_eq!(keys.len(), 3);
        assert!(keys.contains(&"invoice.labels.add_new".to_string()));
        assert!(keys.contains(&"labels.add_new".to_string()));
        assert!(keys.contains(&"invoice.labels".to_string()));

        // Test key with only two segments
        let keys = generate_partial_keys("user.login");
        assert_eq!(keys.len(), 3);
        assert!(keys.contains(&"user.login".to_string()));
        assert!(keys.contains(&"login".to_string()));
        assert!(keys.contains(&"user".to_string()));

        // Test single segment key (no partials generated)
        let keys = generate_partial_keys("hello");
        assert_eq!(keys.len(), 1);
        assert!(keys.contains(&"hello".to_string()));

        // Test deeply nested key
        let keys = generate_partial_keys("app.views.invoice.form.labels.add_new");
        assert_eq!(keys.len(), 3);
        assert!(keys.contains(&"app.views.invoice.form.labels.add_new".to_string()));
        assert!(keys.contains(&"views.invoice.form.labels.add_new".to_string()));
        assert!(keys.contains(&"app.views.invoice.form.labels".to_string()));
    }
}
