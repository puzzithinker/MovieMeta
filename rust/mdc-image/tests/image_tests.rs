use image::{DynamicImage, GenericImageView, Rgba, RgbaImage};
use mdc_image::{CropMode, FaceDetector, FaceLocation, ImageProcessor, ProcessorConfig};
use tempfile::TempDir;

/// Create a test image (solid color)
fn create_test_image(width: u32, height: u32, color: Rgba<u8>) -> DynamicImage {
    let img = RgbaImage::from_pixel(width, height, color);
    DynamicImage::ImageRgba8(img)
}

#[test]
fn test_image_load_save() {
    let temp_dir = TempDir::new().unwrap();
    let test_path = temp_dir.path().join("test.png");

    let processor = ImageProcessor::default();

    // Create and save an image
    let img = create_test_image(100, 100, Rgba([255, 0, 0, 255]));
    processor.save_image(&img, &test_path).unwrap();

    // Load it back
    let loaded = processor.load_image(&test_path).unwrap();
    assert_eq!(loaded.dimensions(), (100, 100));
}

#[test]
fn test_resize_operations() {
    let processor = ImageProcessor::default();
    let img = create_test_image(800, 600, Rgba([0, 255, 0, 255]));

    // Exact resize
    let resized = processor.resize(&img, 400, 300);
    assert_eq!(resized.dimensions(), (400, 300));

    // Fit resize (maintains aspect ratio)
    let fitted = processor.resize_fit(&img, 200, 200);
    let (w, h) = fitted.dimensions();
    assert!(w <= 200 && h <= 200);
}

#[test]
fn test_crop_mode_copy() {
    let temp_dir = TempDir::new().unwrap();
    let fanart_path = temp_dir.path().join("fanart.png");
    let poster_path = temp_dir.path().join("poster.png");

    let processor = ImageProcessor::default();

    // Create and save fanart
    let img = create_test_image(300, 200, Rgba([0, 0, 255, 255]));
    processor.save_image(&img, &fanart_path).unwrap();

    // Copy mode
    processor
        .cut_image(&fanart_path, &poster_path, CropMode::Copy, None)
        .unwrap();

    // Verify poster was created and has same dimensions
    let poster = processor.load_image(&poster_path).unwrap();
    assert_eq!(poster.dimensions(), (300, 200));
}

#[test]
fn test_crop_mode_smart_width() {
    let temp_dir = TempDir::new().unwrap();
    let fanart_path = temp_dir.path().join("fanart_wide.png");
    let poster_path = temp_dir.path().join("poster_smart.png");

    let processor = ImageProcessor::default();

    // Create wide image (800x400, aspect > 2/3)
    let img = create_test_image(800, 400, Rgba([255, 255, 0, 255]));
    processor.save_image(&img, &fanart_path).unwrap();

    // Smart crop without face detection
    processor
        .cut_image(&fanart_path, &poster_path, CropMode::Smart, None)
        .unwrap();

    // Verify poster was cropped
    let poster = processor.load_image(&poster_path).unwrap();
    let (w, h) = poster.dimensions();

    // Should be cropped to approximately 2:3 ratio
    assert!(w < 800); // Width should be reduced
    assert_eq!(h, 400); // Height should remain
}

#[test]
fn test_crop_mode_smart_height() {
    let temp_dir = TempDir::new().unwrap();
    let fanart_path = temp_dir.path().join("fanart_tall.png");
    let poster_path = temp_dir.path().join("poster_tall.png");

    let processor = ImageProcessor::default();

    // Create tall image (400x800, aspect < 2/3)
    let img = create_test_image(400, 800, Rgba([255, 0, 255, 255]));
    processor.save_image(&img, &fanart_path).unwrap();

    // Smart crop without face detection
    processor
        .cut_image(&fanart_path, &poster_path, CropMode::Smart, None)
        .unwrap();

    // Verify poster was cropped
    let poster = processor.load_image(&poster_path).unwrap();
    let (w, h) = poster.dimensions();

    // Should be cropped to approximately 2:3 ratio
    assert_eq!(w, 400); // Width should remain
    assert!(h < 800); // Height should be reduced
}

#[test]
fn test_crop_with_face_location() {
    let temp_dir = TempDir::new().unwrap();
    let fanart_path = temp_dir.path().join("fanart_face.png");
    let poster_path = temp_dir.path().join("poster_face.png");

    let processor = ImageProcessor::default();

    // Create wide image
    let img = create_test_image(800, 400, Rgba([128, 128, 128, 255]));
    processor.save_image(&img, &fanart_path).unwrap();

    // Provide face location (center at x=600, top at y=50)
    let face = FaceLocation {
        center_x: 600,
        top_y: 50,
        confidence: 0.95,
    };

    // Smart crop with face detection
    processor
        .cut_image(&fanart_path, &poster_path, CropMode::Smart, Some(face))
        .unwrap();

    // Verify poster was created
    assert!(poster_path.exists());
}

#[test]
fn test_force_smart_crop_config() {
    let temp_dir = TempDir::new().unwrap();
    let fanart_path = temp_dir.path().join("fanart_force.png");
    let poster_path = temp_dir.path().join("poster_force.png");

    let config = ProcessorConfig {
        force_smart_crop: true,
        ..Default::default()
    };

    let processor = ImageProcessor::new(config);

    // Create wide image
    let img = create_test_image(800, 400, Rgba([64, 64, 64, 255]));
    processor.save_image(&img, &fanart_path).unwrap();

    // Use Copy mode, but should be forced to Smart
    processor
        .cut_image(&fanart_path, &poster_path, CropMode::Copy, None)
        .unwrap();

    // Verify poster was cropped (not copied)
    let poster = processor.load_image(&poster_path).unwrap();
    let (w, _h) = poster.dimensions();
    assert!(w < 800); // Should be cropped, not copied
}

#[test]
fn test_face_detector_availability() {
    let detector = FaceDetector::new();
    let available = detector.is_available();

    // This test just checks that availability detection works
    // It doesn't require face_recognition to be installed
    println!("Face detection available: {}", available);
}

#[test]
fn test_custom_aspect_ratio() {
    let config = ProcessorConfig {
        aspect_ratio: 0.75, // 3:4 instead of 2:3
        ..Default::default()
    };

    let processor = ImageProcessor::new(config);

    assert_eq!(processor.config.aspect_ratio, 0.75);
}
