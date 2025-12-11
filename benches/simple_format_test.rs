use criterion::{black_box, criterion_group, criterion_main, Criterion};
use cs::output::TreeFormatter;
use cs::{CodeReference, SearchResult, TranslationEntry};
use std::path::PathBuf;

fn simple_benchmark_test(c: &mut Criterion) {
    // Create a simple test result
    let result = SearchResult {
        query: "test".to_string(),
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

    c.bench_function("simple_format_test", |b| {
        let formatter = TreeFormatter::new().with_simple_format(true);
        b.iter(|| black_box(formatter.format_result(black_box(&result))));
    });

    c.bench_function("tree_format_test", |b| {
        let formatter = TreeFormatter::new().with_simple_format(false);
        b.iter(|| black_box(formatter.format_result(black_box(&result))));
    });
}

criterion_group!(benches, simple_benchmark_test);
criterion_main!(benches);
