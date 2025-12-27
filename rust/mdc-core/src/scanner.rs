//! File scanner for discovering movie files
//!
//! This module provides async file scanning functionality to discover video files
//! in a directory tree with various filtering options.

use anyhow::{anyhow, Result};
use regex::Regex;
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::time::SystemTime;
use tokio::fs;
use walkdir::WalkDir;

/// Scanner configuration
#[derive(Debug, Clone)]
pub struct ScannerConfig {
    /// Source folder to scan
    pub source_folder: PathBuf,

    /// Media file extensions (e.g., ".mp4", ".avi", ".mkv")
    pub media_types: Vec<String>,

    /// Main mode (1=scraping, 2=organizing, 3=analysis)
    pub main_mode: i32,

    /// Link mode (0=move, 1=soft link, 2=hard link)
    pub link_mode: i32,

    /// Skip files whose NFO was modified within N days
    pub nfo_skip_days: i32,

    /// Path to failed_list.txt
    pub failed_list_path: Option<PathBuf>,

    /// Ignore failed list
    pub ignore_failed_list: bool,

    /// Success folder (for link mode NFO checking)
    pub success_folder: Option<PathBuf>,

    /// Folders to escape/skip
    pub escape_folders: Vec<String>,

    /// Scan hardlinks in non-mode-3
    pub scan_hardlink: bool,

    /// Optional regex filter from CLI
    pub cli_regex: Option<String>,

    /// Debug mode
    pub debug: bool,
}

impl Default for ScannerConfig {
    fn default() -> Self {
        Self {
            source_folder: PathBuf::from("."),
            media_types: vec![
                ".mp4".to_string(),
                ".avi".to_string(),
                ".rmvb".to_string(),
                ".wmv".to_string(),
                ".mov".to_string(),
                ".mkv".to_string(),
                ".flv".to_string(),
                ".ts".to_string(),
                ".webm".to_string(),
                ".iso".to_string(),
                ".mpg".to_string(),
                ".m4v".to_string(),
            ],
            main_mode: 1,
            link_mode: 0,
            nfo_skip_days: 0,
            failed_list_path: None,
            ignore_failed_list: false,
            success_folder: None,
            escape_folders: Vec::new(),
            scan_hardlink: false,
            cli_regex: None,
            debug: false,
        }
    }
}

/// Scan result statistics
#[derive(Debug, Default, Clone)]
pub struct ScanStats {
    /// Total files found
    pub total_files: usize,

    /// Files skipped due to failed list
    pub skip_failed: usize,

    /// Files skipped due to NFO modification date
    pub skip_nfo_days: usize,

    /// Files skipped due to success folder NFO check
    pub skip_success_nfo: usize,
}

/// Calculate days since file modification
fn file_modification_days(path: &Path) -> Result<i32> {
    let metadata = std::fs::metadata(path)?;
    let mtime = metadata.modified()?;
    let now = SystemTime::now();

    let duration = now.duration_since(mtime)
        .unwrap_or_else(|_| std::time::Duration::from_secs(0));

    let days = (duration.as_secs() / (24 * 60 * 60)) as i32;

    if days < 0 {
        Ok(9999)
    } else {
        Ok(days)
    }
}

/// Load failed file list from file
async fn load_failed_list(path: &Path) -> Result<HashSet<String>> {
    if !path.exists() {
        return Ok(HashSet::new());
    }

    let content = fs::read_to_string(path).await?;
    let failed_set: HashSet<String> = content
        .lines()
        .map(|line| line.trim().to_string())
        .filter(|line| !line.is_empty())
        .collect();

    Ok(failed_set)
}

/// Check if path is a symlink
#[cfg(unix)]
fn is_symlink(path: &Path) -> bool {
    path.is_symlink()
}

#[cfg(windows)]
fn is_symlink(path: &Path) -> bool {
    // Windows symlink detection
    match path.symlink_metadata() {
        Ok(metadata) => metadata.file_type().is_symlink(),
        Err(_) => false,
    }
}

/// Check if path has multiple hard links
fn has_hardlinks(path: &Path) -> bool {
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        if let Ok(metadata) = std::fs::metadata(path) {
            metadata.nlink() > 1
        } else {
            false
        }
    }

    #[cfg(windows)]
    {
        // Windows doesn't easily expose nlink count
        false
    }
}

/// File scanner
pub struct Scanner {
    config: ScannerConfig,
}

impl Scanner {
    /// Create a new scanner with the given configuration
    pub fn new(config: ScannerConfig) -> Self {
        Self { config }
    }

