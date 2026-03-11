/// Benchmarks: walker performance on real and synthetic fixture trees.
///
/// Performance targets (from PLAN.md):
///   - Walk 10,000-file repo:  < 30ms  (walker only)
///   - Full scan pipeline:     < 200ms (scan + detect, measured by Phase 1 exit criteria)
///
/// Run with: `cargo bench`
/// HTML reports: `target/criterion/report/index.html`
use std::fs;
use std::path::Path;

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use repo_intel_core::{config::Config, run_pipeline, scanner};

// ── Synthetic fixture generation ──────────────────────────────────────────────

/// Create a synthetic directory tree with `file_count` files spread across
/// a shallow structure (depth ≤ 4, breadth ≤ 20 files per dir).
///
/// Mimics a real TypeScript project: mix of .ts, .tsx, .test.ts, .css files.
fn create_synthetic_tree(root: &Path, file_count: usize) {
    let extensions = [".ts", ".tsx", ".test.ts", ".css", ".json", ".md"];
    let dirs = [
        "src/components",
        "src/hooks",
        "src/services",
        "src/utils",
        "src/pages",
        "src/lib",
        "src/store",
        "src/types",
    ];

    // Create all directories
    for dir in &dirs {
        fs::create_dir_all(root.join(dir)).unwrap();
    }

    // Distribute files evenly across dirs
    let files_per_dir = (file_count / dirs.len()).max(1);

    for (dir_idx, dir) in dirs.iter().enumerate() {
        let dir_path = root.join(dir);
        let count = if dir_idx == 0 {
            // First dir gets the remainder
            file_count - files_per_dir * (dirs.len() - 1)
        } else {
            files_per_dir
        };

        for i in 0..count {
            let ext = extensions[i % extensions.len()];
            let name = format!("file_{i}{ext}");
            fs::write(dir_path.join(&name), "// placeholder").unwrap();
        }
    }

    // Add a realistic package.json at root
    fs::write(
        root.join("package.json"),
        r#"{"name":"bench-project","dependencies":{"next":"14","react":"18","tailwindcss":"3"},"devDependencies":{"vitest":"1","typescript":"5"}}"#,
    ).unwrap();
}

// ── Walker benchmarks ─────────────────────────────────────────────────────────

fn bench_walker(c: &mut Criterion) {
    let tmp = tempfile::TempDir::new().unwrap();

    // Create a 10k-file synthetic tree once, reuse across runs
    create_synthetic_tree(tmp.path(), 10_000);

    let mut group = c.benchmark_group("walker");

    group.bench_function("walk_10k_files", |b| {
        b.iter(|| repo_intel_core::scanner::walker::walk(black_box(tmp.path()), black_box(8)))
    });

    // Also bench smaller sizes to see scaling
    for size in [100, 1_000, 5_000] {
        let small_tmp = tempfile::TempDir::new().unwrap();
        create_synthetic_tree(small_tmp.path(), size);

        group.bench_with_input(BenchmarkId::new("walk_n_files", size), &size, |b, _| {
            b.iter(|| {
                repo_intel_core::scanner::walker::walk(black_box(small_tmp.path()), black_box(8))
            })
        });
    }

    group.finish();
}

// ── Full scan pipeline benchmarks ─────────────────────────────────────────────

fn bench_scan_pipeline(c: &mut Criterion) {
    let tmp = tempfile::TempDir::new().unwrap();
    create_synthetic_tree(tmp.path(), 10_000);

    let cfg = Config::default();

    let mut group = c.benchmark_group("scan_pipeline");

    // scanner::scan (walk + signal collection)
    group.bench_function("scan_10k", |b| {
        b.iter(|| scanner::scan(black_box(tmp.path()), black_box(&cfg)).unwrap())
    });

    group.finish();
}

// ── Full pipeline benchmark ───────────────────────────────────────────────────

fn bench_full_pipeline(c: &mut Criterion) {
    let tmp = tempfile::TempDir::new().unwrap();
    create_synthetic_tree(tmp.path(), 10_000);

    let cfg = Config::default();

    let mut group = c.benchmark_group("full_pipeline");

    // scan → detect → build context (everything Rust does)
    group.bench_function("run_pipeline_10k", |b| {
        b.iter(|| run_pipeline(black_box(tmp.path()), black_box(&cfg)).unwrap())
    });

    group.finish();
}

// ── Fixture benchmarks ────────────────────────────────────────────────────────

fn bench_fixtures(c: &mut Criterion) {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let fixtures_dir = manifest_dir.join("tests").join("fixtures");
    let cfg = Config::default();

    let mut group = c.benchmark_group("fixtures");

    for fixture_name in &[
        "nextjs-basic",
        "react-spa",
        "node-api",
        "rust-axum",
        "monorepo",
    ] {
        let fixture_path = fixtures_dir.join(fixture_name);
        if !fixture_path.exists() {
            continue;
        }

        group.bench_with_input(
            BenchmarkId::new("full_pipeline", fixture_name),
            fixture_name,
            |b, _| b.iter(|| run_pipeline(black_box(&fixture_path), black_box(&cfg)).unwrap()),
        );
    }

    group.finish();
}

// ── Signal file parsing benchmarks ───────────────────────────────────────────

fn bench_signal_parsing(c: &mut Criterion) {
    // A realistic large package.json with many dependencies
    let large_pkg = r#"{
        "name": "large-project",
        "dependencies": {
            "next": "14", "react": "18", "react-dom": "18",
            "@prisma/client": "5", "zustand": "4", "@tanstack/react-query": "5",
            "axios": "1", "zod": "3", "@apollo/client": "3", "graphql": "16",
            "stripe": "14", "resend": "2", "socket.io": "4",
            "tailwindcss": "3", "styled-components": "6"
        },
        "devDependencies": {
            "typescript": "5", "vitest": "1", "@testing-library/react": "16",
            "@playwright/test": "1", "vite": "5", "tsup": "8",
            "prisma": "5", "@types/node": "20"
        }
    }"#;

    let tmp = tempfile::TempDir::new().unwrap();
    fs::write(tmp.path().join("package.json"), large_pkg).unwrap();
    let cfg = Config::default();

    c.bench_function("parse_large_package_json", |b| {
        b.iter(|| scanner::scan(black_box(tmp.path()), black_box(&cfg)).unwrap())
    });
}

// ── Criterion groups ──────────────────────────────────────────────────────────

criterion_group!(
    benches,
    bench_walker,
    bench_scan_pipeline,
    bench_full_pipeline,
    bench_fixtures,
    bench_signal_parsing,
);
criterion_main!(benches);
