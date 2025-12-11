use criterion::{criterion_group, criterion_main, Criterion};

fn minimal_test(c: &mut Criterion) {
    c.bench_function("minimal", |b| {
        b.iter(|| {
            // Just a simple operation
            1 + 1
        });
    });
}

criterion_group!(benches, minimal_test);
criterion_main!(benches);
