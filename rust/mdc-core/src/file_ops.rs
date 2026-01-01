//! File operations for organizing movie files
//!
//! Provides cross-platform file operations including move, copy, and linking.
//!
//! # Platform Considerations
//!
//! - **Hard links**: Fully supported on all platforms via `std::fs::hard_link`
//! - **Soft links (symlinks)**:
//!   - Unix/Linux/macOS: Works without special privileges
//!   - Windows: Requires administrator privileges or Developer Mode (Windows 10+)
//!   - Recommendation: Use hard links (--link-mode 2) on Windows for better compatibility
//! - **Move operations**: Fully cross-platform with automatic fallback to copy+delete

use anyhow::{Context, Result};
use std::fs;

#[cfg(unix)]
use std::os::unix::fs as unix_fs;

#[cfg(windows)]
use std::os::windows::fs as windows_fs;

use std::path::{Path, PathBuf};

use crate::file_metadata::FileSnapshot;
use crate::processor::LinkMode;

/// Check if two files are identical (same size and modification time)
fn files_are_identical(a: &Path, b: &Path) -> Result<bool> {
    let meta_a = fs::metadata(a)
        .with_context(|| format!("Failed to read metadata for: {}", a.display()))?;
    let meta_b = fs::metadata(b)
        .with_context(|| format!("Failed to read metadata for: {}", b.display()))?;

    // Compare file size
    if meta_a.len() != meta_b.len() {
        return Ok(false);
    }

    // Compare modification time
    let Ok(mtime_a) = meta_a.modified() else {
        return Ok(false);
    };
    let Ok(mtime_b) = meta_b.modified() else {
        return Ok(false);
    };

    // Files are considered identical if they have the same size and modification time
    Ok(mtime_a == mtime_b)
}

/// Get human-readable file size
fn get_file_size(path: &Path) -> String {
    if let Ok(metadata) = fs::metadata(path) {
        let size = metadata.len() as f64;
        if size < 1024.0 {
            format!("{} B", size)
        } else if size < 1024.0 * 1024.0 {
            format!("{:.2} KB", size / 1024.0)
        } else if size < 1024.0 * 1024.0 * 1024.0 {
            format!("{:.2} MB", size / (1024.0 * 1024.0))
        } else {
            format!("{:.2} GB", size / (1024.0 * 1024.0 * 1024.0))
        }
    } else {
        "unknown".to_string()
    }
}

