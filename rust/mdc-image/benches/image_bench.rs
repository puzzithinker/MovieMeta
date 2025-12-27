use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use image::{DynamicImage, Rgba, RgbaImage};
use mdc_image::{CropMode, FaceLocation, ImageProcessor, ProcessorConfig};
use tempfile::TempDir;

/// Create a test image (solid color)
fn create_test_image(width: u32, height: u32, color: Rgba<u8>) -> DynamicImage {
    let img = RgbaImage::from_pixel(width, height, color);
    DynamicImage::ImageRgba8(img)
}

fn bench_image_load_save(c: &mut Criterion) {
    let temp_dir = TempDir::new().unwrap();
    let test_path = temp_dir.path().join("bench_load_save.png");
    let processor = ImageProcessor::default();

    // Create and save initial image
    let img = create_test_image(1920, 1080, Rgba([255, 0, 0, 255]));
    processor.save_image(&img, &test_path).unwrap();

    let mut group = c.benchmark_group("image_io");

    group.bench_function("load_1920x1080", |b| {
        b.iter(|| {
            let _img = processor.load_image(black_box(&test_path)).unwrap();
        })
    });

    group.bench_function("save_1920x1080", |b| {
        let img = create_test_image(1920, 1080, Rgba([0, 255, 0, 255]));
        let save_path = temp_dir.path().join("bench_save.png");
        b.iter(|| {
            processor.save_image(black_box(&img), black_box(&save_path)).unwrap();
        })
    });

    group.finish();
}

fn bench_resize_operations(c: &mut Criterion) {
    let processor = ImageProcessor::default();
    let mut group = c.benchmark_group("resize");

    // Test different image sizes
    let sizes = [(1920, 1080), (3840, 2160), (800, 600)];

    for (width, height) in sizes {
        let img = create_test_image(width, height, Rgba([128, 128, 128, 255]));

        group.bench_with_input(
            BenchmarkId::new("exact", format!("{}x{}", width, height)),
            &img,
            |b, img| {
                b.iter(|| {
                    let _resized = processor.resize(black_box(img), 400, 300);
                })
            },
        );

        group.bench_with_input(
            BenchmarkId::new("fit", format!("{}x{}", width, height)),
            &img,
            |b, img| {
                b.iter(|| {
                    let _fitted = processor.resize_fit(black_box(img), 800, 800);
                })
            },
        );
    }

    group.finish();
}

fn bench_crop_operations(c: &mut Criterion) {
    let temp_dir = TempDir::new().unwrap();
    let processor = ImageProcessor::default();
    let mut group = c.benchmark_group("crop");

    // Wide image (typical movie cover)
    let wide_img = create_test_image(1920, 1080, Rgba([255, 128, 0, 255]));
    let wide_path = temp_dir.path().join("bench_wide.png");
    processor.save_image(&wide_img, &wide_path).unwrap();

    // Tall image
    let tall_img = create_test_image(800, 1600, Rgba([0, 128, 255, 255]));
    let tall_path = temp_dir.path().join("bench_tall.png");
    processor.save_image(&tall_img, &tall_path).unwrap();

    group.bench_function("copy_mode", |b| {
        let output_path = temp_dir.path().join("bench_copy_out.png");
        b.iter(|| {
            processor
                .cut_image(
                    black_box(&wide_path),
                    black_box(&output_path),
                    black_box(CropMode::Copy),
                    black_box(None),
                )
                .unwrap();
        })
    });

    group.bench_function("smart_crop_wide_no_face", |b| {
        let output_path = temp_dir.path().join("bench_smart_wide_out.png");
        b.iter(|| {
            processor
                .cut_image(
                    black_box(&wide_path),
                    black_box(&output_path),
                    black_box(CropMode::Smart),
                    black_box(None),
                )
                .unwrap();
        })
    });

    group.bench_function("smart_crop_wide_with_face", |b| {
        let output_path = temp_dir.path().join("bench_smart_wide_face_out.png");
        let face = FaceLocation {
            center_x: 1200,
            top_y: 300,
            confidence: 0.95,
        };
        b.iter(|| {
            processor
                .cut_image(
                    black_box(&wide_path),
                    black_box(&output_path),
                    black_box(CropMode::Smart),
                    black_box(Some(face)),
                )
                .unwrap();
        })
    });

    group.bench_function("smart_crop_tall_no_face", |b| {
        let output_path = temp_dir.path().join("bench_smart_tall_out.png");
        b.iter(|| {
            processor
                .cut_image(
                    black_box(&tall_path),
                    black_box(&output_path),
                    black_box(CropMode::Smart),
                    black_box(None),
                )
                .unwrap();
        })
    });

    group.bench_function("force_smart_crop", |b| {
        let config = ProcessorConfig {
            force_smart_crop: true,
            ..Default::default()
        };
        let processor = ImageProcessor::new(config);
        let output_path = temp_dir.path().join("bench_force_smart_out.png");
        b.iter(|| {
            processor
                .cut_image(
                    black_box(&wide_path),
                    black_box(&output_path),
                    black_box(CropMode::Copy), // Will be forced to Smart
                    black_box(None),
                )
                .unwrap();
        })
    });

    group.finish();
}

fn bench_watermark(c: &mut Criterion) {
    let temp_dir = TempDir::new().unwrap();
    let processor = ImageProcessor::default();

    // Create base image
    let base_img = create_test_image(800, 600, Rgba([200, 200, 200, 255]));

    // Create watermark image (smaller, with transparency)
    let mut watermark_img = RgbaImage::from_pixel(100, 50, Rgba([0, 0, 0, 128]));
    for x in 0..100 {
        for y in 0..50 {
            watermark_img.put_pixel(x, y, Rgba([255, 255, 255, 128]));
        }
    }
    let watermark = DynamicImage::ImageRgba8(watermark_img);
    let watermark_path = temp_dir.path().join("bench_watermark.png");
    processor.save_image(&watermark, &watermark_path).unwrap();

    c.bench_function("add_watermark", |b| {
        b.iter(|| {
            let _result = processor
                .add_watermark(
                    black_box(&base_img),
                    black_box(&watermark_path),
                    black_box((50, 50)),
                )
                .unwrap();
        })
    });
}

fn bench_aspect_ratio_calculations(c: &mut Criterion) {
    let mut group = c.benchmark_group("aspect_ratio");

    // Test different aspect ratios
    let aspect_ratios = [2.0 / 3.0, 3.0 / 4.0, 9.0 / 16.0, 1.0];

    for ratio in aspect_ratios {
        let config = ProcessorConfig {
            aspect_ratio: ratio,
            ..Default::default()
        };
        let processor = ImageProcessor::new(config);

        group.bench_with_input(
            BenchmarkId::new("crop_width", format!("{:.3}", ratio)),
            &ratio,
            |b, _| {
                let img = create_test_image(1920, 1080, Rgba([100, 100, 100, 255]));
                b.iter(|| {
                    let _cropped = processor.resize(black_box(&img), 400, 600);
                })
            },
        );
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_image_load_save,
    bench_resize_operations,
    bench_crop_operations,
    bench_watermark,
    bench_aspect_ratio_calculations
);
criterion_main!(benches);
