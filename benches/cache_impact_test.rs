use criterion::{black_box, criterion_group, criterion_main, Criterion};
use cs::output::TreeFormatter;
use cs::{CodeReference, SearchResult, SearchResultCache, TranslationEntry};
use std::path::PathBuf;
use std::time::Duration;

/// Test to verify cache impact on benchmark results
fn cache_impact_test(c: &mut Criterion) {
    let mut group = c.benchmark_group("cache_impact");
    group.sample_size(10);
    group.measurement_time(Duration::from_secs(5));

    // Create a test result
    let result = SearchResult {
        query: "cache_test".to_string(),
        translation_entries: vec![TranslationEntry {
            file: PathBuf::from("test.yml"),
            line: 1,
            key: "test.key".to_string(),
            value: "test value".to_string(),
        }],
        code_references: vec![CodeReference {
            file: PathBuf::from("test.ts"),
            line: 10,
            pattern: "test".to_string(),
            context: "const test = 'value';".to_string(),
            key_path: "test.key".to_string(),
            context_before: vec!["// before".to_string()],
            context_after: vec!["// after".to_string()],
        }],
    };

    // Test with cache cleared before each run
    group.bench_function("tree_format_no_cache", |b| {
        b.iter_batched(
            || {
                // Clear cache before each iteration
                if let Ok(cache) = SearchResultCache::new() {
                    let _ = cache.clear();
                }
                TreeFormatter::new().with_simple_format(false)
            },
            |formatter| black_box(formatter.format_result(black_box(&result))),
            criterion::BatchSize::PerIteration,
        );
    });

    group.bench_function("simple_format_no_cache", |b| {
        b.iter_batched(
            || {
                // Clear cache before each iteration
                if let Ok(cache) = SearchResultCache::new() {
                    let _ = cache.clear();
                }
                TreeFormatter::new().with_simple_format(true)
            },
            |formatter| black_box(formatter.format_result(black_box(&result))),
            criterion::BatchSize::PerIteration,
        );
    });

    // Test without clearing cache (potential cache hits)
    group.bench_function("tree_format_with_cache", |b| {
        let formatter = TreeFormatter::new().with_simple_format(false);
        b.iter(|| black_box(formatter.format_result(black_box(&result))));
    });

    group.bench_function("simple_format_with_cache", |b| {
        let formatter = TreeFormatter::new().with_simple_format(true);
        b.iter(|| black_box(formatter.format_result(black_box(&result))));
    });

    group.finish();
}

criterion_group!(benches, cache_impact_test);
criterion_main!(benches);
