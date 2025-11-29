# Code Search (cs)

**Intelligent code search tool for tracing UI text through i18n files to implementation**

## Problem Statement

Modern IDEs like JetBrains suite provide powerful code navigation features through context menus - find definition, find references, etc. However, these IDEs are resource-intensive and not always practical for quick searches or lightweight environments.

When debugging user-reported issues like "I clicked the 'Add New' button and nothing happens," developers without IDE support must perform a tedious multi-step manual search:

1. Search for the text: `rg 'add new' -F`
2. Manually scan results to find translation files (e.g., `en.yml`)
3. Open the translation file and locate the key: `add_new: 'add new'`
4. Examine the YAML structure to find the full key path: `invoice.labels.add_new`
5. Search for the key usage: `rg 'invoice.labels.add_new' -F`
6. Finally locate the implementation in `components/invoices.ts`

This manual process is time-consuming, error-prone, and interrupts the development flow.

## Solution

`code-search` (abbreviated as `cs`) is a lightweight CLI tool that automates this entire workflow, providing intelligent code tracing from UI text through translation files to actual implementation.

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

- **Smart Text Tracing**: Automatically follows references from UI text to implementation
- **Call Graph Tracing**: Trace function calls forward (`--trace`) or backward (`--traceback`)
- **i18n Aware**: Understands YAML/JSON translation file structures
- **Pattern Recognition**: Identifies common i18n patterns (I18n.t, t(), $t, etc.)
- **Tree Visualization**: Clear visual representation of the reference chain
- **Depth Control**: Configurable trace depth to prevent explosion in large codebases
- **Cycle Detection**: Handles recursive/circular calls without hanging
- **Lightweight**: Built on ripgrep for fast performance
- **No IDE Required**: Works in any terminal environment

## Use Cases

- **Bug Triage**: Quickly locate implementation code from user-reported UI issues
- **Code Exploration**: Understand what a function does by tracing its calls
- **Impact Analysis**: Find all callers of a function before refactoring
- **Code Review**: Trace UI text to verify correct implementation
- **Refactoring**: Find all usages of translation keys
- **Onboarding**: Help new developers understand code organization and call flow
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
- **ripgrep**: Fast text searching
- **Regex patterns**: Function definition and call detection
- **YAML/JSON parsers**: Translation file processing
- **Tree builders**: Visual output formatting

## Installation

### Prerequisites
- [ripgrep](https://github.com/BurntSushi/ripgrep#installation) must be installed

### From Source (Rust)
```bash
cargo install code-search
```

### From GitHub Releases
Download the pre-compiled binary for your platform from [Releases](https://github.com/user/code-search/releases).

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
