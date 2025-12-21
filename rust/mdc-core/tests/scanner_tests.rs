use mdc_core::scanner::{Scanner, ScannerConfig};
use std::fs::{self, File};
use std::io::Write;
use tempfile::TempDir;

/// Helper to create test file structure
fn create_test_structure(base: &std::path::Path) {
    // Create root level files
    File::create(base.join("SSIS-001.mp4")).unwrap();
    File::create(base.join("ABP-123.avi")).unwrap();
    File::create(base.join("readme.txt")).unwrap();

    // Create subdirectory with movies
    let subdir1 = base.join("JAV");
    fs::create_dir(&subdir1).unwrap();
    File::create(subdir1.join("STARS-456.mkv")).unwrap();
    File::create(subdir1.join("IPX-789.mp4")).unwrap();

    // Create nested subdirectory
    let subdir2 = subdir1.join("Uncensored");
    fs::create_dir(&subdir2).unwrap();
    File::create(subdir2.join("HEYZO-1234.mp4")).unwrap();

    // Create escape folder (should be skipped)
    let escape = base.join("backup");
    fs::create_dir(&escape).unwrap();
    File::create(escape.join("old-movie.mp4")).unwrap();
}

#[tokio::test]
async fn test_basic_scan() {
    let temp_dir = TempDir::new().unwrap();
    create_test_structure(temp_dir.path());

    let config = ScannerConfig {
        source_folder: temp_dir.path().to_path_buf(),
        ..Default::default()
    };

    let scanner = Scanner::new(config);
    let (results, stats) = scanner.scan().await.unwrap();

    // Should find 6 video files (excluding readme.txt and backup folder)
    assert_eq!(results.len(), 6);
    assert_eq!(stats.total_files, 6);
}

#[tokio::test]
async fn test_escape_folder() {
    let temp_dir = TempDir::new().unwrap();
    create_test_structure(temp_dir.path());

    let config = ScannerConfig {
        source_folder: temp_dir.path().to_path_buf(),
        escape_folders: vec!["backup".to_string()],
        main_mode: 1, // Not mode 3, so escape folders are skipped
        ..Default::default()
    };

    let scanner = Scanner::new(config);
    let (results, _stats) = scanner.scan().await.unwrap();

    // Should find 5 video files (excluding backup folder)
    assert_eq!(results.len(), 5);

    // Verify no backup files
    for path in &results {
        assert!(!path.to_string_lossy().contains("backup"));
    }
}

#[tokio::test]
async fn test_escape_folder_mode3() {
    let temp_dir = TempDir::new().unwrap();
    create_test_structure(temp_dir.path());

    let config = ScannerConfig {
        source_folder: temp_dir.path().to_path_buf(),
        escape_folders: vec!["backup".to_string()],
        main_mode: 3, // Mode 3 doesn't skip escape folders
        ..Default::default()
    };

    let scanner = Scanner::new(config);
    let (results, _stats) = scanner.scan().await.unwrap();

    // Should find all 6 video files
    assert_eq!(results.len(), 6);
}

#[tokio::test]
async fn test_media_type_filter() {
    let temp_dir = TempDir::new().unwrap();
    create_test_structure(temp_dir.path());

    let config = ScannerConfig {
        source_folder: temp_dir.path().to_path_buf(),
        media_types: vec![".mp4".to_string()],
        ..Default::default()
    };

    let scanner = Scanner::new(config);
    let (results, _stats) = scanner.scan().await.unwrap();

    // Should find only .mp4 files (4 total)
    assert_eq!(results.len(), 4);

    for path in &results {
        assert_eq!(path.extension().unwrap(), "mp4");
    }
}

#[tokio::test]
async fn test_failed_list() {
    let temp_dir = TempDir::new().unwrap();
    create_test_structure(temp_dir.path());

    // Create failed_list.txt
    let failed_folder = temp_dir.path().join("failed");
    fs::create_dir(&failed_folder).unwrap();
    let failed_list_path = failed_folder.join("failed_list.txt");

    let failed_file = temp_dir.path().join("SSIS-001.mp4");
    let mut file = File::create(&failed_list_path).unwrap();
    writeln!(file, "{}", failed_file.display()).unwrap();

    let config = ScannerConfig {
        source_folder: temp_dir.path().to_path_buf(),
        failed_list_path: Some(failed_list_path),
        main_mode: 3, // Mode 3 respects failed list
        ignore_failed_list: false,
        ..Default::default()
    };

    let scanner = Scanner::new(config);
    let (results, stats) = scanner.scan().await.unwrap();

    // Should find 5 files (6 total - 1 in failed list)
    assert_eq!(results.len(), 5);
    assert_eq!(stats.skip_failed, 1);

    // Verify SSIS-001.mp4 is not in results
    for path in &results {
        assert!(!path.ends_with("SSIS-001.mp4"));
    }
}

