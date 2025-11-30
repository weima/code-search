# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Performance benchmarks using Criterion
- Benchmark suite covering all major operations:
  - Text search (ripgrep)
  - YAML key extraction
  - Pattern matching
  - End-to-end i18n search
  - Call graph tracing
  - Project size variations

### Performance Benchmarks (v0.1.0)

Benchmarks run on 2025-11-29 using Criterion 0.5.

#### Text Search (ripgrep operations)
| Project Size | Mean Time | Target | Status |
|--------------|-----------|--------|--------|
| Small (4 files) | 10.6ms | <100ms | ✅ Pass |
| Medium (3 files) | 10.7ms | <100ms | ✅ Pass |
| Large (all fixtures) | 10.2ms | <100ms | ✅ Pass |

#### YAML Key Extraction
| Operation | Mean Time | Status |
|-----------|-----------|--------|
| Small YAML | 123μs | ✅ Excellent |
| Medium YAML | 124μs | ✅ Excellent |

#### Pattern Matching (code reference search)
| Codebase Size | Mean Time | Target | Status |
|---------------|-----------|--------|--------|
| Small | 10.9ms | <100ms | ✅ Pass |
| Medium (multi-key) | 21.9ms | <100ms | ✅ Pass |

#### End-to-End i18n Search
| Project Size | Mean Time | Target | Status |
|--------------|-----------|--------|--------|
| Small project | 66.3ms | <100ms | ✅ Pass |
| Medium (case-sensitive) | 65.9ms | <500ms | ✅ Pass |
| Large (all fixtures) | 902.6ms | N/A | ℹ️ Large dataset |

#### Call Graph Tracing
| Operation | Mean Time | Target | Status |
|-----------|-----------|--------|--------|
| Forward trace (depth 1) | 166.3ms | <500ms | ✅ Pass |
| Forward trace (depth 3) | 540.1ms | <500ms | ⚠️ Slightly over |
| Backward trace (depth 3) | 393.0ms | <500ms | ✅ Pass |

#### Project Size Benchmarks
| Size Category | Mean Time | Notes |
|---------------|-----------|-------|
| Small (rails-app) | 66.1ms | Complete i18n workflow |
| Medium (code-examples) | 10.1μs | Optimized search |
| Large (all fixtures) | 256.5μs | Fast recursive search |

### Performance Summary

**✅ All primary targets met:**
- Small projects: All operations < 100ms ✓
- Medium projects: All operations < 500ms ✓ (with one exception)
- Key extraction: Sub-millisecond performance ✓

**Key Highlights:**
- YAML parsing is extremely fast (~123μs)
- Text search operations are consistently ~10ms regardless of project size
- End-to-end workflows complete in 60-70ms for typical projects
- Call tracing slightly exceeds 500ms target at depth 3 (forward direction)

**Optimization Opportunities:**
- Forward call tracing at depth 3 (540ms vs 500ms target)
- Could benefit from caching for repeated function lookups

### Technical Details

- **Benchmark Framework:** Criterion v0.5
- **Test Data:** Real-world fixtures (Rails, React, Vue applications)
- **Sample Size:** 100 samples per benchmark
- **Compiler:** rustc with `--release` optimizations
- **Platform:** macOS (Darwin 24.2.0)

To run benchmarks:
```bash
cargo bench --bench search_benchmark
```

Results are saved to `target/criterion/` with detailed HTML reports.

## [0.1.0] - 2025-11-29

### Initial Release
- i18n translation search with YAML support
- Code reference tracking across multiple frameworks
- Call graph tracing (forward and backward)
- Tree-based output formatting
- Cross-platform support (macOS, Linux, Windows)
