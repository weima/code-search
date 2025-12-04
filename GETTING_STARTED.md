# Getting Started with Code Search (cs)

`code-search` (or `cs`) is a powerful CLI tool designed to help you navigate your codebase efficiently. It goes beyond simple text search by understanding the structure of your code, including i18n translation keys and function call graphs.

## Installation

Choose the installation method that works best for you:

### Homebrew (macOS/Linux)
```bash
brew tap weima/code-search https://github.com/weima/code-search
brew install cs
```

### NPM (Cross-platform)
```bash
npm install -g code-search-cli
```

### Cargo (Rust)
```bash
cargo install code-search-cli
```

## Basic Usage

### 1. Searching for Text
The most common use case is searching for UI text to find where it's defined and used.

```bash
cs "Add New"
```

This command will:
1.  Search for "Add New" in your translation files (e.g., `en.yml`).
2.  Find the translation key (e.g., `invoice.labels.add_new`).
3.  Find all occurrences of that key in your code (e.g., `t('invoice.labels.add_new')`).
4.  Also perform a direct text search for "Add New" to catch hardcoded strings.

### 2. Case Sensitivity
By default, searches are case-insensitive.

*   **Case-Insensitive (Default):** `cs "add new"` matches "Add New", "ADD NEW", etc.
*   **Case-Sensitive:** Use the `-s` or `--case-sensitive` flag.
    ```bash
    cs "Add New" -s
    ```
*   **Explicit Ignore Case:** Use `-i` or `--ignore-case` (useful if you have an alias that defaults to case-sensitive).
    ```bash
    cs "add new" -i
    ```

### 3. Word Matching
To match whole words only, use the `-w` or `--word-regexp` flag. This prevents partial matches (e.g., "cat" matching "category").

```bash
cs "user" -w
```

### 4. File Filtering
You can restrict your search to specific file types using the `-g` or `--glob` flag.

```bash
# Only search in Rust files
cs "struct" -g "*.rs"

# Only search in TypeScript and JavaScript files
cs "function" -g "*.ts" -g "*.js"
```

### 5. Regular Expressions
For more complex queries, you can use regular expressions with the `--regex` flag.

```bash
# Find patterns starting with "user_" followed by digits
cs "user_\d+" --regex
```

## Advanced Features

### Call Graph Tracing
`cs` can trace function calls to help you understand code flow without an IDE.

*   **Forward Trace (What does this function call?):**
    ```bash
    cs "processPayment" --trace
    ```

*   **Backward Trace (Who calls this function?):**
    ```bash
    cs "validateToken" --traceback
    ```

*   **Bidirectional Trace:**
    ```bash
    cs "updateUser" --trace-all
    ```

*   **Depth Control:**
    Limit the depth of the trace (default is 3).
    ```bash
    cs "main" --trace --depth 5
    ```

### Custom File Extensions
If you work with custom file extensions (e.g., `.vue`, `.erb`), `cs` usually detects them. If not, you can explicitly include them:

```bash
cs "search text" --include-extensions html.ui,vue.custom
```

## Troubleshooting

If you're not seeing the results you expect:
1.  **Check your current directory:** `cs` searches from the current working directory recursively.
2.  **Check `.gitignore`:** By default, `cs` respects `.gitignore`.
3.  **Verify file extensions:** Ensure your source files are being included (use `--include-extensions` if needed).

## Getting Help
For a full list of commands and options:

```bash
cs --help
```
