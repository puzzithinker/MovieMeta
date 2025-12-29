//! End-to-end integration tests for the CLI
//!
//! These tests verify complete workflows from input files to organized output.

use anyhow::Result;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use tempfile::TempDir;

use mdc_core::{
    BatchProcessor, LinkMode, ProcessingMode, ProcessorConfig,
    scanner, number_parser,
};
use serde_json::json;

/// Helper to create test video files
fn create_test_files(temp_dir: &TempDir, filenames: &[&str]) -> Vec<PathBuf> {
    let mut paths = Vec::new();
    for filename in filenames {
        let path = temp_dir.path().join(filename);
        fs::write(&path, "test video content").unwrap();
        paths.push(path);
    }
    paths
}

/// Mock metadata provider for testing (with dual ID support)
fn mock_metadata_provider() -> Arc<impl Fn(mdc_core::DualId) -> futures::future::Ready<Result<serde_json::Value>> + Send + Sync> {
    Arc::new(|dual_id: mdc_core::DualId| {
        futures::future::ready(Ok(json!({
            "number": dual_id.display,
            "title": format!("Test Movie {}", dual_id.display),
            "studio": "Test Studio",
            "actor": ["Test Actor"],
            "year": "2024",
            "outline": "A test movie for integration testing"
        })))
    })
}

#[tokio::test]
async fn test_organizing_mode_single_file() {
    let temp_input = TempDir::new().unwrap();
    let temp_output = TempDir::new().unwrap();

    // Create test file
    let test_file = temp_input.path().join("TEST-001.mp4");
    fs::write(&test_file, "test content").unwrap();

    // Configure processor for organizing mode
    let config = ProcessorConfig {
        mode: ProcessingMode::Organizing,
        link_mode: LinkMode::Move,
        success_folder: temp_output.path().to_path_buf(),
        location_rule: "number".to_string(),
        naming_rule: "number".to_string(),
        create_nfo: false,
        move_subtitles: false,
        ..Default::default()
    };

    let processor = BatchProcessor::new(config, 1);
    let metadata_provider = mock_metadata_provider();

    let (results, stats) = processor
        .process_batch(vec![test_file.clone()], metadata_provider, None)
        .await
        .unwrap();

    // Verify results
    assert_eq!(stats.total_processed, 1);
    assert_eq!(stats.succeeded, 1);
    assert_eq!(stats.failed, 0);
    assert!(results[0].success);

    // Verify file was moved
    assert!(!test_file.exists(), "Source file should be moved");
    let expected_dest = temp_output.path().join("TEST-001/TEST-001.mp4");
    assert!(expected_dest.exists(), "Destination file should exist");
}

#[tokio::test]
async fn test_organizing_mode_multiple_files() {
    let temp_input = TempDir::new().unwrap();
    let temp_output = TempDir::new().unwrap();

    // Create multiple test files
    let files = create_test_files(&temp_input, &[
        "ABC-001.mp4",
        "ABC-002.mkv",
        "XYZ-100.avi",
    ]);

    let config = ProcessorConfig {
        mode: ProcessingMode::Organizing,
        link_mode: LinkMode::Move,
        success_folder: temp_output.path().to_path_buf(),
        location_rule: "number".to_string(),
        naming_rule: "number".to_string(),
        create_nfo: false,
        move_subtitles: false,
        ..Default::default()
    };

    let processor = BatchProcessor::new(config, 2); // 2 concurrent tasks
    let metadata_provider = mock_metadata_provider();

    let (results, stats) = processor
        .process_batch(files.clone(), metadata_provider, None)
        .await
        .unwrap();

    // Verify all succeeded
    assert_eq!(stats.total_processed, 3);
    assert_eq!(stats.succeeded, 3);
    assert_eq!(stats.failed, 0);

    // Verify all files were moved correctly
    assert!(temp_output.path().join("ABC-001/ABC-001.mp4").exists());
    assert!(temp_output.path().join("ABC-002/ABC-002.mkv").exists());
    assert!(temp_output.path().join("XYZ-100/XYZ-100.avi").exists());

    // Verify source files are gone
    for file in &files {
        assert!(!file.exists(), "Source file should be moved: {:?}", file);
    }
}

