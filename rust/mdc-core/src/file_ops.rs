//! File operations for organizing movie files

use anyhow::{Context, Result};
use std::fs;
use std::os::unix::fs as unix_fs;
use std::path::{Path, PathBuf};

use crate::processor::LinkMode;

/// Move a file to destination
pub fn move_file(src: &Path, dest: &Path) -> Result<()> {
    // Ensure destination directory exists
    if let Some(parent) = dest.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create directory: {:?}", parent))?;
    }

    // If destination exists, remove it first
    if dest.exists() {
        fs::remove_file(dest)
            .with_context(|| format!("Failed to remove existing file: {:?}", dest))?;
    }

    // Try to rename first (fastest if on same filesystem)
    if let Err(_) = fs::rename(src, dest) {
        // If rename fails (cross-filesystem), copy and delete
        fs::copy(src, dest)
            .with_context(|| format!("Failed to copy {} to {}", src.display(), dest.display()))?;
        fs::remove_file(src)
            .with_context(|| format!("Failed to remove source file: {}", src.display()))?;
    }

    tracing::info!("Moved file: {} -> {}", src.display(), dest.display());
    Ok(())
}

/// Copy a file to destination
pub fn copy_file(src: &Path, dest: &Path) -> Result<()> {
    // Ensure destination directory exists
    if let Some(parent) = dest.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create directory: {:?}", parent))?;
    }

    fs::copy(src, dest)
        .with_context(|| format!("Failed to copy {} to {}", src.display(), dest.display()))?;

    tracing::info!("Copied file: {} -> {}", src.display(), dest.display());
    Ok(())
}

/// Create a symbolic link
pub fn create_soft_link(src: &Path, dest: &Path) -> Result<()> {
    // Ensure destination directory exists
    if let Some(parent) = dest.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create directory: {:?}", parent))?;
    }

    // Remove existing link if present
    if dest.exists() || dest.symlink_metadata().is_ok() {
        fs::remove_file(dest)
            .with_context(|| format!("Failed to remove existing link: {:?}", dest))?;
    }

    unix_fs::symlink(src, dest)
        .with_context(|| format!("Failed to create symlink: {} -> {}", dest.display(), src.display()))?;

    tracing::info!("Created soft link: {} -> {}", dest.display(), src.display());
    Ok(())
}

/// Create a hard link (falls back to soft link on error)
pub fn create_hard_link(src: &Path, dest: &Path) -> Result<()> {
    // Ensure destination directory exists
    if let Some(parent) = dest.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create directory: {:?}", parent))?;
    }

    // Remove existing link if present
    if dest.exists() {
        fs::remove_file(dest)
            .with_context(|| format!("Failed to remove existing file: {:?}", dest))?;
    }

    // Try hard link first
    match fs::hard_link(src, dest) {
        Ok(_) => {
            tracing::info!("Created hard link: {} -> {}", dest.display(), src.display());
            Ok(())
        }
        Err(e) => {
            tracing::warn!("Hard link failed ({}), falling back to soft link", e);
            create_soft_link(src, dest)
        }
    }
}

/// Execute file operation based on link mode
pub fn execute_file_operation(src: &Path, dest: &Path, link_mode: LinkMode) -> Result<()> {
    match link_mode {
        LinkMode::Move => move_file(src, dest),
        LinkMode::SoftLink => create_soft_link(src, dest),
        LinkMode::HardLink => create_hard_link(src, dest),
    }
}

