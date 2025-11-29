# Contributing to Code Search

Thanks for your interest in contributing! This document outlines the guidelines for contributing to this project.

## Getting Started

1. Fork the repository
2. Clone your fork: `git clone https://github.com/YOUR_USERNAME/code-search.git`
3. Install Rust: https://rustup.rs/
4. Install ripgrep: https://github.com/BurntSushi/ripgrep#installation
5. Build the project: `cargo build`
6. Run tests: `cargo test`

## Development Setup

```bash
# Install development tools
rustup component add clippy rustfmt

# Run linter
cargo clippy

# Format code
cargo fmt

# Run tests with coverage (optional)
cargo install cargo-tarpaulin
cargo tarpaulin
```

## Code Style

### Rust Guidelines

- Follow the [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- Run `cargo fmt` before committing
- Run `cargo clippy` and fix all warnings
- Use meaningful variable and function names
- Prefer `snake_case` for functions and variables
- Prefer `PascalCase` for types and traits

### Documentation

- Add doc comments (`///`) to all public functions, structs, and modules
- Include examples in doc comments where helpful
- Keep comments concise and up-to-date

```rust
/// Searches for function definitions matching the given name.
///
/// # Arguments
/// * `name` - The function name to search for
///
/// # Returns
/// A vector of `FunctionDef` structs, or an error if the search fails.
///
/// # Example
/// ```
/// let finder = FunctionFinder::new();
/// let defs = finder.find_definition("main")?;
/// ```
pub fn find_definition(&self, name: &str) -> Result<Vec<FunctionDef>> {
    // ...
}
```

## Commit Messages

Follow [Conventional Commits](https://www.conventionalcommits.org/):

```
<type>(<scope>): <description>

[optional body]

[optional footer]
```

### Types

- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation only
- `style`: Code style (formatting, no logic change)
- `refactor`: Code change that neither fixes a bug nor adds a feature
- `test`: Adding or updating tests
- `chore`: Build process, dependencies, or tooling

### Examples

```
feat(trace): add --traceback flag for reverse call tracing

fix(yaml): handle malformed YAML files without crashing

docs(readme): add installation instructions for Windows

test(search): add integration tests for multi-file matches
```

## Branching Strategy

We use a simplified Git Flow:

```
main (stable, release-ready)
  └── feat/feature-name (feature development)
  └── fix/bug-description (bug fixes)
  └── docs/what-changed (documentation)
```

### Branch Naming

| Type | Pattern | Example |
|------|---------|---------|
| Feature | `feat/<short-description>` | `feat/call-tracing` |
| Bug fix | `fix/<issue-or-description>` | `fix/yaml-parse-crash` |
| Documentation | `docs/<what>` | `docs/readme-examples` |
| Refactor | `refactor/<what>` | `refactor/error-handling` |
| Test | `test/<what>` | `test/integration-suite` |

### Branch Rules

- **`main`** is protected:
  - No direct pushes
  - Requires PR with at least 1 approval (when we have more contributors)
  - All CI checks must pass
  - Branch must be up-to-date before merging

- **Feature branches**:
  - Branch from `main`
  - Keep focused on a single feature/fix
  - Rebase on `main` before PR if needed
  - Delete after merge

### Workflow

```bash
# Start new feature
git checkout main
git pull origin main
git checkout -b feat/my-feature

# Work on feature...
git add .
git commit -m "feat: add my feature"

# Keep up to date with main
git fetch origin
git rebase origin/main

# Push and create PR
git push -u origin feat/my-feature
```

## Pull Request Process

1. Create a feature branch from `main`:
   ```bash
   git checkout -b feat/your-feature-name
   ```

2. Make your changes with clear, atomic commits

3. Ensure all checks pass:
   ```bash
   cargo fmt --check
   cargo clippy
   cargo test
   ```

4. Update documentation if needed (README, doc comments)

5. Rebase on main if your branch is behind:
   ```bash
   git fetch origin
   git rebase origin/main
   ```

6. Push and create a Pull Request

7. Fill out the PR template with:
   - What the change does
   - Why it's needed
   - How to test it

### PR Checklist

- [ ] Code compiles without warnings (`cargo build`)
- [ ] All tests pass (`cargo test`)
- [ ] Linter passes (`cargo clippy`)
- [ ] Code is formatted (`cargo fmt`)
- [ ] New code has tests (if applicable)
- [ ] Documentation updated (if applicable)

## Testing

### Running Tests

```bash
# Run all tests
cargo test

# Run specific test
cargo test test_name

# Run tests with output
cargo test -- --nocapture

# Run integration tests only
cargo test --test '*'
```

### Writing Tests

- Place unit tests in the same file as the code, in a `tests` module
- Place integration tests in `tests/` directory
- Use descriptive test names: `test_<function>_<scenario>_<expected>`
- Test both success and error cases

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_definition_returns_match() {
        let finder = FunctionFinder::new();
        let result = finder.find_definition("main");
        assert!(result.is_ok());
    }

    #[test]
    fn test_find_definition_not_found_returns_empty() {
        let finder = FunctionFinder::new();
        let result = finder.find_definition("nonexistent").unwrap();
        assert!(result.is_empty());
    }
}
```

## Reporting Issues

### Bug Reports

Include:
- Rust version (`rustc --version`)
- OS and version
- Steps to reproduce
- Expected vs actual behavior
- Error messages (full output)

### Feature Requests

Include:
- Use case / problem you're trying to solve
- Proposed solution (if any)
- Alternatives considered

## Code of Conduct

- Be respectful and inclusive
- Assume good intentions
- Focus on the code, not the person
- Welcome newcomers and help them learn

## Questions?

Open an issue with the `question` label or start a discussion.

---

Thank you for contributing! 🎉
