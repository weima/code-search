# Project Strategy & Roadmap

This document serves as a living guide for the strategic direction of `code-search`. It outlines our vision, technical philosophy, and the roadmap for making this tool a standard utility for developers.

## 1. Vision & Value Proposition

**The Problem: The "Spaghetti Code" Gap**
Modern development, especially in frontend and i18n-heavy applications, involves complex indirect references. A simple UI text like "Save Changes" often leads to a labyrinth:
`UI Text` → `Translation Key` → `Namespace Variable` → `Component Usage` → `Implementation`

Standard tools fail here:
*   **`grep`/`rg`**: Fast but lacks semantic understanding. It finds the text but not the connection.
*   **IDEs**: Understand code structure but often struggle with dynamic keys (e.g., `t("errors." + code)`) and are too heavy for quick CLI workflows.

**Our Solution**
`code-search` aims to bridge this gap. We are building a tool that combines the **speed of a CLI utility** with the **semantic understanding of an IDE**.

**Goal**: To become as essential and ubiquitous in a developer's toolkit as `ls`, `grep`, or `git`.

## 2. Technical Philosophy: "Hardcore" Performance

To achieve our goal, we adhere to a strict philosophy of performance and stability. We are not building a prototype; we are building infrastructure.

### Core Tenets
1.  **Zero Latency**: Results should appear instantly. We aim for performance parity with `ripgrep`.
2.  **Correctness First**: False positives are annoying; false negatives are unacceptable.
3.  **Unix Philosophy**: Do one thing well. Support piping. Output structured data (JSON) for composability.

### Performance Targets
*   **Parallelism**: Utilize all available CPU cores efficiently.
*   **Zero-Copy**: Minimize memory allocation. Use zero-copy parsing where possible.
*   **Lock-Free**: Avoid mutex contention in hot paths (e.g., search loops).

## 3. Cross-Platform Commitment

Developers work on macOS, Linux, and Windows. `code-search` must be a first-class citizen on all three.

*   **Windows Support**: We are committed to solving Windows-specific challenges (path separators, line endings, permissions).
*   **Testing**: We use cross-compilation strategies (e.g., `cross`) to ensure our test suite passes in Windows environments, even when developing on Unix-like systems.

## 4. Future Architecture & Inspiration

We draw inspiration from best-in-class CLI tools to guide our evolution.

### Interactive Mode (Inspired by `yazi`)
While the CLI is our foundation, we envision an interactive TUI (Text User Interface) mode (`cs -i`) for exploration:
*   Navigate results with keyboard shortcuts.
*   Preview code context in split panes.
*   Jump directly to the editor.
*   *Tech Stack*: `ratatui` (Rust TUI library).

### Composability (Inspired by `jq`)
We will support rich, structured output formats (`--json`) to allow users to pipe results into tools like `jq` for advanced filtering and processing.

## 5. Roadmap

### Phase 1: Foundation & Stability (Current)
*   [ ] **Windows Compatibility**: Resolve unit test failures on Windows (pathing/newlines).
*   [ ] **CI/CD**: Robust pipelines for all major OSs.
*   [ ] **Core Features**: Solidify i18n tracing and call graph logic.

### Phase 2: Performance Optimization
*   [ ] **Concurrency Refactor**: Replace `Mutex`-based result collection with lock-free channels to maximize throughput.
*   [ ] **Benchmarking**: Establish a baseline against `ripgrep` to measure and minimize overhead.
*   [ ] **Profiling**: Identify and eliminate CPU hotspots.

### Bottom-Up Trace Optimization

We assume non-tangled translation trees: multiple matches for the same value appear in increasing line order, and their ancestors are shared or appear after earlier matches. The bottom-up tracers (YAML/JSON) now:

* Walk matches in ascending line order.
* Track ancestor prefixes by line number; later traces stop as soon as they hit a previously seen ancestor instead of climbing to the file start.
* Respect a monotonic cutoff so we avoid re-walking earlier regions unless we reconnect to a known common parent.

This preserves correctness (common parents still discovered) while reducing redundant upward scans on large files.

### Phase 3: The "Pro" Experience
*   [ ] **Interactive TUI**: Implement the `cs -i` interactive mode.
*   [ ] **Structured Output**: Full JSON output support for all commands.
*   [ ] **Plugin System**: Allow community extensions for custom file types or frameworks.

---

*This document is open for discussion. We welcome contributions that align with these goals.*