/// Move a file to destination
///
/// # Safety
///
/// This function performs the following safety checks:
/// 1. Verifies source file exists before any operations
/// 2. Detects if source and destination are the same file (already moved)
/// 3. Detects if destination contains identical content
/// 4. Only deletes destination after source is verified
///
/// This prevents data loss when re-running on already-processed files.
pub fn move_file(src: &Path, dest: &Path) -> Result<()> {
    // 1. CRITICAL SAFETY CHECK: Verify source exists FIRST
    if !src.exists() {
        anyhow::bail!(
            "Source file does not exist: {}\nWill not proceed with move operation to avoid data loss",
            src.display()
        );
    }

    // Ensure destination directory exists
    if let Some(parent) = dest.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create directory: {:?}", parent))?;
    }

    // 2. Check if source and destination are the same file (already moved)
    if let (Ok(src_canonical), Ok(dest_canonical)) = (src.canonicalize(), dest.canonicalize()) {
        if src_canonical == dest_canonical {
            tracing::info!(
                "Source and destination are the same file (already moved): {}",
                dest.display()
            );
            return Ok(());
        }
    }

    // 3. Check if destination exists
    if dest.exists() {
        // 3a. Check if files are identical (same size and modification time)
        if files_are_identical(src, dest)? {
            tracing::info!(
                "Destination already contains identical file ({}), removing source: {}",
                get_file_size(dest),
                src.display()
            );
            fs::remove_file(src)
                .with_context(|| format!("Failed to remove source file: {}", src.display()))?;
            tracing::info!("Removed duplicate source file: {}", src.display());
            return Ok(());
        }

        // 3b. Different files - warn and delete destination (source verified, safe to proceed)
        let snapshot = FileSnapshot::capture(dest);
        tracing::warn!(
            "Destination file exists and differs from source - replacing: {}",
            snapshot.format()
        );

        fs::remove_file(dest)
            .with_context(|| format!("Failed to remove existing file: {:?}", dest))?;

        tracing::info!("Deleted existing destination: {}", dest.display());
    }

    // Capture source metadata before move (for logging in case of cross-filesystem move)
    let src_snapshot = FileSnapshot::capture(src);

    // 4. Try to rename first (fastest if on same filesystem)
    if let Err(_) = fs::rename(src, dest) {
        // If rename fails (cross-filesystem), copy and delete
        tracing::debug!("Rename failed, falling back to copy+delete for cross-filesystem move");

        fs::copy(src, dest)
            .with_context(|| format!("Failed to copy {} to {}", src.display(), dest.display()))?;

        tracing::info!(
            "Removing source file after copy (cross-filesystem move): {}",
            src_snapshot.format()
        );

        fs::remove_file(src)
            .with_context(|| format!("Failed to remove source file: {}", src.display()))?;

        tracing::info!("Deleted source after copy: {}", src.display());
    }

    tracing::info!(
        "Moved file: {} -> {} ({})",
        src.display(),
        dest.display(),
        src_snapshot.format_size()
    );
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
///
/// # Platform-specific behavior
///
/// - **Unix/Linux/macOS**: Creates a single symlink for both files and directories
/// - **Windows**: Automatically detects file vs directory and uses appropriate function
///   - Requires administrator privileges OR Windows 10+ Developer Mode
///   - Consider using hard links (LinkMode::HardLink) or move mode (LinkMode::Move) as alternatives
///
/// # Errors
///
/// - Fails if parent directory cannot be created
/// - Fails if existing destination cannot be removed
/// - On Windows: Fails with helpful message if symlink privileges are insufficient
#[cfg(unix)]
pub fn create_soft_link(src: &Path, dest: &Path) -> Result<()> {
    // 1. CRITICAL SAFETY CHECK: Verify source exists FIRST
    if !src.exists() {
        anyhow::bail!(
            "Source file does not exist: {}\nWill not proceed with link operation to avoid data loss",
            src.display()
        );
    }

    // Ensure destination directory exists
    if let Some(parent) = dest.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create directory: {:?}", parent))?;
    }

    // 2. Check if source and destination are the same (already linked)
    if let (Ok(src_canonical), Ok(dest_canonical)) = (src.canonicalize(), dest.canonicalize()) {
        if src_canonical == dest_canonical {
            tracing::info!(
                "Source and destination are the same file (already linked): {}",
                dest.display()
            );
            return Ok(());
        }
    }

    // 3. Remove existing link if present (source verified, safe to proceed)
    if dest.exists() || dest.symlink_metadata().is_ok() {
        let snapshot = FileSnapshot::capture(dest);
        tracing::warn!(
            "Removing existing link before creating soft link: {}",
            snapshot.format()
        );

        fs::remove_file(dest)
            .with_context(|| format!("Failed to remove existing link: {:?}", dest))?;

        tracing::info!("Deleted existing link: {}", dest.display());
    }

    unix_fs::symlink(src, dest).with_context(|| {
        format!(
            "Failed to create symlink: {} -> {}",
            dest.display(),
            src.display()
        )
    })?;

    tracing::info!("Created soft link: {} -> {}", dest.display(), src.display());
    Ok(())
}