    /// Scan for movie files
    pub async fn scan(&self) -> Result<(Vec<PathBuf>, ScanStats)> {
        let mut stats = ScanStats::default();
        let mut results = Vec::new();

        // Validate source folder
        if !self.config.source_folder.is_dir() {
            return Err(anyhow!("Source folder not found: {:?}", self.config.source_folder));
        }

        // Load failed list if enabled
        let failed_set = if let Some(ref failed_path) = self.config.failed_list_path {
            if (self.config.main_mode == 3 || self.config.link_mode > 0)
                && !self.config.ignore_failed_list
            {
                load_failed_list(failed_path).await.unwrap_or_default()
            } else {
                HashSet::new()
            }
        } else {
            HashSet::new()
        };

        // Compile CLI regex if provided
        let cli_regex = if let Some(ref regex_str) = self.config.cli_regex {
            Regex::new(regex_str).ok()
        } else {
            None
        };

        // Trailer regex
        let trailer_regex = Regex::new(r"(?i)-trailer\.").unwrap();

        // Escape folder set
        let escape_set: HashSet<String> = self.config.escape_folders
            .iter()
            .map(|s| s.to_string())
            .collect();

        // Walk directory tree
        for entry in WalkDir::new(&self.config.source_folder)
            .follow_links(false)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();

            // Skip non-files
            if !path.is_file() {
                continue;
            }

            // Skip escape folders (except in mode 3)
            if self.config.main_mode != 3 {
                if let Some(parent) = path.parent() {
                    let parent_parts: HashSet<String> = parent
                        .components()
                        .filter_map(|c| c.as_os_str().to_str())
                        .map(|s| s.to_string())
                        .collect();

                    if !parent_parts.is_disjoint(&escape_set) {
                        continue;
                    }
                }
            }

            // Check file extension
            let ext = path.extension()
                .and_then(|e| e.to_str())
                .map(|e| format!(".{}", e.to_lowercase()))
                .unwrap_or_default();

            if !self.config.media_types.contains(&ext) {
                continue;
            }

            // Check failed list
            let path_str = path.to_string_lossy().to_string();
            if failed_set.contains(&path_str) {
                stats.skip_failed += 1;
                if self.config.debug {
                    tracing::info!("Skip failed movie: {}", path_str);
                }
                continue;
            }

            // Skip symlinks and hardlinks in non-mode-3
            let is_sym = is_symlink(path);
            if self.config.main_mode != 3 {
                if is_sym || (has_hardlinks(path) && !self.config.scan_hardlink) {
                    continue;
                }
            }

            // CLI regex filter
            if let Some(ref regex) = cli_regex {
                if !regex.is_match(&path_str) {
                    continue;
                }
            }

            // Trailer filter
            if let Some(filename) = path.file_name().and_then(|f| f.to_str()) {
                if trailer_regex.is_match(filename) {
                    continue;
                }
            }

            // NFO skip logic for mode 3
            if self.config.main_mode == 3 && self.config.nfo_skip_days > 0 {
                let nfo_path = path.with_extension("nfo");
                if nfo_path.exists() {
                    match file_modification_days(&nfo_path) {
                        Ok(days) if days <= self.config.nfo_skip_days => {
                            stats.skip_nfo_days += 1;
                            if self.config.debug {
                                tracing::info!(
                                    "Skip movie by NFO modified within {} days: {}",
                                    self.config.nfo_skip_days,
                                    path_str
                                );
                            }
                            continue;
                        }
                        _ => {}
                    }
                } else if self.config.debug {
                    tracing::info!("Metadata {}.nfo not found for '{}'",
                        path.file_stem().unwrap_or_default().to_string_lossy(),
                        path_str
                    );
                }
            }

            results.push(path.to_path_buf());
        }

        stats.total_files = results.len();

        // Additional NFO checking for link modes
        if self.config.nfo_skip_days > 0
            && self.config.link_mode > 0
            && self.config.main_mode != 3
        {
            if let Some(ref success_folder) = self.config.success_folder {
                results = self.filter_by_success_nfo(results, success_folder, &mut stats).await?;
            }
        }

