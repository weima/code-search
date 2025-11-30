# Code Search (cs)

**Intelligent code search tool for tracing text to implementation code**

## Problem Statement

Modern IDEs like JetBrains suite provide powerful code navigation features through context menus - find definition, find references, etc. However, these IDEs are resource-intensive and not always practical for quick searches or lightweight environments.

Developers frequently need to search for text in their codebase, whether it's:
- **UI text** from user bug reports: "The 'Add New' button isn't working"
- **Function names** for understanding code flow: "What does `processPayment` call?"
- **Variable names** for refactoring: "Where is `userId` used?"
- **Error messages** for debugging: "Where does 'Invalid token' come from?"

For UI text specifically, the search is even more complex when i18n is involved:

1. Search for the text: `rg 'add new' -F`
2. Manually scan results to find translation files (e.g., `en.yml`)
3. Open the translation file and locate the key: `add_new: 'add new'`
4. Examine the YAML structure to find the full key path: `invoice.labels.add_new`
5. Search for the key usage: `rg 'invoice.labels.add_new' -F`
6. Finally locate the implementation in `components/invoices.ts`

This manual process is time-consuming, error-prone, and interrupts the development flow.

## Solution

`code-search` (abbreviated as `cs`) is a lightweight CLI tool that automates code discovery workflows:
- **Smart text search**: Find any text (UI text, code, error messages) in your codebase
- **i18n-aware tracing**: Automatically follows references from UI text through translation files to implementation
- **Call graph tracing**: Trace function calls forward or backward to understand code flow

### i18n Text Tracing

```bash
$ cs 'add new'

'add new'
   |
   |-> 'add_new: add new' at line 56 of en.yml
                |
                |-> 'invoice.labels.add_new' as the structure
                         |
                         |-> I18n.t('invoice.labels.add_new') at line 128 of components/invoices.ts
```

### Call Graph Tracing

Trace function calls forward (what does this function call?) or backward (who calls this function?):

```bash
# Forward trace - what does bar() call?
$ cs 'bar' --trace

bar
|-> zoo1 (utils.ts:45)
|-> zoo2 (helpers.ts:23)
|-> zoo3 (api.ts:89)

# Backward trace - who calls bar()?
$ cs 'bar' --traceback

blah1 -> foo1 -> bar
blah2 -> foo2 -> bar

# Control trace depth (default: 3, max: 10)
$ cs 'bar' --trace --depth 5
```

## Key Features

- **Universal Text Search**: Find any text - UI text, function names, variable names, error messages
- **Smart i18n Tracing**: Automatically follows references from UI text through translation files to implementation
- **Call Graph Tracing**: Trace function calls forward (`--trace`) or backward (`--traceback`)
- **i18n Format Support**: Understands YAML/JSON translation file structures
- **Pattern Recognition**: Identifies common i18n patterns (I18n.t, t(), $t, etc.) and function definitions
- **Tree Visualization**: Clear visual representation of the reference chain
- **Depth Control**: Configurable trace depth to prevent explosion in large codebases
- **Cycle Detection**: Handles recursive/circular calls without hanging
- **Lightweight**: Uses ripgrep library for fast performance (no external dependencies)
- **No IDE Required**: Works in any terminal environment

## Use Cases

- **Bug Triage**: Quickly locate implementation from user-reported issues (UI text or error messages)
- **Code Exploration**: Find where functions, variables, or constants are defined and used
- **Call Flow Analysis**: Understand what a function does by tracing its calls
- **Impact Analysis**: Find all callers of a function before refactoring
- **i18n Workflow**: Trace UI text through translation files to verify correct implementation
- **Debugging**: Locate error message sources or trace variable usage
- **Onboarding**: Help new developers understand code organization and data flow
- **Quick Navigation**: Fast code navigation without heavy IDE overhead

## Supported Patterns

### Translation File Formats
- YAML (Rails i18n, Ruby)
- JSON (JavaScript/TypeScript i18n)
- Properties files (Java)

### i18n Function Patterns
- Ruby: `I18n.t('key')`, `t('key')`
- JavaScript/TypeScript: `i18n.t('key')`, `$t('key')`, `t('key')`
- React: `useTranslation()`, `<Trans>`
- Vue: `$t('key')`, `{{ $t('key') }}`

## Custom File Extensions

By default, the tool recognizes common code file extensions (`.ts`, `.tsx`, `.js`, `.jsx`, `.vue`, `.rb`, `.py`, `.java`, `.php`, `.rs`, `.go`, `.cpp`, `.c`, `.cs`, `.kt`, `.swift`). 

For projects with custom file extensions, use the `--include-extensions` flag:

