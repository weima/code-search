# Code Search Project Constitution

## Metadata

- **Project Name**: Code Search (cs)
- **Ratification Date**: 2025-11-28
- **Last Amended Date**: 2025-11-28
- **Constitution Version**: 1.0

## Article I: Performance First

### Principle
Performance is a primary feature, not an optimization afterthought.

### Rules
1. All operations must complete within defined performance budgets:
   - Small projects (< 1k files): < 100ms
   - Medium projects (< 10k files): < 500ms
   - Large projects (< 100k files): < 2s
2. Memory footprint must not exceed 100MB for typical operations
3. Performance regressions are treated as critical bugs
4. All new features must include performance benchmarks

### Rationale
This tool exists to be lightweight and fast - an alternative to resource-heavy IDEs. If it's slow, it has failed its core mission.

## Article II: Simplicity and Focus

### Principle
Do one thing exceptionally well: trace UI text to implementation code.

### Rules
1. Reject feature creep - every feature must directly support the core workflow
2. CLI interface only - no GUI, no web interface in core tool
3. Minimal dependencies - prefer standard library and proven tools (ripgrep, semgrep)
4. Zero configuration should work for 80% of use cases

### Rationale
Complexity is the enemy of reliability and performance. A focused tool that solves one problem well is more valuable than a Swiss Army knife.

## Article III: Developer Experience

### Principle
The tool should feel intuitive to developers already familiar with command-line workflows.

### Rules
1. Default output must be immediately understandable without reading documentation
2. Error messages must be actionable and suggest fixes
3. Common operations should require minimal keystrokes
4. Support standard conventions (`--help`, `--version`, `--verbose`)
5. Respect existing conventions (`.gitignore`, NO_COLOR env variable)

### Rationale
Developer time is valuable. A tool that requires constant documentation lookups or produces cryptic errors will not be adopted.

## Article IV: Testing and Quality

### Principle
Code quality is non-negotiable; testing is mandatory.

### Rules
1. Minimum 80% test coverage for all code
2. All features must include:
   - Unit tests for core logic
   - Integration tests for end-to-end workflows
   - Performance benchmarks
3. CI/CD must pass before any merge
4. No warnings allowed in builds
5. Code must pass linter checks (language-appropriate)

### Rationale
This tool will be used in critical debugging workflows. Bugs in the tool waste developer time and erode trust.

## Article V: Multi-Framework Support

### Principle
The tool must work across different i18n frameworks and languages.

### Rules
1. Core architecture must be extensible to support new patterns
2. Pattern detection must be configurable (regex, semgrep rules)
3. Default patterns must cover:
   - Ruby (Rails i18n)
   - JavaScript/TypeScript (react-i18next, vue-i18n)
   - YAML and JSON translation files
4. Adding new framework support should not require core rewrites

### Rationale
Real-world development spans many frameworks. A tool limited to one ecosystem has limited value.

## Article VI: Clear Architecture

### Principle
The codebase should be organized for maintainability and extensibility.

### Rules
1. Separation of concerns:
   - Search logic separate from parsing
   - Parsing separate from output formatting
   - Pattern matching separate from file I/O
2. Each module must have a single, clear responsibility
3. Public APIs must be documented with examples
4. Core abstractions (Searcher, Parser, Formatter) must be interface-driven

### Rationale
Code that is easy to understand is easy to extend and fix. Future contributors should be able to add new features without understanding the entire codebase.

## Article VII: Security and Safety

### Principle
Never execute arbitrary code; prevent malicious inputs.

### Rules
1. No eval() or equivalent in any language
2. All file paths must be validated against directory traversal
3. Search depth must be limited to prevent infinite loops
4. External tool invocations (ripgrep, semgrep) must sanitize inputs
5. Respect .gitignore and .csignore to avoid scanning sensitive files

### Rationale
This tool will run in production codebases. A security vulnerability could expose sensitive information or cause system damage.

## Article VIII: Documentation Standards

### Principle
Documentation is part of the deliverable, not an afterthought.

### Rules
1. All features must be documented in README.md before merge
2. API documentation must include examples
3. Complex algorithms must include comments explaining "why," not "what"
4. Configuration options must be documented with examples
5. Breaking changes require migration guides

### Rationale
Undocumented features might as well not exist. Good documentation multiplies the value of good code.

## Article IX: Open Source Best Practices

### Principle
This project follows open source community standards.

### Rules
1. Apache License 2.0 for all code
2. Contributor guidelines must be clear and welcoming
3. Issues must be triaged within 48 hours
4. Pull requests must be reviewed within 72 hours
5. Semantic versioning for all releases
6. Changelog maintained for every release

### Rationale
Open source thrives on community participation. Clear processes and quick feedback encourage contributions.

## Governance

### Amendment Process
1. Proposed amendments must be submitted as pull requests
2. Amendments require approval from at least 2 core maintainers
3. Breaking changes to principles require project-wide discussion
4. All amendments must include rationale and impact analysis

### Versioning Policy
- Major version bump: Breaking changes to principles
- Minor version bump: New principles added
- Patch version bump: Clarifications or corrections

### Violation Handling
1. All principle violations must be documented and justified
2. Temporary violations require a remediation plan with timeline
3. Permanent exceptions require constitutional amendment

---

**This constitution serves as the immutable foundation for all Code Search development decisions.**
