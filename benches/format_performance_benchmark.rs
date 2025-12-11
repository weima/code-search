use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use cs::output::TreeFormatter;
use cs::{CodeReference, SearchQuery, SearchResult, TranslationEntry};
use std::path::PathBuf;
use std::time::Duration;

/// Get the path to test fixtures
fn fixture_path(subdir: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join(subdir)
}

/// Generate synthetic search results for benchmarking
fn generate_search_results(num_translation_entries: usize, num_code_refs: usize) -> SearchResult {
    let mut translation_entries = Vec::new();
    let mut code_references = Vec::new();

    // Generate translation entries
    for i in 0..num_translation_entries {
        translation_entries.push(TranslationEntry {
            file: PathBuf::from(format!("locales/en_{}.yml", i % 10)),
            line: i + 1,
            key: format!("app.section_{}.key_{}", i % 5, i),
            value: format!("Translation value for key {}", i),
        });
    }

    // Generate code references with context
    for i in 0..num_code_refs {
        let context_before = (0..2)
            .map(|j| format!("  // Context line {} before", j))
            .collect();
        let context_after = (0..2)
            .map(|j| format!("  // Context line {} after", j))
            .collect();

        code_references.push(CodeReference {
            file: PathBuf::from(format!("src/components/Component_{}.tsx", i % 20)),
            line: (i * 3) + 10,
            pattern: format!("t\\('app\\.section_{}\\.", i % 5),
            context: format!("  const text = t('app.section_{}.key_{}');", i % 5, i),
            key_path: format!("app.section_{}.key_{}", i % 5, i),
            context_before,
            context_after,
        });
    }

    SearchResult {
        query: "benchmark_test".to_string(),
        translation_entries,
        code_references,
    }
}

/// Benchmark simple vs tree format performance
fn bench_format_comparison(c: &mut Criterion) {
    let mut group = c.benchmark_group("format_comparison");
    group.sample_size(100);
    group.measurement_time(Duration::from_secs(10));

    let sizes = [
        ("small", 10, 20),
        ("medium", 50, 100),
        ("large", 200, 500),
        ("xlarge", 1000, 2000),
    ];

    for (size_name, num_translations, num_code_refs) in sizes.iter() {
        let result = generate_search_results(*num_translations, *num_code_refs);

        // Benchmark tree format (default)
        group.bench_with_input(
            BenchmarkId::new("tree_format", size_name),
            &result,
            |b, result| {
                let formatter = TreeFormatter::new().with_simple_format(false);
                b.iter(|| black_box(formatter.format_result(black_box(result))));
            },
        );

        // Benchmark simple format
        group.bench_with_input(
            BenchmarkId::new("simple_format", size_name),
            &result,
            |b, result| {
                let formatter = TreeFormatter::new().with_simple_format(true);
                b.iter(|| black_box(formatter.format_result(black_box(result))));
            },
        );
    }

    group.finish();
}

/// Benchmark context line processing overhead
fn bench_context_processing(c: &mut Criterion) {
    let mut group = c.benchmark_group("context_processing");
    group.sample_size(50);
    group.measurement_time(Duration::from_secs(15));

    let context_sizes = [0, 1, 2, 5, 10];
    let num_matches = 100;

    for context_lines in context_sizes.iter() {
        // Generate code references with varying context sizes
        let mut code_references = Vec::new();
        for i in 0..num_matches {
            let context_before = (0..*context_lines)
                .map(|j| format!("  // Context line {} before match {}", j, i))
                .collect();
            let context_after = (0..*context_lines)
                .map(|j| format!("  // Context line {} after match {}", j, i))
                .collect();

            code_references.push(CodeReference {
                file: PathBuf::from(format!("src/file_{}.ts", i % 10)),
                line: (i * 5) + 10,
                pattern: "processData\\(".to_string(),
                context: format!("  const result = processData({});", i),
                key_path: format!("data.key_{}", i),
                context_before,
                context_after,
            });
        }

        let result = SearchResult {
            query: "benchmark_context".to_string(),
            translation_entries: Vec::new(),
            code_references,
        };

        // Benchmark tree format with context
        group.bench_with_input(
            BenchmarkId::new("tree_with_context", format!("{}_lines", context_lines)),
            &result,
            |b, result| {
                let formatter = TreeFormatter::new().with_simple_format(false);
                b.iter(|| black_box(formatter.format_result(black_box(result))));
            },
        );

        // Benchmark simple format (no context processing)
        group.bench_with_input(
            BenchmarkId::new("simple_no_context", format!("{}_lines", context_lines)),
            &result,
            |b, result| {
                let formatter = TreeFormatter::new().with_simple_format(true);
                b.iter(|| black_box(formatter.format_result(black_box(result))));
            },
        );
    }

    group.finish();
}

