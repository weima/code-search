use clap::Parser;
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
        // Default i18n search mode
        println!("I18n search mode");
        println!("Search text: {}", cli.search_text);
        println!("Case sensitive: {}", cli.case_sensitive);

        // TODO: Implement i18n search functionality
        eprintln!("I18n search not yet implemented");
        process::exit(1);
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