#[tokio::test]
async fn test_nfo_skip_mode3() {
    let temp_dir = TempDir::new().unwrap();
    let base = temp_dir.path();

    // Create video file
    File::create(base.join("SSIS-001.mp4")).unwrap();

    // Create NFO file (just modified)
    File::create(base.join("SSIS-001.nfo")).unwrap();

    // Create old video without NFO
    File::create(base.join("ABP-123.mp4")).unwrap();

    let config = ScannerConfig {
        source_folder: base.to_path_buf(),
        main_mode: 3,
        nfo_skip_days: 30, // Skip NFOs modified within 30 days
        ..Default::default()
    };

    let scanner = Scanner::new(config);
    let (results, stats) = scanner.scan().await.unwrap();

    // Should find 1 file (ABP-123.mp4, SSIS-001 skipped due to recent NFO)
    assert_eq!(results.len(), 1);
    assert_eq!(stats.skip_nfo_days, 1);

    // Verify only ABP-123 is in results
    assert!(results[0].ends_with("ABP-123.mp4"));
}

#[tokio::test]
async fn test_trailer_filter() {
    let temp_dir = TempDir::new().unwrap();
    let base = temp_dir.path();

    File::create(base.join("SSIS-001.mp4")).unwrap();
    File::create(base.join("SSIS-001-trailer.mp4")).unwrap();
    File::create(base.join("ABP-123-TRAILER.mp4")).unwrap();

    let config = ScannerConfig {
        source_folder: base.to_path_buf(),
        ..Default::default()
    };

    let scanner = Scanner::new(config);
    let (results, _stats) = scanner.scan().await.unwrap();

    // Should find only 1 file (trailers excluded)
    assert_eq!(results.len(), 1);
    assert!(results[0].ends_with("SSIS-001.mp4"));
}

#[tokio::test]
async fn test_cli_regex_filter() {
    let temp_dir = TempDir::new().unwrap();
    let base = temp_dir.path();

    File::create(base.join("SSIS-001.mp4")).unwrap();
    File::create(base.join("ABP-123.mp4")).unwrap();
    File::create(base.join("STARS-456.mp4")).unwrap();

    let config = ScannerConfig {
        source_folder: base.to_path_buf(),
        cli_regex: Some(r"SSIS-\d+".to_string()),
        ..Default::default()
    };

    let scanner = Scanner::new(config);
    let (results, _stats) = scanner.scan().await.unwrap();

    // Should find only SSIS-001
    assert_eq!(results.len(), 1);
    assert!(results[0].ends_with("SSIS-001.mp4"));
}

#[tokio::test]
async fn test_complex_directory_structure() {
    let temp_dir = TempDir::new().unwrap();
    let base = temp_dir.path();

    // Create complex nested structure
    let d1 = base.join("JAV");
    let d2 = d1.join("SSIS");
    let d3 = d2.join("2024");
    fs::create_dir_all(&d3).unwrap();

    File::create(d3.join("SSIS-001.mp4")).unwrap();
    File::create(d3.join("SSIS-002.mp4")).unwrap();

    let d4 = d1.join("ABP");
    fs::create_dir(&d4).unwrap();
    File::create(d4.join("ABP-123.avi")).unwrap();

    let config = ScannerConfig {
        source_folder: base.to_path_buf(),
        ..Default::default()
    };

    let scanner = Scanner::new(config);
    let (results, _stats) = scanner.scan().await.unwrap();

    assert_eq!(results.len(), 3);
}

#[tokio::test]
async fn test_empty_directory() {
    let temp_dir = TempDir::new().unwrap();

    let config = ScannerConfig {
        source_folder: temp_dir.path().to_path_buf(),
        ..Default::default()
    };

    let scanner = Scanner::new(config);
    let (results, stats) = scanner.scan().await.unwrap();

    assert_eq!(results.len(), 0);
    assert_eq!(stats.total_files, 0);
}

#[tokio::test]
async fn test_case_insensitive_extensions() {
    let temp_dir = TempDir::new().unwrap();
    let base = temp_dir.path();

    File::create(base.join("movie1.MP4")).unwrap();
    File::create(base.join("movie2.Mp4")).unwrap();
    File::create(base.join("movie3.AVI")).unwrap();

    let config = ScannerConfig {
        source_folder: base.to_path_buf(),
        ..Default::default()
    };

    let scanner = Scanner::new(config);
    let (results, _stats) = scanner.scan().await.unwrap();

    // All should be found regardless of case
    assert_eq!(results.len(), 3);
}