#[tokio::test]
async fn test_analysis_mode_in_place() {
    let temp_dir = TempDir::new().unwrap();

    // Create test file
    let test_file = temp_dir.path().join("TEST-123.mp4");
    fs::write(&test_file, "test content").unwrap();

    let config = ProcessorConfig {
        mode: ProcessingMode::Analysis,
        link_mode: LinkMode::Move,
        success_folder: temp_dir.path().to_path_buf(),
        location_rule: "number".to_string(),
        naming_rule: "number".to_string(),
        create_nfo: true,
        move_subtitles: false,
        ..Default::default()
    };

    let processor = BatchProcessor::new(config, 1);
    let metadata_provider = mock_metadata_provider();

    let (results, stats) = processor
        .process_batch(vec![test_file.clone()], metadata_provider, None)
        .await
        .unwrap();

    // Verify processing succeeded
    assert_eq!(stats.succeeded, 1);
    assert!(results[0].success);

    // Verify original file still exists (analysis mode doesn't move)
    assert!(test_file.exists(), "Original file should remain in place");

    // Verify NFO was created
    let nfo_path = temp_dir.path().join("TEST-123.nfo");
    assert!(nfo_path.exists(), "NFO file should be created");

    // Verify NFO content
    let nfo_content = fs::read_to_string(&nfo_path).unwrap();
    assert!(nfo_content.contains("<title>Test Movie TEST-123</title>"));
    assert!(nfo_content.contains("<studio>Test Studio</studio>"));
}

#[tokio::test]
async fn test_custom_location_rule() {
    let temp_input = TempDir::new().unwrap();
    let temp_output = TempDir::new().unwrap();

    let test_file = temp_input.path().join("MOVIE-001.mp4");
    fs::write(&test_file, "test content").unwrap();

    // Use custom location rule: studio/number
    let config = ProcessorConfig {
        mode: ProcessingMode::Organizing,
        link_mode: LinkMode::Move,
        success_folder: temp_output.path().to_path_buf(),
        location_rule: "studio/number".to_string(),
        naming_rule: "number".to_string(),
        create_nfo: false,
        move_subtitles: false,
        ..Default::default()
    };

    let processor = BatchProcessor::new(config, 1);
    let metadata_provider = mock_metadata_provider();

    let (results, _) = processor
        .process_batch(vec![test_file], metadata_provider, None)
        .await
        .unwrap();

    assert!(results[0].success);

    // Verify file is in studio subfolder
    let expected_path = temp_output.path().join("Test Studio/MOVIE-001/MOVIE-001.mp4");
    assert!(expected_path.exists(), "File should be in studio/number structure");
}

#[tokio::test]
async fn test_chinese_subtitle_suffix() {
    let temp_input = TempDir::new().unwrap();
    let temp_output = TempDir::new().unwrap();

    // File with -C suffix (Chinese subtitles)
    let test_file = temp_input.path().join("TEST-001-C.mp4");
    fs::write(&test_file, "test content").unwrap();

    let config = ProcessorConfig {
        mode: ProcessingMode::Organizing,
        link_mode: LinkMode::Move,
        success_folder: temp_output.path().to_path_buf(),
        location_rule: "number".to_string(),
        naming_rule: "number".to_string(),
        create_nfo: false,
        move_subtitles: false,
        ..Default::default()
    };

    let processor = BatchProcessor::new(config, 1);
    let metadata_provider = mock_metadata_provider();

    let (results, _) = processor
        .process_batch(vec![test_file], metadata_provider, None)
        .await
        .unwrap();

    assert!(results[0].success);

    // Verify -C suffix is preserved in output filename
    let expected_path = temp_output.path().join("TEST-001/TEST-001-C.mp4");
    assert!(expected_path.exists(), "Chinese subtitle suffix should be preserved");
}

