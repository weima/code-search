use clap::Parser;
use std::env;
use std::process;

/// Code Search - Trace text to implementation code
#[derive(Parser, Debug)]
#[command(name = "cs")]
#[command(author, version, about, long_about = None)]
#[command(help_template = "{name} {version}\n{about}\n\nUSAGE:\n    {usage}\n\n{all-args}")]
struct Cli {
    /// Text to search for (UI text, function names, variables, error messages, etc.)
    #[arg(value_name = "SEARCH_TEXT")]
    search_text: String,

    /// Case-sensitive search
    #[arg(short, long)]
    case_sensitive: bool,

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

    if depth < 1 || depth > 10 {
        return Err(format!("depth must be between 1 and 10, got {}", depth));
    }

    Ok(depth)
}

use colored::*;
use regex::RegexBuilder;
use std::path::Path;

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
        println!("Trace mode enabled");
        println!("Search text: {}", cli.search_text);
        println!("Depth: {}", cli.depth);

        if cli.trace {
            println!("Direction: Forward (what does '{}' call?)", cli.search_text);
        } else if cli.traceback {
            println!("Direction: Backward (who calls '{}'?)", cli.search_text);
        } else if cli.trace_all {
            println!(
                "Direction: Both (callers and callees of '{}')",
                cli.search_text
            );
        }

        // TODO: Implement call tracing functionality
        eprintln!("Call tracing not yet implemented");
        process::exit(1);
    } else {
        let mut all_matches = Vec::new();

        // First, search translation files using KeyExtractor
        let current_dir = env::current_dir().unwrap_or_else(|_| Path::new(".").to_path_buf());
        let key_extractor = cs::KeyExtractor::new();
        let searcher = cs::search::TextSearcher::new().case_sensitive(cli.case_sensitive);

        match key_extractor.extract(&current_dir, &cli.search_text) {
            Ok(translation_matches) => {
                for entry in translation_matches {
                    // Add the translation file match
                    let translation_match = cs::Match {
                        file: entry.file,
                        line: entry.line,
                        content: format!("{}: \"{}\"", entry.key, entry.value),
                    };
                    all_matches.push(translation_match);

                    // Search for code that uses this translation key
                    match searcher.search(&entry.key) {
                        Ok(key_matches) => {
                            all_matches.extend(key_matches);
                        }
                        Err(_) => {
                            // Ignore errors when searching for specific keys
                        }
                    }
                }
            }
            Err(_) => {
                // Ignore translation search errors and continue with code search
            }
        }

        // Then, search code files for the original text using TextSearcher
        match searcher.search(&cli.search_text) {
            Ok(code_matches) => {
                all_matches.extend(code_matches);
            }
            Err(e) => {
                eprintln!("Error: {}", e);
                process::exit(1);
            }
        }

        if all_matches.is_empty() {
            println!("No matches found for '{}'", cli.search_text);
        } else {
            for m in all_matches {
                let content = m.content.trim();

                // Highlight the match
                let pattern = &cli.search_text;

                let re = RegexBuilder::new(&regex::escape(pattern))
                    .case_insensitive(!cli.case_sensitive)
                    .build()
                    .unwrap_or_else(|_| {
                        // Fallback if regex fails (shouldn't happen with escaped string)
                        RegexBuilder::new("").build().unwrap()
                    });

                let highlighted =
                    re.replace_all(content, |caps: &regex::Captures| caps[0].bold().to_string());

                println!("{}:{}:{}", m.file.display(), m.line, highlighted);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