/// Benchmark context overlap detection and merging
fn bench_context_overlap_merging(c: &mut Criterion) {
    let mut group = c.benchmark_group("context_overlap_merging");
    group.sample_size(30);
    group.measurement_time(Duration::from_secs(20));

    // Test different overlap scenarios
    let overlap_scenarios = [
        ("no_overlap", 10),   // Matches 10 lines apart
        ("some_overlap", 5),  // Matches 5 lines apart (overlapping context)
        ("heavy_overlap", 2), // Matches 2 lines apart (heavy overlap)
        ("adjacent", 1),      // Adjacent matches
    ];

    for (scenario_name, line_spacing) in overlap_scenarios.iter() {
        let mut code_references = Vec::new();
        let num_matches = 50;
        let file_path = PathBuf::from("src/test_file.ts");

        // Generate matches with specified spacing
        for i in 0..num_matches {
            let line_num = (i * line_spacing) + 10;
            let context_before = vec![
                format!("  // Context before line {}", line_num - 2),
                format!("  // Context before line {}", line_num - 1),
            ];
            let context_after = vec![
                format!("  // Context after line {}", line_num + 1),
                format!("  // Context after line {}", line_num + 2),
            ];

            code_references.push(CodeReference {
                file: file_path.clone(),
                line: line_num,
                pattern: "getValue\\(\\)".to_string(),
                context: format!("  const match_{} = getValue();", i),
                key_path: format!("key_{}", i),
                context_before,
                context_after,
            });
        }

        let result = SearchResult {
            query: "benchmark_overlap".to_string(),
            translation_entries: Vec::new(),
            code_references,
        };

        group.bench_with_input(
            BenchmarkId::new("overlap_processing", scenario_name),
            &result,
            |b, result| {
                let formatter = TreeFormatter::new().with_simple_format(false);
                b.iter(|| black_box(formatter.format_result(black_box(result))));
            },
        );
    }

    group.finish();
}

/// Benchmark memory usage with large files and many matches
fn bench_memory_usage(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_usage");
    group.sample_size(20);
    group.measurement_time(Duration::from_secs(30));

    let memory_scenarios = [
        ("small_files_many_matches", 100, 10), // 100 matches per file, 10 files
        ("large_files_few_matches", 10, 100),  // 10 matches per file, 100 files
        ("balanced", 50, 50),                  // 50 matches per file, 50 files
    ];

    for (scenario_name, matches_per_file, num_files) in memory_scenarios.iter() {
        let mut code_references = Vec::new();

        for file_idx in 0..*num_files {
            let file_path = PathBuf::from(format!("src/large_file_{}.ts", file_idx));

            for match_idx in 0..*matches_per_file {
                let line_num = (match_idx * 10) + 1;

                // Generate larger context to simulate real files
                let context_before = (0..2)
                    .map(|i| {
                        format!(
                            "  // Large context line {} in file {} before match {}",
                            line_num - 2 + i,
                            file_idx,
                            match_idx
                        )
                    })
                    .collect();
                let context_after = (0..2)
                    .map(|i| {
                        format!(
                            "  // Large context line {} in file {} after match {}",
                            line_num + 1 + i,
                            file_idx,
                            match_idx
                        )
                    })
                    .collect();

                code_references.push(CodeReference {
                    file: file_path.clone(),
                    line: line_num,
                    pattern: "processLargeDataStructure\\(\\)".to_string(),
                    context: format!(
                        "  const largeVariableName_{}_{}_ = processLargeDataStructure();",
                        file_idx, match_idx
                    ),
                    key_path: format!(
                        "large.nested.key.structure.file_{}.match_{}",
                        file_idx, match_idx
                    ),
                    context_before,
                    context_after,
                });
            }
        }

        let result = SearchResult {
            query: "benchmark_memory".to_string(),
            translation_entries: Vec::new(),
            code_references,
        };

        // Benchmark tree format
        group.bench_with_input(
            BenchmarkId::new("tree_format_memory", scenario_name),
            &result,
            |b, result| {
                let formatter = TreeFormatter::new().with_simple_format(false);
                b.iter(|| black_box(formatter.format_result(black_box(result))));
            },
        );

        // Benchmark simple format
        group.bench_with_input(
            BenchmarkId::new("simple_format_memory", scenario_name),
            &result,
            |b, result| {
                let formatter = TreeFormatter::new().with_simple_format(true);
                b.iter(|| black_box(formatter.format_result(black_box(result))));
            },
        );
    }

    group.finish();
}

