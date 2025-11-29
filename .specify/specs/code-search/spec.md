# Feature Specification: Code Search CLI Tool

## Metadata

- **Feature Name**: Code Search Core Functionality
- **Feature ID**: CS-001
- **Priority**: P1 (Critical - MVP)
- **Specification Date**: 2025-11-28
- **Version**: 1.0

## Overview

Build a lightweight CLI tool (`cs`) that automates the tedious multi-step process of tracing UI text through internationalization (i18n) translation files to locate implementation code. The tool eliminates the manual workflow developers currently perform when debugging user-reported UI issues.

## Problem Statement

When users report issues like "I clicked the 'Add New' button and nothing happens," developers without heavy IDE support must perform a 6-step manual search process:

1. Search for literal text: `rg 'add new' -F`
2. Manually identify translation files in results
3. Open translation file to find the key (e.g., `add_new: 'add new'`)
4. Determine the full key path (e.g., `invoice.labels.add_new`)
5. Search for key usage: `rg 'invoice.labels.add_new' -F`
6. Locate implementation file (e.g., `components/invoices.ts:128`)

This manual process is time-consuming (2-5 minutes per search), error-prone (easy to miss nested keys), and interrupts developer flow.

## User Stories & Journeys

### US-1: Basic Text-to-Code Trace (P1 - Critical)
**As a** developer debugging a UI issue
**I want to** trace UI text directly to implementation code
**So that** I can quickly locate the relevant code without manual searching

**Priority Justification**: This is the core MVP functionality. Without this, the tool provides no value.

**Acceptance Criteria**:
```gherkin
Given a project with i18n translation files
When I run `cs "add new"`
Then I see a tree showing:
  - The search text
  - Translation file location and line number
  - Full translation key path
  - All code files that use that key
  - Line numbers for each usage
And the results are displayed within 500ms for typical projects
```

**Out of Scope**: Multiple language support, fuzzy matching, configuration files

### US-2: Clear Visual Output (P1 - Critical)
**As a** developer using the tool
**I want to** see results in a clear tree format
**So that** I can immediately understand the reference chain

**Priority Justification**: Critical for UX - unintuitive output means the tool won't be used.

**Acceptance Criteria**:
```gherkin
Given search results exist
When the tool displays output
Then I see a tree structure with:
  - Clear hierarchical indentation
  - File paths and line numbers for each node
  - Visual connectors (|, |->  etc.)
  - No truncated information
And the output is readable in 80-column terminals
```

**Out of Scope**: Colored output, interactive navigation, multiple output formats

### US-3: Multiple Match Handling (P1 - Critical)
**As a** developer
**I want to** see all locations where a translation key is used
**So that** I can understand the full impact of changes

**Priority Justification**: Required for real-world usage - translation keys are often reused.

**Acceptance Criteria**:
```gherkin
Given a translation key used in multiple files
When I search for the associated text
Then I see all usage locations in the tree
And each usage shows full file path and line number
And related usages are grouped visually
```

**Out of Scope**: Filtering by file type, sorting options, usage statistics

### US-4: Framework Pattern Detection (P2 - Important)
**As a** developer working with different frameworks
**I want** the tool to detect common i18n patterns automatically
**So that** it works across Ruby, JavaScript, and TypeScript projects

**Priority Justification**: Important for broad applicability but not critical for MVP.

**Acceptance Criteria**:
```gherkin
Given a project using Rails i18n (Ruby)
When I search for UI text
Then the tool finds I18n.t('key') and t('key') patterns

Given a project using react-i18next
When I search for UI text
Then the tool finds t('key'), i18n.t('key'), and useTranslation patterns

Given a project using vue-i18n
When I search for UI text
Then the tool finds $t('key') and {{ $t('key') }} patterns
```

**Out of Scope**: Custom pattern configuration, semgrep integration, complex interpolation patterns

### US-5: Error Handling and Guidance (P2 - Important)
**As a** developer
**I want** clear error messages when searches fail
**So that** I know how to adjust my search

