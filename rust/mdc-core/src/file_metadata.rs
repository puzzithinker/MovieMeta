//! File metadata capture utilities for logging
//!
//! This module provides utilities to capture file metadata before deletion operations,
//! enabling comprehensive audit trails for file operations.

use std::path::Path;
use std::fs;
use chrono::{DateTime, Local};

/// File metadata snapshot for logging purposes
///
/// Captures essential metadata about a file before it's deleted, including:
/// - File path
/// - Size in bytes
/// - Last modification time
/// - Whether the file exists
#[derive(Debug, Clone)]
pub struct FileSnapshot {
    /// Full path to the file
    pub path: String,

    /// File size in bytes
    pub size_bytes: u64,

    /// Last modification time as formatted string
    pub modified: String,

    /// Whether the file exists
    pub exists: bool,
}

impl FileSnapshot {
    /// Capture file metadata before deletion
    ///
    /// This method safely captures metadata even if the file doesn't exist
    /// or if there are permission errors.
    ///
    /// # Arguments
    /// * `path` - Path to the file to capture metadata from
    ///
    /// # Returns
    /// FileSnapshot with metadata (exists=false if file doesn't exist)
    ///
    /// # Examples
    /// ```
    /// use std::path::Path;
    /// use mdc_core::file_metadata::FileSnapshot;
    ///
    /// let snapshot = FileSnapshot::capture(Path::new("/path/to/file.mp4"));
    /// println!("File: {}", snapshot.format());
    /// ```
    pub fn capture(path: &Path) -> Self {
        match fs::metadata(path) {
            Ok(metadata) => {
                let modified = metadata.modified()
                    .ok()
                    .and_then(|t| {
                        // Convert SystemTime to DateTime<Local>
                        let dt: DateTime<Local> = t.into();
                        Some(dt.format("%Y-%m-%d %H:%M:%S").to_string())
                    })
                    .unwrap_or_else(|| "unknown".to_string());

                Self {
                    path: path.display().to_string(),
                    size_bytes: metadata.len(),
                    modified,
                    exists: true,
                }
            }
            Err(_) => {
                Self {
                    path: path.display().to_string(),
                    size_bytes: 0,
                    modified: "unknown".to_string(),
                    exists: false,
                }
            }
        }
    }

    /// Format snapshot for logging with all metadata
    ///
    /// # Returns
    /// Human-readable string with path, size, and modification time
    ///
    /// # Examples
    /// ```
    /// # use std::path::Path;
    /// # use mdc_core::file_metadata::FileSnapshot;
    /// let snapshot = FileSnapshot::capture(Path::new("/path/to/file.mp4"));
    /// let formatted = snapshot.format();
    /// // Output: "/path/to/file.mp4 (size: 1.5 GB, modified: 2026-01-01 12:00:00)"
    /// ```
    pub fn format(&self) -> String {
        if self.exists {
            format!(
                "{} (size: {}, modified: {})",
                self.path,
                self.format_size(),
                self.modified
            )
        } else {
            format!("{} (not found)", self.path)
        }
    }

    /// Get human-readable file size
    ///
    /// Converts bytes to appropriate unit (B, KB, MB, GB)
    ///
    /// # Returns
    /// Formatted size string (e.g., "1.5 GB", "256.0 KB")
    pub fn format_size(&self) -> String {
        let size = self.size_bytes as f64;
        if size < 1024.0 {
            format!("{} B", size)
        } else if size < 1024.0 * 1024.0 {
            format!("{:.2} KB", size / 1024.0)
        } else if size < 1024.0 * 1024.0 * 1024.0 {
            format!("{:.2} MB", size / (1024.0 * 1024.0))
        } else {
            format!("{:.2} GB", size / (1024.0 * 1024.0 * 1024.0))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use tempfile::TempDir;

    #[test]
    fn test_capture_existing_file() {
        let temp = TempDir::new().unwrap();
        let file_path = temp.path().join("test.txt");
        let mut file = File::create(&file_path).unwrap();
        file.write_all(b"test content").unwrap();
        drop(file);

        let snapshot = FileSnapshot::capture(&file_path);

        assert!(snapshot.exists);
        assert_eq!(snapshot.size_bytes, 12);
        assert!(snapshot.path.contains("test.txt"));
        assert_ne!(snapshot.modified, "unknown");
    }

    #[test]
    fn test_capture_nonexistent_file() {
        let snapshot = FileSnapshot::capture(Path::new("/nonexistent/file.txt"));

        assert!(!snapshot.exists);
        assert_eq!(snapshot.size_bytes, 0);
        assert_eq!(snapshot.modified, "unknown");
        assert!(snapshot.path.contains("/nonexistent/file.txt"));
    }

    #[test]
    fn test_format_size_bytes() {
        let snapshot = FileSnapshot {
            path: "test.txt".to_string(),
            size_bytes: 512,
            modified: "2026-01-01 12:00:00".to_string(),
            exists: true,
        };

        assert_eq!(snapshot.format_size(), "512 B");
    }

    #[test]
    fn test_format_size_kilobytes() {
        let snapshot = FileSnapshot {
            path: "test.txt".to_string(),
            size_bytes: 1024 * 10, // 10 KB
            modified: "2026-01-01 12:00:00".to_string(),
            exists: true,
        };

        assert_eq!(snapshot.format_size(), "10.00 KB");
    }

    #[test]
    fn test_format_size_megabytes() {
        let snapshot = FileSnapshot {
            path: "test.txt".to_string(),
            size_bytes: 1024 * 1024 * 5, // 5 MB
            modified: "2026-01-01 12:00:00".to_string(),
            exists: true,
        };

        let formatted = snapshot.format();
        assert!(formatted.contains("5.00 MB"));
    }

    #[test]
    fn test_format_size_gigabytes() {
        let snapshot = FileSnapshot {
            path: "movie.mp4".to_string(),
            size_bytes: 1024 * 1024 * 1024 * 2, // 2 GB
            modified: "2026-01-01 12:00:00".to_string(),
            exists: true,
        };

        assert_eq!(snapshot.format_size(), "2.00 GB");
    }

    #[test]
    fn test_format_existing_file() {
        let snapshot = FileSnapshot {
            path: "/path/to/test.mp4".to_string(),
            size_bytes: 1024 * 1024 * 100, // 100 MB
            modified: "2026-01-01 12:00:00".to_string(),
            exists: true,
        };

        let formatted = snapshot.format();
        assert!(formatted.contains("/path/to/test.mp4"));
        assert!(formatted.contains("100.00 MB"));
        assert!(formatted.contains("2026-01-01 12:00:00"));
    }

    #[test]
    fn test_format_nonexistent_file() {
        let snapshot = FileSnapshot {
            path: "/nonexistent.mp4".to_string(),
            size_bytes: 0,
            modified: "unknown".to_string(),
            exists: false,
        };

        let formatted = snapshot.format();
        assert_eq!(formatted, "/nonexistent.mp4 (not found)");
    }
}