/// Create a symbolic link
///
/// # Platform-specific behavior
///
/// - **Unix/Linux/macOS**: Creates a single symlink for both files and directories
/// - **Windows**: Automatically detects file vs directory and uses appropriate function
///   - Requires administrator privileges OR Windows 10+ Developer Mode
///   - Consider using hard links (LinkMode::HardLink) or move mode (LinkMode::Move) as alternatives
///
/// # Errors
///
/// - Fails if parent directory cannot be created
/// - Fails if existing destination cannot be removed
/// - On Windows: Fails with helpful message if symlink privileges are insufficient
#[cfg(windows)]
pub fn create_soft_link(src: &Path, dest: &Path) -> Result<()> {
    // 1. CRITICAL SAFETY CHECK: Verify source exists FIRST
    if !src.exists() {
        anyhow::bail!(
            "Source file does not exist: {}\nWill not proceed with link operation to avoid data loss",
            src.display()
        );
    }

    // Ensure destination directory exists
    if let Some(parent) = dest.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create directory: {:?}", parent))?;
    }

    // 2. Check if source and destination are the same (already linked)
    if let (Ok(src_canonical), Ok(dest_canonical)) = (src.canonicalize(), dest.canonicalize()) {
        if src_canonical == dest_canonical {
            tracing::info!(
                "Source and destination are the same file (already linked): {}",
                dest.display()
            );
            return Ok(());
        }
    }

    // 3. Remove existing link if present (source verified, safe to proceed)
    if dest.exists() || dest.symlink_metadata().is_ok() {
        let snapshot = FileSnapshot::capture(dest);
        tracing::warn!(
            "Removing existing link before creating soft link: {}",
            snapshot.format()
        );

        fs::remove_file(dest)
            .with_context(|| format!("Failed to remove existing link: {:?}", dest))?;

        tracing::info!("Deleted existing link: {}", dest.display());
    }

    // Detect if source is a file or directory
    let metadata = fs::metadata(src)
        .with_context(|| format!("Failed to read source metadata: {}", src.display()))?;

    let result = if metadata.is_dir() {
        windows_fs::symlink_dir(src, dest)
    } else {
        windows_fs::symlink_file(src, dest)
    };

    result.with_context(|| {
        format!(
            "Failed to create symlink: {} -> {}. \
            On Windows, symlinks require administrator privileges or Developer Mode. \
            Consider using hard links (--link-mode 2) or move mode (--link-mode 0) instead.",
            dest.display(),
            src.display()
        )
    })?;

    tracing::info!("Created soft link: {} -> {}", dest.display(), src.display());
    Ok(())
}