        Ok((results, stats))
    }

    /// Filter results by checking success folder NFO modification dates
    async fn filter_by_success_nfo(
        &self,
        results: Vec<PathBuf>,
        success_folder: &Path,
        stats: &mut ScanStats,
    ) -> Result<Vec<PathBuf>> {
        use crate::number_parser::get_number;

        // Collect numbers from success folder NFOs modified within nfo_skip_days
        let mut skip_numbers = HashSet::new();

        for entry in WalkDir::new(success_folder)
            .follow_links(false)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();

            // Only check .nfo files
            if path.extension().and_then(|e| e.to_str()) != Some("nfo") {
                continue;
            }

            // Check modification date
            match file_modification_days(path) {
                Ok(days) if days <= self.config.nfo_skip_days => {
                    // Extract number from NFO stem
                    if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                        if let Ok(number) = get_number(stem, None) {
                            skip_numbers.insert(number.to_lowercase());
                        }
                    }
                }
                _ => {}
            }
        }

        // Filter out files whose numbers are in skip_numbers
        let mut filtered = Vec::new();
        for file_path in results {
            let should_skip = if let Some(filename) = file_path.file_name().and_then(|f| f.to_str()) {
                if let Ok(number) = get_number(filename, None) {
                    skip_numbers.contains(&number.to_lowercase())
                } else {
                    false
                }
            } else {
                false
            };

            if should_skip {
                stats.skip_success_nfo += 1;
                if self.config.debug {
                    tracing::info!(
                        "Skip file successfully processed within {} days: {}",
                        self.config.nfo_skip_days,
                        file_path.display()
                    );
                }
            } else {
                filtered.push(file_path);
            }
        }

        Ok(filtered)
    }
}

/// Convenience function for simple directory scanning
///
/// This is a simplified interface that scans a directory for video files
/// matching the given media types, without advanced features like NFO checking.
pub async fn scan_directory(path: &Path, media_types: &[&str]) -> Result<Vec<PathBuf>> {
    let media_types_owned: Vec<String> = media_types.iter().map(|s| {
        if s.starts_with('.') {
            s.to_string()
        } else {
            format!(".{}", s)
        }
    }).collect();

    let config = ScannerConfig {
        source_folder: path.to_path_buf(),
        media_types: media_types_owned,
        main_mode: 1,
        link_mode: 0,
        nfo_skip_days: 0,
        failed_list_path: None,
        ignore_failed_list: true,
        success_folder: None,
        escape_folders: vec![],
        scan_hardlink: false,
        cli_regex: None,
        debug: false,
    };

    let scanner = Scanner::new(config);
    let (files, _stats) = scanner.scan().await?;
    Ok(files)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{self, File};
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_scanner_basic() {
        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path();

        // Create test files
        File::create(temp_path.join("movie1.mp4")).unwrap();
        File::create(temp_path.join("movie2.avi")).unwrap();
        File::create(temp_path.join("movie3.mkv")).unwrap();
        File::create(temp_path.join("readme.txt")).unwrap();

        let config = ScannerConfig {
            source_folder: temp_path.to_path_buf(),
            ..Default::default()
        };

        let scanner = Scanner::new(config);
        let (results, _stats) = scanner.scan().await.unwrap();

        assert_eq!(results.len(), 3); // Only video files
    }

    #[tokio::test]
    async fn test_scanner_with_subdirs() {
        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path();

        // Create directory structure
        let subdir = temp_path.join("subdir");
        fs::create_dir(&subdir).unwrap();

        File::create(temp_path.join("movie1.mp4")).unwrap();
        File::create(subdir.join("movie2.mp4")).unwrap();

        let config = ScannerConfig {
            source_folder: temp_path.to_path_buf(),
            ..Default::default()
        };

        let scanner = Scanner::new(config);
        let (results, _stats) = scanner.scan().await.unwrap();

        assert_eq!(results.len(), 2);
    }

    #[tokio::test]
    async fn test_scanner_media_type_filter() {
        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path();

        File::create(temp_path.join("movie1.mp4")).unwrap();
        File::create(temp_path.join("movie2.avi")).unwrap();
        File::create(temp_path.join("movie3.mkv")).unwrap();

        let config = ScannerConfig {
            source_folder: temp_path.to_path_buf(),
            media_types: vec![".mp4".to_string()],
            ..Default::default()
        };

        let scanner = Scanner::new(config);
        let (results, _stats) = scanner.scan().await.unwrap();

        assert_eq!(results.len(), 1); // Only .mp4
    }

    #[test]
    fn test_file_modification_days() {
        use std::io::Write;

        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");

        let mut file = File::create(&file_path).unwrap();
        file.write_all(b"test").unwrap();
        drop(file);

        let days = file_modification_days(&file_path).unwrap();
        assert_eq!(days, 0); // Just created, should be 0 days old
    }
}