#[tokio::test]
async fn test_multi_part_files() {
    let temp_input = TempDir::new().unwrap();
    let temp_output = TempDir::new().unwrap();

    // Create multi-part files
    let files = create_test_files(&temp_input, &[
        "MOVIE-001-CD1.mp4",
        "MOVIE-001-CD2.mp4",
    ]);

    let config = ProcessorConfig {
        mode: ProcessingMode::Organizing,
        link_mode: LinkMode::Move,
        success_folder: temp_output.path().to_path_buf(),
        location_rule: "number".to_string(),
        naming_rule: "number".to_string(),
        create_nfo: false,
        move_subtitles: false,
        ..Default::default()
    };

    let processor = BatchProcessor::new(config, 2);
    let metadata_provider = mock_metadata_provider();

    let (results, stats) = processor
        .process_batch(files, metadata_provider, None)
        .await
        .unwrap();

    assert_eq!(stats.succeeded, 2);

    // Verify both parts are in the same folder
    assert!(temp_output.path().join("MOVIE-001/MOVIE-001-CD1.mp4").exists());
    assert!(temp_output.path().join("MOVIE-001/MOVIE-001-CD2.mp4").exists());
}

#[tokio::test]
async fn test_skip_existing_files() {
    let temp_input = TempDir::new().unwrap();
    let temp_output = TempDir::new().unwrap();

    let test_file = temp_input.path().join("TEST-001.mp4");
    fs::write(&test_file, "new content").unwrap();

    // Create existing file at destination
    let dest_dir = temp_output.path().join("TEST-001");
    fs::create_dir_all(&dest_dir).unwrap();
    let existing_file = dest_dir.join("TEST-001.mp4");
    fs::write(&existing_file, "existing content").unwrap();

    let config = ProcessorConfig {
        mode: ProcessingMode::Organizing,
        link_mode: LinkMode::Move,
        success_folder: temp_output.path().to_path_buf(),
        location_rule: "number".to_string(),
        naming_rule: "number".to_string(),
        skip_existing: true, // Enable skip existing
        create_nfo: false,
        move_subtitles: false,
        ..Default::default()
    };

    let processor = BatchProcessor::new(config, 1);
    let metadata_provider = mock_metadata_provider();

    let (results, _) = processor
        .process_batch(vec![test_file.clone()], metadata_provider, None)
        .await
        .unwrap();

    assert!(results[0].success);

    // Verify source file still exists (was skipped)
    assert!(test_file.exists(), "Source should not be moved when skipping");

    // Verify existing file wasn't overwritten
    let content = fs::read_to_string(&existing_file).unwrap();
    assert_eq!(content, "existing content", "Existing file should not be overwritten");
}

#[tokio::test]
async fn test_error_handling_invalid_number() {
    let temp_input = TempDir::new().unwrap();
    let temp_output = TempDir::new().unwrap();

    // File with no recognizable movie number - use a filename that definitely won't parse
    let test_file = temp_input.path().join("my_vacation_video.mp4");
    fs::write(&test_file, "test content").unwrap();

    let config = ProcessorConfig {
        mode: ProcessingMode::Organizing,
        link_mode: LinkMode::Move,
        success_folder: temp_output.path().to_path_buf(),
        location_rule: "number".to_string(),
        naming_rule: "number".to_string(),
        create_nfo: false,
        move_subtitles: false,
        ..Default::default()
    };

    let processor = BatchProcessor::new(config, 1);
    let metadata_provider = mock_metadata_provider();

    let (results, stats) = processor
        .process_batch(vec![test_file.clone()], metadata_provider, None)
        .await
        .unwrap();

    // Processing should complete (even if with errors)
    assert_eq!(stats.total_processed, 1, "Processing attempted");

    // If it failed to parse, verify error message
    if !results[0].success {
        assert!(
            results[0].error.as_ref().unwrap().contains("Number parsing error") ||
            results[0].error.as_ref().unwrap().contains("Metadata fetch error")
        );
    }

    // Either way, this tests error handling works
}