/// Create a hard link (falls back to soft link on error)
pub fn create_hard_link(src: &Path, dest: &Path) -> Result<()> {
    // 1. CRITICAL SAFETY CHECK: Verify source exists FIRST
    if !src.exists() {
        anyhow::bail!(
            "Source file does not exist: {}\nWill not proceed with link operation to avoid data loss",
            src.display()
        );
    }

    // Ensure destination directory exists
    if let Some(parent) = dest.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create directory: {:?}", parent))?;
    }

    // 2. Check if source and destination are the same (already linked)
    if let (Ok(src_canonical), Ok(dest_canonical)) = (src.canonicalize(), dest.canonicalize()) {
        if src_canonical == dest_canonical {
            tracing::info!(
                "Source and destination are the same file (already linked): {}",
                dest.display()
            );
            return Ok(());
        }
    }

    // 3. Remove existing link if present (source verified, safe to proceed)
    if dest.exists() {
        let snapshot = FileSnapshot::capture(dest);
        tracing::warn!(
            "Removing existing file before creating hard link: {}",
            snapshot.format()
        );

        fs::remove_file(dest)
            .with_context(|| format!("Failed to remove existing file: {:?}", dest))?;

        tracing::info!("Deleted existing file: {}", dest.display());
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
                                let dest_sub =
                                    dest_dir.join(format!("{}.{}", dest_filename_base, ext));
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

    // Limit length to avoid filesystem issues (character-aware for UTF-8 safety)
    if result.chars().count() > 200 {
        result = result.chars().take(200).collect();
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
    #[cfg_attr(windows, ignore)] // Skip on Windows by default (requires privileges)
    fn test_create_soft_link() {
        let temp = TempDir::new().unwrap();
        let src = temp.path().join("source.txt");
        let link = temp.path().join("link.txt");

        File::create(&src).unwrap();
        std::fs::write(&src, "test content").unwrap();

        let result = create_soft_link(&src, &link);

        #[cfg(unix)]
        {
            result.unwrap();
            assert!(link.exists());
            assert_eq!(std::fs::read_to_string(&link).unwrap(), "test content");
            // Verify it's actually a symlink
            assert!(link.symlink_metadata().unwrap().file_type().is_symlink());
        }

        #[cfg(windows)]
        {
            // May succeed with privileges, or fail gracefully
            if let Ok(_) = result {
                assert!(link.exists());
                assert_eq!(std::fs::read_to_string(&link).unwrap(), "test content");
            }
        }
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

        // Test length limiting with multi-byte UTF-8 characters (Japanese)
        // Each Japanese character is 3 bytes in UTF-8
        let japanese_name = "日本語のタイトル".repeat(30); // Creates a string with 210 Japanese chars
        let sanitized = sanitize_filename(&japanese_name);
        assert!(sanitized.chars().count() <= 200);
        // Should not panic on character boundary
        assert!(!sanitized.is_empty());
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

    #[test]
    #[cfg(windows)]
    fn test_windows_symlink_privilege_error() {
        let temp = TempDir::new().unwrap();
        let src = temp.path().join("source.txt");
        let link = temp.path().join("link.txt");

        File::create(&src).unwrap();

        let result = create_soft_link(&src, &link);

        // Verify error message is helpful when it fails
        if let Err(e) = result {
            let err_msg = e.to_string();
            assert!(
                err_msg.contains("administrator privileges") || err_msg.contains("Developer Mode"),
                "Error message should explain Windows privilege requirements: {}",
                err_msg
            );
        }
    }

    #[test]
    #[cfg(windows)]
    fn test_windows_directory_symlink() {
        let temp = TempDir::new().unwrap();
        let src_dir = temp.path().join("source_dir");
        let link_dir = temp.path().join("link_dir");

        fs::create_dir(&src_dir).unwrap();
        File::create(src_dir.join("file.txt")).unwrap();

        let result = create_soft_link(&src_dir, &link_dir);

        if let Ok(_) = result {
            // Verify directory symlink works
            assert!(link_dir.exists());
            assert!(link_dir.join("file.txt").exists());
        }
    }

    #[test]
    fn test_move_file_source_does_not_exist_preserves_destination() {
        // CRITICAL SAFETY TEST: Verifies the data loss bug is fixed
        // This test simulates the exact scenario that caused the user's data loss:
        // 1. Destination file exists (from previous run)
        // 2. Source file doesn't exist (was already moved)
        // 3. Trying to move should FAIL and PRESERVE destination

        let temp = TempDir::new().unwrap();
        let nonexistent_source = temp.path().join("nonexistent.mp4");
        let existing_dest = temp.path().join("important_data.mp4");

        // Create destination file with important data
        std::fs::write(&existing_dest, "This is critical data that must not be lost").unwrap();
        assert!(existing_dest.exists());

        let original_content = std::fs::read_to_string(&existing_dest).unwrap();
        let original_size = existing_dest.metadata().unwrap().len();

        // Attempt to move non-existent source to existing destination
        let result = move_file(&nonexistent_source, &existing_dest);

        // CRITICAL: Move should fail
        assert!(result.is_err(), "Move should fail when source doesn't exist");

        // CRITICAL: Destination file must still exist with original content
        assert!(existing_dest.exists(), "Destination file must not be deleted!");
        assert_eq!(
            std::fs::read_to_string(&existing_dest).unwrap(),
            original_content,
            "Destination file content must be unchanged!"
        );
        assert_eq!(
            existing_dest.metadata().unwrap().len(),
            original_size,
            "Destination file size must be unchanged!"
        );

        // Verify error message mentions source doesn't exist
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("Source file does not exist"),
            "Error should clearly state source doesn't exist: {}", err_msg);
        assert!(err_msg.contains("data loss"),
            "Error should mention data loss prevention: {}", err_msg);
    }

    #[test]
    fn test_move_file_already_moved_same_file() {
        // Test that moving a file to itself (already moved) is detected and skipped
        let temp = TempDir::new().unwrap();
        let file_path = temp.path().join("movie.mp4");

        std::fs::write(&file_path, "movie content").unwrap();

        // Try to "move" file to itself
        let result = move_file(&file_path, &file_path);

        // Should succeed (no-op)
        assert!(result.is_ok(), "Moving file to itself should succeed as no-op");
        assert!(file_path.exists(), "File should still exist");
        assert_eq!(std::fs::read_to_string(&file_path).unwrap(), "movie content");
    }

}
