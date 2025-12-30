use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use mdc_core::number_parser::get_number;

fn bench_get_number(c: &mut Criterion) {
    let mut group = c.benchmark_group("number_parser");

    // Benchmark various filename patterns
    let test_cases = vec![
        ("Standard JAV", "SSIS-001.mp4"),
        ("With Quality", "FHD-ABP-123.mp4"),
        ("Tokyo Hot", "tokyo-hot-n1234.mp4"),
        ("Carib", "carib-123456-789.mp4"),
        ("HEYZO", "HEYZO-1234.mp4"),
        ("Heydouga", "heydouga-4017-123.mp4"),
        ("FC2", "FC2-PPV-1234567.mp4"),
        ("Complex", "javlibrary.com@FHD-SSIS-001-C-CD1.mp4"),
        ("Western", "x-art.18.05.15.mp4"),
        ("Underscore", "ABC_123.mp4"),
    ];

    for (name, filename) in test_cases {
        group.bench_with_input(
            BenchmarkId::from_parameter(name),
            filename,
            |b, filename| {
                b.iter(|| get_number(black_box(filename), None));
            },
        );
    }

    group.finish();
}

fn bench_is_uncensored(c: &mut Criterion) {
    use mdc_core::number_parser::is_uncensored;

    let mut group = c.benchmark_group("is_uncensored");

    let test_cases = vec![
        ("Censored", "SSIS-001"),
        ("Uncensored HEYZO", "HEYZO-1234"),
        ("Uncensored Carib", "123456-789"),
        ("Uncensored Tokyo Hot", "n1234"),
    ];

    for (name, number) in test_cases {
        group.bench_with_input(BenchmarkId::from_parameter(name), number, |b, number| {
            b.iter(|| is_uncensored(black_box(number), None));
        });
    }

    group.finish();
}

// Batch processing benchmark - simulate processing 100 files
fn bench_batch_processing(c: &mut Criterion) {
    let filenames: Vec<&str> = vec![
        "SSIS-001.mp4",
        "ABP-123.avi",
        "STARS-456.mkv",
        "MIDE-789.wmv",
        "FHD-IPX-001.mp4",
        "tokyo-hot-n1234.mp4",
        "carib-123456-789.mp4",
        "HEYZO-1234.mp4",
        "FC2-PPV-1234567.mp4",
        "x-art.18.05.15.mp4",
    ];

    // Repeat to make 100 filenames
    let mut batch: Vec<&str> = Vec::new();
    for _ in 0..10 {
        batch.extend_from_slice(&filenames);
    }

    c.bench_function("batch_100_files", |b| {
        b.iter(|| {
            for filename in &batch {
                let _ = get_number(black_box(filename), None);
            }
        });
    });
}

criterion_group!(
    benches,
    bench_get_number,
    bench_is_uncensored,
    bench_batch_processing
);
criterion_main!(benches);