**Priority Justification**: Important for UX but not blocking for initial release.

**Acceptance Criteria**:
```gherkin
Given no translation files found for search text
When the tool completes
Then I see a message "No translation files found containing 'search text'"
And I see suggested directories searched
And I see a tip about configuration

Given translation key found but no code usage
When the tool completes
Then I see the translation file location
And I see a warning "Translation exists but no code references found"
And I see a suggestion to check for dynamic key construction
```

**Out of Scope**: Did-you-mean suggestions, automatic retry with fuzzy matching

## Requirements

### Functional Requirements

**FR-001**: Text Search
- The tool must search all files for literal text matches
- Search must be case-insensitive by default
- Search must support glob patterns for file filtering

**FR-002**: Translation File Detection
- Must identify translation files based on:
  - File extension (.yml, .yaml, .json)
  - Directory patterns (locales/, i18n/, config/locales/)
  - Content structure (key-value pairs)
- Must support nested YAML structures (up to 10 levels deep)

**FR-003**: Key Path Extraction
- Must parse YAML and JSON files to extract translation key-value pairs
- Must construct full dot-notation key paths (e.g., `invoice.labels.add_new`)
- Must preserve key hierarchy and structure

**FR-004**: Pattern Matching
- Must find i18n function calls using regular expressions
- Must support minimum patterns:
  - Ruby: `I18n.t('key')`, `t('key')`
  - JavaScript/TypeScript: `i18n.t('key')`, `t('key')`, `$t('key')`
- Must capture surrounding context (line of code)

**FR-005**: Tree Output
- Must display results in hierarchical tree format
- Must show file paths, line numbers, and relevant code snippets
- Must fit within 80-column terminal width
- Must be readable on terminals with no color support

**FR-006**: Performance
- Must complete searches in < 500ms for projects with < 10k files
- Must limit memory usage to < 100MB during operation
- Must not hang or crash on large codebases

**FR-007**: Error Handling
- Must handle missing files gracefully (no crashes)
- Must handle malformed YAML (skip with warning)
- Must handle empty search results (clear message)
- Must validate search input (reject empty strings)

### Non-Functional Requirements

**NFR-001**: Performance
- Response time: < 500ms for typical searches
- Memory footprint: < 100MB
- Scalability: Handle projects up to 10k files

**NFR-002**: Reliability
- Zero crashes on valid inputs
- Graceful degradation on invalid inputs
- Clear error messages for all failure modes

**NFR-003**: Usability
- Zero configuration required for standard project layouts
- Intuitive command-line interface
- Self-documenting output

**NFR-004**: Maintainability
- Modular architecture (search, parse, format as separate modules)
- Test coverage ≥ 80%
- Clear API boundaries

## Key Entities

### SearchQuery
**Attributes**:
- text: string (the search text)
- case_sensitive: boolean (default: false)
- file_patterns: string[] (glob patterns, optional)

**Relationships**:
- Produces → SearchResult[]

### TranslationFile
**Attributes**:
- path: string (file path)
- format: enum (YAML, JSON)
- language: string (e.g., "en", "fr")

**Relationships**:
- Contains → TranslationEntry[]

### TranslationEntry
**Attributes**:
- file: string (file path)
- line: number (line number in file)
- key_path: string (e.g., "invoice.labels.add_new")
- value: string (translated text)
- language: string

**Relationships**:
- UsedBy → CodeReference[]

### CodeReference
**Attributes**:
- file: string (file path)
- line: number (line number)
- pattern: string (matched i18n pattern)
- context: string (surrounding code)

**Relationships**:
- References → TranslationEntry

### ReferenceTree
**Attributes**:
- root: TreeNode (search text)
- matches: TreeNode[] (translation and code nodes)

**Relationships**:
- Displays → OutputFormat

## Success Criteria

### User Experience Metrics
- [ ] Average search time < 500ms (measured on projects with 5k files)
- [ ] Output comprehensible without reading docs (tested with 5 new users)
- [ ] Zero configuration works for 80% of test projects (Ruby, React, Vue)

