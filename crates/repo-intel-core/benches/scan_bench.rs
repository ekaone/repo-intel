use criterion::{black_box, criterion_group, criterion_main, Criterion};
use repo_intel_core::{config::Config, scanner::scan};
use std::path::Path;

fn bench_scan_nextjs(c: &mut Criterion) {
    let fixture = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/nextjs-basic");
    let cfg = Config::default();

    c.bench_function("scan_nextjs_basic", |b| {
        b.iter(|| {
            scan(black_box(&fixture), black_box(&cfg)).expect("scan should succeed")
        })
    });
}

fn bench_scan_monorepo(c: &mut Criterion) {
    let fixture = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/monorepo");
    let cfg = Config::default();

    c.bench_function("scan_monorepo", |b| {
        b.iter(|| {
            scan(black_box(&fixture), black_box(&cfg)).expect("scan should succeed")
        })
    });
}

criterion_group!(benches, bench_scan_nextjs, bench_scan_monorepo);
criterion_main!(benches);
