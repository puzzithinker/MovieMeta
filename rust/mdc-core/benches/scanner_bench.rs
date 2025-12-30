use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use mdc_core::scanner::{Scanner, ScannerConfig};
use std::fs::{self, File};
use tempfile::TempDir;

/// Create a large test directory with N files
fn create_large_test_dir(base: &std::path::Path, num_files: usize) {
    // Create subdirectories
    for i in 0..10 {
        let subdir = base.join(format!("dir_{}", i));
        fs::create_dir(&subdir).unwrap();

        // Create files in each subdirectory
        let files_per_dir = num_files / 10;
        for j in 0..files_per_dir {
            let filename = format!("MOVIE-{:05}.mp4", i * files_per_dir + j);
            File::create(subdir.join(filename)).unwrap();
        }
    }

    // Create some non-video files
    File::create(base.join("readme.txt")).unwrap();
    File::create(base.join("info.md")).unwrap();
}

fn bench_scan_sizes(c: &mut Criterion) {
    let mut group = c.benchmark_group("scanner_sizes");

    for size in [100, 500, 1000, 5000].iter() {
        let temp_dir = TempDir::new().unwrap();
        create_large_test_dir(temp_dir.path(), *size);

        let config = ScannerConfig {
            source_folder: temp_dir.path().to_path_buf(),
            ..Default::default()
        };

        group.bench_with_input(
            BenchmarkId::from_parameter(format!("{}_files", size)),
            size,
            |b, _size| {
                b.iter(|| {
                    let scanner = Scanner::new(config.clone());
                    let rt = tokio::runtime::Runtime::new().unwrap();
                    rt.block_on(async { scanner.scan().await })
                });
            },
        );
    }

    group.finish();
}

fn bench_scan_10k_files(c: &mut Criterion) {
    let temp_dir = TempDir::new().unwrap();
    create_large_test_dir(temp_dir.path(), 10000);

    let config = ScannerConfig {
        source_folder: temp_dir.path().to_path_buf(),
        ..Default::default()
    };

    c.bench_function("scan_10k_files", |b| {
        b.iter(|| {
            let scanner = Scanner::new(config.clone());
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async { scanner.scan().await })
        });
    });
}

fn bench_scan_with_filters(c: &mut Criterion) {
    let temp_dir = TempDir::new().unwrap();
    create_large_test_dir(temp_dir.path(), 1000);

    let mut group = c.benchmark_group("scanner_filters");

    // Baseline - no filters
    let config_baseline = ScannerConfig {
        source_folder: temp_dir.path().to_path_buf(),
        ..Default::default()
    };

    group.bench_function("no_filters", |b| {
        b.iter(|| {
            let scanner = Scanner::new(config_baseline.clone());
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async { scanner.scan().await })
        });
    });

    // With media type filter
    let config_media = ScannerConfig {
        source_folder: temp_dir.path().to_path_buf(),
        media_types: vec![".mp4".to_string(), ".avi".to_string()],
        ..Default::default()
    };

    group.bench_function("media_filter", |b| {
        b.iter(|| {
            let scanner = Scanner::new(config_media.clone());
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async { scanner.scan().await })
        });
    });

    // With CLI regex
    let config_regex = ScannerConfig {
        source_folder: temp_dir.path().to_path_buf(),
        cli_regex: Some(r"MOVIE-\d{5}".to_string()),
        ..Default::default()
    };

    group.bench_function("regex_filter", |b| {
        b.iter(|| {
            let scanner = Scanner::new(config_regex.clone());
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async { scanner.scan().await })
        });
    });

    group.finish();
}

fn bench_deep_directory_structure(c: &mut Criterion) {
    let temp_dir = TempDir::new().unwrap();
    let base = temp_dir.path();

    // Create deep nested structure: 10 levels deep
    let mut current = base.to_path_buf();
    for i in 0..10 {
        current = current.join(format!("level_{}", i));
        fs::create_dir(&current).unwrap();

        // Add 10 files at each level
        for j in 0..10 {
            File::create(current.join(format!("movie_{}.mp4", j))).unwrap();
        }
    }

    let config = ScannerConfig {
        source_folder: base.to_path_buf(),
        ..Default::default()
    };

    c.bench_function("deep_directory_10_levels", |b| {
        b.iter(|| {
            let scanner = Scanner::new(config.clone());
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async { scanner.scan().await })
        });
    });
}

criterion_group!(
    benches,
    bench_scan_sizes,
    bench_scan_10k_files,
    bench_scan_with_filters,
    bench_deep_directory_structure
);
criterion_main!(benches);