/// Benchmark end-to-end search with different output formats
fn bench_end_to_end_with_formats(c: &mut Criterion) {
    let mut group = c.benchmark_group("end_to_end_formats");
    group.sample_size(10);
    group.measurement_time(Duration::from_secs(30));

    let base_dir = fixture_path("rails-app");
    if !base_dir.exists() {
        eprintln!("⚠️  Skipping end-to-end benchmark: fixture not found");
        return;
    }

    // Benchmark with cache cleared before each run to get accurate timing
    group.bench_function("end_to_end_no_cache", |b| {
        b.iter_batched(
            || {
                // Clear cache before each iteration to avoid cache hits
                if let Ok(cache) = cs::SearchResultCache::new() {
                    let _ = cache.clear();
                }
                base_dir.clone()
            },
            |dir| {
                let query = SearchQuery::new(black_box("add new".to_string())).with_base_dir(dir);
                cs::run_search(query).unwrap()
            },
            criterion::BatchSize::PerIteration,
        );
    });

    // Benchmark with cache enabled (potential cache hits after first run)
    group.bench_function("end_to_end_with_cache", |b| {
        b.iter(|| {
            let query =
                SearchQuery::new(black_box("add new".to_string())).with_base_dir(base_dir.clone());
            cs::run_search(query).unwrap()
        });
    });

    group.finish();
}

/// Benchmark special character handling performance
fn bench_special_character_handling(c: &mut Criterion) {
    let mut group = c.benchmark_group("special_character_handling");
    group.sample_size(100);
    group.measurement_time(Duration::from_secs(10));

    // Generate results with various special characters
    let special_chars_scenarios = [
        ("ascii_only", "Simple ASCII text with no special characters"),
        ("unicode", "Unicode text with émojis 🚀 and spëcial chars ñ"),
        (
            "ansi_codes",
            "\x1b[31mRed text\x1b[0m with \x1b[1mbold\x1b[0m formatting",
        ),
        (
            "shell_meta",
            "Text with $VAR and `command` and | pipes & ampersands",
        ),
        (
            "mixed",
            "\x1b[32mGreen\x1b[0m unicode 🎯 with $VARS and | pipes",
        ),
    ];

    for (scenario_name, content_template) in special_chars_scenarios.iter() {
        let mut code_references = Vec::new();

        // Generate multiple entries with special characters
        for i in 0..50 {
            code_references.push(CodeReference {
                file: PathBuf::from(format!("src/special_{}.ts", i)),
                line: i + 1,
                pattern: "special.*pattern".to_string(),
                context: format!("{} - line {}", content_template, i),
                key_path: format!("key_{}", i),
                context_before: vec![format!("// Before: {}", content_template)],
                context_after: vec![format!("// After: {}", content_template)],
            });
        }

        let result = SearchResult {
            query: "benchmark_special_chars".to_string(),
            translation_entries: Vec::new(),
            code_references,
        };

        // Benchmark simple format (requires escaping)
        group.bench_with_input(
            BenchmarkId::new("simple_format_escaping", scenario_name),
            &result,
            |b, result| {
                let formatter = TreeFormatter::new().with_simple_format(true);
                b.iter(|| black_box(formatter.format_result(black_box(result))));
            },
        );

        // Benchmark tree format (no escaping needed)
        group.bench_with_input(
            BenchmarkId::new("tree_format_no_escaping", scenario_name),
            &result,
            |b, result| {
                let formatter = TreeFormatter::new().with_simple_format(false);
                b.iter(|| black_box(formatter.format_result(black_box(result))));
            },
        );
    }

    group.finish();
}

/// Print benchmark summary and configuration
fn print_benchmark_summary(_c: &mut Criterion) {
    println!("\n📊 Format Performance Benchmark Configuration:");
    println!("{:<30} {:<20}", "Benchmark", "Purpose");
    println!("{}", "=".repeat(52));
    println!(
        "{:<30} {:<20}",
        "format_comparison", "Simple vs Tree format speed"
    );
    println!(
        "{:<30} {:<20}",
        "context_processing", "Context line overhead"
    );
    println!(
        "{:<30} {:<20}",
        "context_overlap_merging", "Overlap detection cost"
    );
    println!("{:<30} {:<20}", "memory_usage", "Large result set handling");
    println!(
        "{:<30} {:<20}",
        "end_to_end_formats", "Complete workflow timing"
    );
    println!(
        "{:<30} {:<20}",
        "special_character_handling", "Escaping performance"
    );
    println!();
}

criterion_group!(
    benches,
    print_benchmark_summary,
    bench_format_comparison,
    bench_context_processing,
    bench_context_overlap_merging,
    bench_memory_usage,
    bench_end_to_end_with_formats,
    bench_special_character_handling,
);

criterion_main!(benches);