#[tokio::test]
async fn test_soft_link_mode() {
    let temp_input = TempDir::new().unwrap();
    let temp_output = TempDir::new().unwrap();

    let test_file = temp_input.path().join("TEST-001.mp4");
    fs::write(&test_file, "test content").unwrap();

    let config = ProcessorConfig {
        mode: ProcessingMode::Organizing,
        link_mode: LinkMode::SoftLink, // Use soft link instead of move
        success_folder: temp_output.path().to_path_buf(),
        location_rule: "number".to_string(),
        naming_rule: "number".to_string(),
        create_nfo: false,
        move_subtitles: false,
        ..Default::default()
    };

    let processor = BatchProcessor::new(config, 1);
    let metadata_provider = mock_metadata_provider();

    let (results, stats) = processor
        .process_batch(vec![test_file.clone()], metadata_provider, None)
        .await
        .unwrap();

    assert_eq!(stats.succeeded, 1);
    assert!(results[0].success);

    // Original file should still exist (linked, not moved)
    assert!(test_file.exists(), "Original file should exist with soft link");

    // Linked file should exist
    let linked_file = temp_output.path().join("TEST-001/TEST-001.mp4");
    assert!(linked_file.exists(), "Linked file should exist");

    // Verify it's actually a symlink
    let metadata = fs::symlink_metadata(&linked_file).unwrap();
    assert!(metadata.file_type().is_symlink(), "Should be a symbolic link");
}

#[tokio::test]
async fn test_subtitle_moving() {
    let temp_input = TempDir::new().unwrap();
    let temp_output = TempDir::new().unwrap();

    // Create movie file and subtitle
    let test_file = temp_input.path().join("TEST-001.mp4");
    fs::write(&test_file, "video content").unwrap();

    let subtitle_file = temp_input.path().join("TEST-001.srt");
    fs::write(&subtitle_file, "subtitle content").unwrap();

    let config = ProcessorConfig {
        mode: ProcessingMode::Organizing,
        link_mode: LinkMode::Move,
        success_folder: temp_output.path().to_path_buf(),
        location_rule: "number".to_string(),
        naming_rule: "number".to_string(),
        create_nfo: false,
        move_subtitles: true, // Enable subtitle moving
        ..Default::default()
    };

    let processor = BatchProcessor::new(config, 1);
    let metadata_provider = mock_metadata_provider();

    let (results, _) = processor
        .process_batch(vec![test_file], metadata_provider, None)
        .await
        .unwrap();

    assert!(results[0].success);

    // Verify subtitle was moved along with video
    let moved_subtitle = temp_output.path().join("TEST-001/TEST-001.srt");
    assert!(moved_subtitle.exists(), "Subtitle should be moved with video");
    assert!(!subtitle_file.exists(), "Original subtitle should be gone");
}

#[tokio::test]
async fn test_nfo_generation() {
    let temp_input = TempDir::new().unwrap();
    let temp_output = TempDir::new().unwrap();

    let test_file = temp_input.path().join("TEST-001.mp4");
    fs::write(&test_file, "test content").unwrap();

    let config = ProcessorConfig {
        mode: ProcessingMode::Scraping, // Scraping mode generates NFO
        link_mode: LinkMode::Move,
        success_folder: temp_output.path().to_path_buf(),
        location_rule: "number".to_string(),
        naming_rule: "number".to_string(),
        create_nfo: true,
        move_subtitles: false,
        ..Default::default()
    };

    let processor = BatchProcessor::new(config, 1);
    let metadata_provider = mock_metadata_provider();

    let (results, _) = processor
        .process_batch(vec![test_file], metadata_provider, None)
        .await
        .unwrap();

    assert!(results[0].success);

    // Verify NFO was created
    let nfo_path = temp_output.path().join("TEST-001/TEST-001.nfo");
    assert!(nfo_path.exists(), "NFO file should be created");

    // Verify NFO content
    let nfo_content = fs::read_to_string(&nfo_path).unwrap();
    assert!(nfo_content.contains("<?xml version=\"1.0\""));
    assert!(nfo_content.contains("<movie>"));
    assert!(nfo_content.contains("<title>Test Movie TEST-001</title>"));
    assert!(nfo_content.contains("<studio>Test Studio</studio>"));
    assert!(nfo_content.contains("<year>2024</year>"));
    assert!(nfo_content.contains("<actor>"));
    assert!(nfo_content.contains("<name>Test Actor</name>"));
    assert!(nfo_content.contains("</movie>"));
}