### System Performance
- [ ] Handles 10k file projects without slowdown
- [ ] Memory usage stays below 100MB
- [ ] No crashes on malformed input files

### Functional Completeness
- [ ] Successfully traces text → translation → code in test projects:
  - Rails app with YAML i18n
  - React app with JSON i18n
  - Vue app with JSON i18n
- [ ] Handles multiple matches correctly (shows all usages)
- [ ] Clear error messages for 100% of failure modes

### Code Quality
- [ ] Test coverage ≥ 80%
- [ ] All linter checks pass
- [ ] Documentation complete (README, API docs, examples)

## Clarifications Needed

### Question 1: JSON Support in MVP?
**Context**: Specification mentions YAML as primary format but many JS projects use JSON.

**Options**:
- A) MVP supports only YAML, JSON in Phase 2
- B) MVP supports both YAML and JSON
- C) MVP supports YAML, basic JSON (flat structure only)

**Recommendation**: Option A for focused MVP, then quick follow-up for JSON

### Question 2: Semgrep Integration Timeline?
**Context**: Semgrep can find complex patterns but adds dependency.

**Options**:
- A) MVP uses only regex patterns (lighter, faster)
- B) MVP includes semgrep for complex patterns (more complete)
- C) Semgrep is optional dependency (user installs if needed)

**Recommendation**: Option A for MVP, Option C for Phase 2

### Question 3: Configuration File in MVP?
**Context**: .csrc configuration mentioned but adds scope.

**Options**:
- A) MVP is zero-config only (hardcoded patterns)
- B) MVP supports optional .csrc (more flexible)
- C) MVP supports environment variables only (lighter than file)

**Recommendation**: Option A for MVP speed, Option B for Phase 2

## Dependencies

- **External**: ripgrep (must be installed on user system)
- **Optional**: semgrep (Phase 2+)
- **Internal**: YAML parser library (language-dependent)

## Risks & Mitigations

| Risk | Impact | Probability | Mitigation |
|------|--------|-------------|------------|
| ripgrep not installed | High | Medium | Check at startup, provide clear install instructions |
| Complex key interpolation | Medium | High | Document limitation, add dynamic key detection in Phase 2 |
| Performance on huge repos | Medium | Low | Implement search depth limits and .csignore support |
| YAML parsing errors | Low | Medium | Graceful error handling, skip malformed files with warning |

### US-6: Call Graph Tracing (P1 - Critical)
**As a** developer exploring unfamiliar code
**I want to** trace function call hierarchies forward and backward
**So that** I can understand code flow without manually searching for each function

**Priority Justification**: Core developer workflow - understanding what a function does and who calls it is fundamental to code comprehension.

**Acceptance Criteria**:
```gherkin
Given a codebase with function definitions and calls
When I run `cs 'bar' --trace`
Then I see a forward call tree showing:
  - Functions that 'bar' calls (callees)
  - Functions those callees call (recursively)
  - Tree depth limited to 3 levels by default
And the output displays as:
  bar
  |-> zoo1
  |-> zoo2
  |-> zoo3

Given a codebase with function definitions and calls
When I run `cs 'bar' --traceback`
Then I see a reverse call tree showing:
  - Functions that call 'bar' (callers)
  - Functions that call those callers (recursively)
  - Tree depth limited to 3 levels by default
And the output displays as:
  blah1 -> foo1 -> bar
  blah2 -> foo2 -> bar

Given a deep call hierarchy
When I run `cs 'bar' --trace --depth 5`
Then the tool traces up to 5 levels deep
And stops at the specified depth to prevent exponential explosion

Given a function with no callees
When I run `cs 'bar' --trace`
Then I see the function name with a message "No outgoing calls found"

Given a function with no callers
When I run `cs 'bar' --traceback`
Then I see the function name with a message "No incoming calls found"
```

**Out of Scope**: Cross-language tracing, dynamic dispatch resolution, macro expansion