```bash
# Include files with custom extensions
cs "search text" --include-extensions html.ui,vue.custom

# Multiple extensions (comma-separated)
cs "search text" --include-extensions erb.rails,blade.php,twig.html

# Extensions with or without leading dot work the same
cs "search text" --include-extensions .html.ui,.vue.custom
```

This is particularly useful for:
- Custom framework file extensions (e.g., `.html.ui` for UI frameworks)
- Template engines with compound extensions (e.g., `.erb.rails`, `.blade.php`)
- Domain-specific file types (e.g., `.vue.custom`, `.component.ts`)

## Partial Key Matching

The tool automatically finds common i18n patterns where developers cache namespaces:

```javascript
// Common pattern: Cache namespace to avoid repetition
const labels = I18n.t('invoice.labels');
const addButton = labels.t('add_new');
const editButton = labels.t('edit');

// Deeper namespace caching
const invoiceNS = I18n.t('invoice');
const addLabel = invoiceNS.labels.t('add_new');
```

When searching for "add new", the tool finds:
- **Translation file**: `invoice.labels.add_new: "add new"`
- **Namespace usage**: `I18n.t('invoice.labels')` (parent namespace)
- **Relative key usage**: `labels.t('add_new')` (child key)

This works by generating strategic partial keys:
- **Full key**: `invoice.labels.add_new`
- **Without first segment**: `labels.add_new` (matches `labels.t('add_new')`)
- **Without last segment**: `invoice.labels` (matches `I18n.t('invoice.labels')`)

### Example Usage

```bash
# Basic i18n search
$ cs "add new"
=== Translation Files ===
config/locales/en.yml:4:invoice.labels.add_new: "add new"

=== Code References ===
app/components/invoices.ts:14:I18n.t('invoice.labels.add_new')
components/InvoiceManager.vue:3:{{ $t('invoice.labels.add_new') }}

# Include custom file types
$ cs "add new" --include-extensions html.ui,erb.rails
=== Translation Files ===
config/locales/en.yml:4:invoice.labels.add_new: "add new"

=== Code References ===
app/components/invoices.ts:14:I18n.t('invoice.labels.add_new')
components/InvoiceManager.vue:3:{{ $t('invoice.labels.add_new') }}
templates/invoice.html.ui:23:i18n('invoice.labels.add_new')
views/invoice.erb.rails:45:<%= t('invoice.labels.add_new') %>
```

## Roadmap

### Phase 1: Core Functionality
- [x] Project setup and architecture
- [ ] YAML translation file parsing
- [ ] Text-to-key mapping
- [ ] Key-to-code tracing
- [ ] Basic tree visualization

### Phase 2: Call Graph Tracing
- [ ] Function definition detection (JS, Ruby, Python, Rust)
- [ ] Forward call tracing (`--trace`)
- [ ] Backward call tracing (`--traceback`)
- [ ] Depth limiting and cycle detection

### Phase 3: Enhanced Features
- [ ] JSON translation support
- [ ] Multiple i18n pattern detection
- [ ] Tree-sitter for improved accuracy
- [ ] Configuration file support

### Phase 4: Advanced Features
- [ ] Interactive navigation
- [ ] Multi-language project support
- [ ] Caching for performance
- [ ] Editor integration (VSCode, Vim)

## Architecture

Built on a foundation of proven tools:
- **ripgrep library**: Embedded fast text searching (no external installation required)
- **Regex patterns**: Function definition and call detection
- **YAML/JSON parsers**: Translation file processing
- **Tree builders**: Visual output formatting

## Installation

### Homebrew (macOS/Linux)
```bash
brew tap weima/code-search
brew install cs
```

### NPM (Cross-platform)
```bash
npm install -g code-search-cli
```

### Cargo (Rust)
```bash
cargo install code-search
```

### From Binary
Download the pre-compiled binary for your platform from [Releases](https://github.com/weima/code-search/releases).

## Usage

```bash
# i18n text tracing (default mode)
cs "button text"

# Forward call tracing
cs "functionName" --trace

# Backward call tracing  
cs "functionName" --traceback

# Both directions
cs "functionName" --trace-all

# Custom depth (default: 3, max: 10)
cs "functionName" --trace --depth 5

# Case-sensitive search
cs "Text" --case-sensitive

# Include custom file extensions for code references
cs "button text" --include-extensions html.ui,vue.custom,erb.rails

# Help
cs --help
```

## Contributing

Contributions are welcome! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

For technical details and implementation plans, see:
- [Spec](.specify/specs/code-search/spec.md)
- [Plan](.specify/specs/code-search/plan.md)
- [Tasks](.specify/specs/code-search/tasks.md)

## License

Apache License 2.0 - See [LICENSE](LICENSE) file for details.