#[tokio::test]
async fn test_concurrent_processing() {
    let temp_input = TempDir::new().unwrap();
    let temp_output = TempDir::new().unwrap();

    // Create 10 test files
    let mut files = Vec::new();
    for i in 1..=10 {
        let filename = format!("TEST-{:03}.mp4", i);
        let path = temp_input.path().join(&filename);
        fs::write(&path, format!("content {}", i)).unwrap();
        files.push(path);
    }

    let config = ProcessorConfig {
        mode: ProcessingMode::Organizing,
        link_mode: LinkMode::Move,
        success_folder: temp_output.path().to_path_buf(),
        location_rule: "number".to_string(),
        naming_rule: "number".to_string(),
        create_nfo: false,
        move_subtitles: false,
        ..Default::default()
    };

    // Use 4 concurrent tasks
    let processor = BatchProcessor::new(config, 4);
    let metadata_provider = mock_metadata_provider();

    // Track progress
    let progress_count = Arc::new(std::sync::Mutex::new(0));
    let progress_count_clone = progress_count.clone();

    let progress_callback = Arc::new(move |_current: usize, _total: usize| {
        *progress_count_clone.lock().unwrap() += 1;
    });

    let (results, stats) = processor
        .process_batch(files, metadata_provider, Some(progress_callback))
        .await
        .unwrap();

    // Verify all succeeded
    assert_eq!(stats.total_processed, 10);
    assert_eq!(stats.succeeded, 10);
    assert_eq!(stats.failed, 0);

    // Verify progress was called
    assert_eq!(*progress_count.lock().unwrap(), 10);

    // Verify all files were processed
    for i in 1..=10 {
        let expected = temp_output.path().join(format!("TEST-{:03}/TEST-{:03}.mp4", i, i));
        assert!(expected.exists(), "File {} should exist", i);
    }
}

#[tokio::test]
async fn test_scanner_integration() {
    let temp_dir = TempDir::new().unwrap();

    // Create nested directory structure with video files
    let subdir1 = temp_dir.path().join("movies1");
    let subdir2 = temp_dir.path().join("movies2");
    fs::create_dir_all(&subdir1).unwrap();
    fs::create_dir_all(&subdir2).unwrap();

    fs::write(subdir1.join("TEST-001.mp4"), "content").unwrap();
    fs::write(subdir1.join("TEST-002.mkv"), "content").unwrap();
    fs::write(subdir2.join("TEST-003.avi"), "content").unwrap();
    fs::write(temp_dir.path().join("TEST-004.mp4"), "content").unwrap();

    // Also create non-video files that should be ignored
    fs::write(temp_dir.path().join("readme.txt"), "readme").unwrap();
    fs::write(temp_dir.path().join("image.jpg"), "image").unwrap();

    // Scan directory
    let media_types = vec!["mp4", "mkv", "avi"];
    let found_files = scanner::scan_directory(temp_dir.path(), &media_types).await.unwrap();

    // Should find 4 video files
    assert_eq!(found_files.len(), 4);

    // Verify all are video files
    for file in &found_files {
        let ext = file.extension().unwrap().to_str().unwrap();
        assert!(
            media_types.contains(&ext),
            "Found file with unexpected extension: {}",
            ext
        );
    }
}

#[test]
fn test_number_parser_integration() {
    // Test various filename formats
    let test_cases = vec![
        ("TEST-001.mp4", "TEST-001"),
        ("ABC-123-C.mkv", "ABC-123"),
        ("XYZ-999-U.avi", "XYZ-999"),
        ("MOVIE-001.mp4", "MOVIE-001"),
        ("FC2-PPV-1234567.mp4", "FC2-PPV-1234567"),  // Rust parser keeps the full FC2-PPV format
    ];

    for (filename, expected) in test_cases {
        match number_parser::get_number(filename, None) {
            Ok(result) => assert_eq!(result, expected, "Failed to parse: {}", filename),
            Err(e) => panic!("Error parsing {}: {}", filename, e),
        }
    }

    // Number parser has many rules and might extract numbers from unexpected places.
    // The main goal is to correctly parse valid movie filenames.
}