### US-7: Combined Trace Output (P2 - Important)
**As a** developer
**I want to** see both callers and callees in a single view
**So that** I can understand the full context of a function

**Priority Justification**: Convenience feature that combines US-6 capabilities.

**Acceptance Criteria**:
```gherkin
Given a function with both callers and callees
When I run `cs 'bar' --trace-all`
Then I see both directions in a combined view:
  - Callers section (who calls bar)
  - Callees section (what bar calls)
And each section respects the depth limit
```

**Out of Scope**: Interactive navigation between callers/callees

## Requirements

### Functional Requirements

**FR-001**: Text Search
- The tool must search all files for literal text matches
- Search must be case-insensitive by default
- Search must support glob patterns for file filtering

**FR-002**: Translation File Detection
- Must identify YAML files as translation files based on:
  - File extension (.yml, .yaml)
  - Directory patterns (locales/, i18n/, config/locales/)
  - Content structure (key-value pairs)
- Must support nested YAML structures (up to 10 levels deep)

**FR-003**: Key Path Extraction
- Must parse YAML files to extract translation key-value pairs
- Must construct full dot-notation key paths (e.g., `invoice.labels.add_new`)
- Must preserve key hierarchy and structure

**FR-004**: Pattern Matching
- Must find i18n function calls using regular expressions
- Must support minimum patterns:
  - Ruby: `I18n.t('key')`, `t('key')`
  - JavaScript/TypeScript: `i18n.t('key')`, `t('key')`, `$t('key')`
- Must capture surrounding context (line of code)

**FR-005**: Tree Output
- Must display results in hierarchical tree format
- Must show file paths, line numbers, and relevant code snippets
- Must fit within 80-column terminal width
- Must be readable on terminals with no color support

**FR-006**: Performance
- Must complete searches in < 500ms for projects with < 10k files
- Must limit memory usage to < 100MB during operation
- Must not hang or crash on large codebases

**FR-007**: Error Handling
- Must handle missing files gracefully (no crashes)
- Must handle malformed YAML (skip with warning)
- Must handle empty search results (clear message)
- Must validate search input (reject empty strings)

**FR-008**: Forward Call Tracing
- Must identify function/method definitions matching the search term
- Must extract function calls within matched function bodies
- Must recursively trace callees up to configurable depth (default: 3)
- Must display call hierarchy as a tree with visual connectors
- Must handle circular call references without infinite loops

**FR-009**: Backward Call Tracing (Traceback)
- Must find all call sites where the target function is invoked
- Must identify the containing function for each call site
- Must recursively trace callers up to configurable depth (default: 3)
- Must display caller chains with clear directional notation
- Must handle multiple independent call paths

**FR-010**: Trace Depth Control
- Must support `--depth N` flag to override default depth limit
- Must enforce maximum depth of 10 to prevent resource exhaustion
- Must display depth-limited indicator when limit is reached

**FR-011**: Cross-Case Function Search
- Must support case-insensitive function name matching across naming conventions
- Must generate and search all case variants for a given function name:
  - snake_case ↔ camelCase ↔ PascalCase conversion
  - Example: `createUser` finds `create_user`, `createUser`, `CreateUser`
- Must preserve original function names in display results (no modification)
- Must work for both forward (`--trace`) and backward (`--traceback`) tracing
- Must handle multi-word conversions: `process_user_data` ↔ `processUserData` ↔ `ProcessUserData`
- Must deduplicate results while maintaining all discovered matches
- Critical for polyglot projects (Rails backend + React frontend)

## Out of Scope (Deferred to Future Phases)

- Interactive navigation mode
- Configuration file (.csrc)
- Colored output
- Reverse search (code → text)
- Fuzzy text matching
- Editor integrations (VSCode, Vim)
- Caching layer
- Multiple language display
- Custom pattern configuration
- Cross-language call tracing
- Dynamic dispatch / virtual method resolution
- Macro expansion in call analysis

---

**This specification is technology-agnostic and focuses on WHAT the tool does and WHY, not HOW it's implemented.**