/// Find and move subtitle files associated with a movie
pub fn move_subtitles(
    movie_path: &Path,
    dest_dir: &Path,
    dest_filename_base: &str,
    link_mode: LinkMode,
) -> Result<Vec<PathBuf>> {
    let subtitle_exts = [
        "smi", "srt", "idx", "sub", "sup", "psb", "ssa", "ass", "usf", "xss", "ssf", "rt", "lrc",
        "sbv", "vtt", "ttml",
    ];

    let mut moved_subs = Vec::new();

    if let Some(parent) = movie_path.parent() {
        let movie_stem = movie_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("");

        // Look for subtitle files with same base name
        if let Ok(entries) = fs::read_dir(parent) {
            for entry in entries.flatten() {
                let path = entry.path();
                if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
                    if subtitle_exts.contains(&ext.to_lowercase().as_str()) {
                        if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                            // Check if subtitle belongs to this movie
                            if stem.starts_with(movie_stem) {
                                let dest_sub = dest_dir.join(format!("{}.{}", dest_filename_base, ext));
                                if let Ok(_) = execute_file_operation(&path, &dest_sub, link_mode) {
                                    moved_subs.push(dest_sub);
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(moved_subs)
}

/// Get video file extension from path
pub fn get_video_extension(path: &Path) -> String {
    path.extension()
        .and_then(|s| s.to_str())
        .map(|s| format!(".{}", s))
        .unwrap_or_else(|| ".mp4".to_string())
}

/// Sanitize filename for filesystem compatibility
pub fn sanitize_filename(name: &str) -> String {
    let invalid_chars = ['<', '>', ':', '"', '/', '\\', '|', '?', '*'];
    let mut result = name.to_string();

    for ch in invalid_chars {
        result = result.replace(ch, "_");
    }

    // Trim whitespace and dots from ends
    result = result.trim().trim_end_matches('.').to_string();

    // Limit length to avoid filesystem issues
    if result.len() > 200 {
        result.truncate(200);
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use tempfile::TempDir;

    #[test]
    fn test_move_file() {
        let temp = TempDir::new().unwrap();
        let src = temp.path().join("source.txt");
        let dest = temp.path().join("subdir/dest.txt");

        File::create(&src).unwrap();
        std::fs::write(&src, "test content").unwrap();

        move_file(&src, &dest).unwrap();

        assert!(!src.exists());
        assert!(dest.exists());
        assert_eq!(std::fs::read_to_string(&dest).unwrap(), "test content");
    }

    #[test]
    fn test_copy_file() {
        let temp = TempDir::new().unwrap();
        let src = temp.path().join("source.txt");
        let dest = temp.path().join("dest.txt");

        File::create(&src).unwrap();
        std::fs::write(&src, "test content").unwrap();

        copy_file(&src, &dest).unwrap();

        assert!(src.exists());
        assert!(dest.exists());
        assert_eq!(std::fs::read_to_string(&dest).unwrap(), "test content");
    }

    #[test]
    fn test_create_soft_link() {
        let temp = TempDir::new().unwrap();
        let src = temp.path().join("source.txt");
        let link = temp.path().join("link.txt");

        File::create(&src).unwrap();
        std::fs::write(&src, "test content").unwrap();

        create_soft_link(&src, &link).unwrap();

        assert!(link.exists());
        assert_eq!(std::fs::read_to_string(&link).unwrap(), "test content");

        // Verify it's actually a symlink
        assert!(link.symlink_metadata().unwrap().file_type().is_symlink());
    }

    #[test]
    fn test_create_hard_link() {
        let temp = TempDir::new().unwrap();
        let src = temp.path().join("source.txt");
        let link = temp.path().join("link.txt");

        File::create(&src).unwrap();
        std::fs::write(&src, "test content").unwrap();

        create_hard_link(&src, &link).unwrap();

        assert!(link.exists());
        assert_eq!(std::fs::read_to_string(&link).unwrap(), "test content");
    }

    #[test]
    fn test_get_video_extension() {
        assert_eq!(get_video_extension(Path::new("movie.mp4")), ".mp4");
        assert_eq!(get_video_extension(Path::new("movie.mkv")), ".mkv");
        assert_eq!(get_video_extension(Path::new("movie")), ".mp4"); // default
    }

    #[test]
    fn test_sanitize_filename() {
        assert_eq!(sanitize_filename("Test Movie"), "Test Movie");
        assert_eq!(sanitize_filename("Test: Movie?"), "Test_ Movie_");
        assert_eq!(sanitize_filename("Test<>Movie"), "Test__Movie");
        assert_eq!(sanitize_filename("  Test Movie  "), "Test Movie");
        assert_eq!(sanitize_filename("Test Movie..."), "Test Movie");

        // Test length limiting
        let long_name = "a".repeat(250);
        let sanitized = sanitize_filename(&long_name);
        assert!(sanitized.len() <= 200);
    }

    #[test]
    fn test_move_subtitles() {
        let temp = TempDir::new().unwrap();
        let movie_path = temp.path().join("movie.mp4");
        let subtitle1 = temp.path().join("movie.srt");
        let subtitle2 = temp.path().join("movie.en.srt");
        let dest_dir = temp.path().join("output");

        File::create(&movie_path).unwrap();
        File::create(&subtitle1).unwrap();
        File::create(&subtitle2).unwrap();

        let moved = move_subtitles(&movie_path, &dest_dir, "new_movie", LinkMode::Move).unwrap();

        assert_eq!(moved.len(), 2);
        assert!(!subtitle1.exists()); // Original should be moved
        assert!(dest_dir.join("new_movie.srt").exists());
    }
}
