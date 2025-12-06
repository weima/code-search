use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use std::path::{Path, PathBuf};
use std::time::Duration;

use cs::parse::YamlParser;

/// Benchmark configuration for large translation files
struct BenchmarkFile {
    name: &'static str,
    path: &'static str,
    query: &'static str,
}

const DISCOURSE_BENCHMARKS: &[BenchmarkFile] = &[
    BenchmarkFile {
        name: "Arabic (605KB, 12.4K lines)",
        path: "/tmp/discourse/config/locales/client.ar.yml",
        query: "Log In",
    },
    BenchmarkFile {
        name: "Polish (461KB, 8.8K lines)",
        path: "/tmp/discourse/config/locales/client.pl_PL.yml",
        query: "Zaloguj",
    },
    BenchmarkFile {
        name: "English (389KB, 8.7K lines)",
        path: "/tmp/discourse/config/locales/client.en.yml",
        query: "Log In",
    },
    BenchmarkFile {
        name: "German (429KB, 9.2K lines)",
        path: "/tmp/discourse/config/locales/client.de.yml",
        query: "Anmelden",
    },
];

/// Benchmark: Bottom-up parsing with query vs. full YAML parsing
fn bench_bottom_up_vs_full_parse(c: &mut Criterion) {
    let mut group = c.benchmark_group("parsing_comparison");
    group.sample_size(10);
    group.measurement_time(Duration::from_secs(30));

    for bench in DISCOURSE_BENCHMARKS {
        let path = PathBuf::from(bench.path);
        if !path.exists() {
            eprintln!("⚠️  Skipping '{}': file not found", bench.name);
            continue;
        }

        // Bottom-up parsing with query (optimized)
        group.bench_with_input(
            BenchmarkId::new("optimized_with_query", bench.name),
            &path,
            |b, path| {
                b.iter(|| {
                    YamlParser::parse_file_with_query(black_box(path), black_box(Some(bench.query)))
                        .unwrap()
                });
            },
        );

        // Full YAML parsing (unoptimized)
        group.bench_with_input(
            BenchmarkId::new("full_parse_no_query", bench.name),
            &path,
            |b, path| {
                b.iter(|| YamlParser::parse_file(black_box(path)).unwrap());
            },
        );
    }

    group.finish();
}

/// Benchmark: Grep prefilter performance
fn bench_grep_prefilter(c: &mut Criterion) {
    let mut group = c.benchmark_group("grep_prefilter");
    group.sample_size(50);
    group.measurement_time(Duration::from_secs(20));

    for bench in DISCOURSE_BENCHMARKS {
        let path = PathBuf::from(bench.path);
        if !path.exists() {
            continue;
        }

        // Match found
        group.bench_with_input(
            BenchmarkId::new("match_found", bench.name),
            &path,
            |b, path| {
                b.iter(|| {
                    YamlParser::contains_query(black_box(path), black_box(bench.query)).unwrap()
                });
            },
        );

        // No match (fastest path)
        group.bench_with_input(
            BenchmarkId::new("no_match_fast_path", bench.name),
            &path,
            |b, path| {
                b.iter(|| {
                    YamlParser::contains_query(black_box(path), black_box("xyzNonExistent99999"))
                        .unwrap()
                });
            },
        );
    }

    group.finish();
}

/// Benchmark: Complete workflow including grep prefilter
fn bench_complete_workflow(c: &mut Criterion) {
    let mut group = c.benchmark_group("complete_workflow");
    group.sample_size(10);
    group.measurement_time(Duration::from_secs(30));

    for bench in DISCOURSE_BENCHMARKS {
        let path = PathBuf::from(bench.path);
        if !path.exists() {
            continue;
        }

        // Simulate real-world: check if file contains query, then parse if match
        group.bench_with_input(
            BenchmarkId::new("prefilter_then_parse", bench.name),
            &path,
            |b, path| {
                b.iter(|| {
                    if YamlParser::contains_query(path, bench.query).unwrap() {
                        YamlParser::parse_file_with_query(
                            black_box(path),
                            black_box(Some(bench.query)),
                        )
                        .unwrap()
                    } else {
                        Vec::new()
                    }
                });
            },
        );

        // No match scenario (grep returns false, skip parsing)
        group.bench_with_input(
            BenchmarkId::new("prefilter_skip_no_match", bench.name),
            &path,
            |b, path| {
                b.iter(|| {
                    if YamlParser::contains_query(path, "xyzNonExistent99999").unwrap() {
                        YamlParser::parse_file_with_query(
                            black_box(path),
                            black_box(Some("xyzNonExistent99999")),
                        )
                        .unwrap()
                    } else {
                        Vec::new()
                    }
                });
            },
        );
    }

    group.finish();
}

/// Helper to print file statistics
fn print_file_stats() {
    println!("\n📊 Benchmark File Statistics:");
    println!("{:<40} {:>10} {:>10}", "File", "Size", "Lines");
    println!("{}", "=".repeat(62));

    for bench in DISCOURSE_BENCHMARKS {
        let path = Path::new(bench.path);
        if path.exists() {
            if let Ok(metadata) = std::fs::metadata(path) {
                let size_kb = metadata.len() / 1024;
                if let Ok(content) = std::fs::read_to_string(path) {
                    let lines = content.lines().count();
                    println!(
                        "{:<40} {:>8} KB {:>10}",
                        bench.name.split(" (").next().unwrap(),
                        size_kb,
                        format!("{} lines", lines)
                    );
                }
            }
        }
    }
    println!();
}

fn print_benchmark_summary(_c: &mut Criterion) {
    print_file_stats();
}

criterion_group!(
    benches,
    print_benchmark_summary,
    bench_grep_prefilter,
    bench_bottom_up_vs_full_parse,
    bench_complete_workflow,
);

criterion_main!(benches);
