use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use cs::{KeyExtractor, PatternMatcher, SearchQuery, TextSearcher, TraceDirection, TraceQuery};
use std::path::PathBuf;

/// Get the path to test fixtures
fn fixture_path(subdir: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join(subdir)
}

/// Benchmark text search operations using ripgrep
fn bench_text_search(c: &mut Criterion) {
    let mut group = c.benchmark_group("text_search");

    // Small project: Single directory with 4 files
    group.bench_function("small_project", |b| {
        let base_dir = fixture_path("rails-app");
        let searcher = TextSearcher::new(base_dir);
        b.iter(|| searcher.search(black_box("add new")).unwrap());
    });

    // Medium project: Search across code-examples
    group.bench_function("medium_project", |b| {
        let base_dir = fixture_path("code-examples");
        let searcher = TextSearcher::new(base_dir);
        b.iter(|| searcher.search(black_box("checkout")).unwrap());
    });

    // Large project: Search across all fixtures
    group.bench_function("large_project", |b| {
        let base_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests")
            .join("fixtures");
        let searcher = TextSearcher::new(base_dir);
        b.iter(|| searcher.search(black_box("function")).unwrap());
    });

    group.finish();
}

/// Benchmark YAML parsing and key extraction
fn bench_key_extraction(c: &mut Criterion) {
    let mut group = c.benchmark_group("key_extraction");

    // Small: Extract from single YAML file
    group.bench_function("small_yaml", |b| {
        let extractor = KeyExtractor::new();
        let base_dir = fixture_path("rails-app");
        b.iter(|| {
            extractor
                .extract(black_box(&base_dir), black_box("add new"))
                .unwrap()
        });
    });

    // Medium: Extract from multiple YAML files
    group.bench_function("medium_yaml", |b| {
        let extractor = KeyExtractor::new();
        let base_dir = fixture_path("rails-app");
        b.iter(|| {
            extractor
                .extract(black_box(&base_dir), black_box("invoice"))
                .unwrap()
        });
    });

    group.finish();
}

/// Benchmark pattern matching for code references
fn bench_pattern_matching(c: &mut Criterion) {
    let mut group = c.benchmark_group("pattern_matching");

    // Small: Find usages in small codebase
    group.bench_function("small_codebase", |b| {
        let base_dir = fixture_path("rails-app");
        let matcher = PatternMatcher::new(base_dir);
        b.iter(|| {
            matcher
                .find_usages(black_box("invoice.labels.add_new"))
                .unwrap()
        });
    });

    // Medium: Find usages with partial keys
    group.bench_function("medium_codebase", |b| {
        let base_dir = fixture_path("rails-app");
        let matcher = PatternMatcher::new(base_dir);
        b.iter(|| {
            // Simulate searching for multiple key variations
            let _ = matcher.find_usages(black_box("invoice.labels")).unwrap();
            let _ = matcher.find_usages(black_box("labels.add_new")).unwrap();
        });
    });

    group.finish();
}

/// Benchmark end-to-end i18n search workflow
fn bench_end_to_end_search(c: &mut Criterion) {
    let mut group = c.benchmark_group("end_to_end_search");

    // Small project: Complete workflow on rails-app
    group.bench_function("small_project", |b| {
        let base_dir = fixture_path("rails-app");
        b.iter(|| {
            let query =
                SearchQuery::new(black_box("add new".to_string())).with_base_dir(base_dir.clone());
            cs::run_search(query).unwrap()
        });
    });

    // Medium project: Search with case sensitivity
    group.bench_function("medium_project_case_sensitive", |b| {
        let base_dir = fixture_path("rails-app");
        b.iter(|| {
            let query = SearchQuery::new(black_box("Add New".to_string()))
                .with_case_sensitive(true)
                .with_base_dir(base_dir.clone());
            cs::run_search(query).unwrap()
        });
    });

    // Large project: Search across all fixtures
    group.bench_function("large_project_all_fixtures", |b| {
        let base_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests")
            .join("fixtures");
        b.iter(|| {
            let query =
                SearchQuery::new(black_box("invoice".to_string())).with_base_dir(base_dir.clone());
            cs::run_search(query).unwrap()
        });
    });

    group.finish();
}

/// Benchmark call graph tracing
fn bench_call_tracing(c: &mut Criterion) {
    let mut group = c.benchmark_group("call_tracing");

    // Small: Forward trace with depth 1
    group.bench_function("forward_depth_1", |b| {
        let base_dir = fixture_path("code-examples");
        b.iter(|| {
            let query = TraceQuery::new(
                black_box("checkout".to_string()),
                TraceDirection::Forward,
                1,
            )
            .with_base_dir(base_dir.clone());
            cs::run_trace(query).unwrap()
        });
    });

    // Medium: Forward trace with depth 3
    group.bench_function("forward_depth_3", |b| {
        let base_dir = fixture_path("code-examples");
        b.iter(|| {
            let query = TraceQuery::new(
                black_box("checkout".to_string()),
                TraceDirection::Forward,
                3,
            )
            .with_base_dir(base_dir.clone());
            cs::run_trace(query).unwrap()
        });
    });

    // Large: Backward trace with depth 3
    group.bench_function("backward_depth_3", |b| {
        let base_dir = fixture_path("code-examples");
        b.iter(|| {
            let query = TraceQuery::new(
                black_box("processPayment".to_string()),
                TraceDirection::Backward,
                3,
            )
            .with_base_dir(base_dir.clone());
            cs::run_trace(query).unwrap()
        });
    });

    group.finish();
}

/// Benchmark different project sizes systematically
fn bench_project_sizes(c: &mut Criterion) {
    let mut group = c.benchmark_group("project_sizes");

    let sizes = [
        ("small", fixture_path("rails-app"), "add new"),
        ("medium", fixture_path("code-examples"), "checkout"),
        (
            "large",
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("tests")
                .join("fixtures"),
            "function",
        ),
    ];

    for (size, base_dir, query_text) in sizes.iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(size),
            &(base_dir.clone(), query_text.to_string()),
            |b, (dir, text)| {
                b.iter(|| {
                    let query =
                        SearchQuery::new(black_box(text.clone())).with_base_dir(dir.clone());
                    cs::run_search(query).unwrap()
                });
            },
        );
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_text_search,
    bench_key_extraction,
    bench_pattern_matching,
    bench_end_to_end_search,
    bench_call_tracing,
    bench_project_sizes
);
criterion_main!(benches);
