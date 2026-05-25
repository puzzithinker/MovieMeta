//! Core processing engine for movie metadata workflow

use std::path::Path;

/// Main processing mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProcessingMode {
    /// Mode 1: Full scraping - download metadata, images, organize files
    Scraping = 1,
    /// Mode 2: Organizing only - move files without downloading metadata
    Organizing = 2,
    /// Mode 3: Analysis - scrape in place without moving files
    Analysis = 3,
}

/// Link mode for file operations
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LinkMode {
    /// Move files (default)
    Move = 0,
    /// Create soft links (symlinks)
    SoftLink = 1,
    /// Create hard links (fallback to soft link if impossible)
    HardLink = 2,
}

/// File attributes detected from filename/path
#[derive(Debug, Clone, Default)]
pub struct FileAttributes {
    /// Multi-part file (CD1, CD2, etc.)
    pub multi_part: bool,
    /// Part suffix (e.g., "-CD1")
    pub part: String,
    /// Chinese subtitle flag
    pub cn_sub: bool,
    /// Uncensored flag
    pub uncensored: bool,
    /// Uncensored + Chinese subtitle
    pub uncensored_cn: bool,
    /// 4K video
    pub is_4k: bool,
    /// ISO format
    pub is_iso: bool,
}

impl FileAttributes {
    /// Detect attributes from file path
    pub fn from_path(path: &Path) -> Self {
        let path_str = path.to_string_lossy();
        let mut attrs = FileAttributes::default();

        // Multi-part detection with comprehensive pattern support
        // Priority order: explicit markers first, then numeric suffixes
        //
        // For numeric patterns, we need to be careful not to match the movie number itself.
        // Movie numbers typically follow patterns like: ABC-123, SSIS-456, T28-001
        // Multi-part suffixes come AFTER: ABC-123-2, SSIS-456-CD1, etc.
        //
        // The numeric pattern requires that there's already a complete ID before the number,
        // by looking for: letters + dash/number, then another dash/underscore + just digits + extension
        let patterns = [
            (r"[-_](CD\d+)", "CD"),              // -CD1, -CD2, _CD1
            (r"(?i)[-_](DISK\d+)", "DISK"),      // -disk1, -disk2, _DISK3
            (r"(?i)[-_](DISC\d+)", "DISC"),      // -disc1, -disc2, _DISC3
            (r"(?i)[-_](PART\d+)", "PART"),      // -part1, -part2, _PART3
            (r"(?i)[-_](PARTS\d+)", "PARTS"),    // -parts1 (less common)
            (r"(?i)[-_](PT\d+)", "PT"),          // -pt1, -pt2, _PT3
            // Numeric pattern: requires previous dash with digits (movie ID), then another dash + 1-2 digits
            // This prevents matching the movie number itself
            (r"-\d+[-_](\d{1,2})(?:[-_][CU])?\.[\w]+$", "NUM"),
        ];

        for (pattern, _label) in patterns {
            if let Some(caps) = regex::Regex::new(pattern)
                .ok()
                .and_then(|re| re.captures(&path_str))
            {
                let captured = match caps.get(1) {
                    Some(m) => m.as_str().to_uppercase(),
                    None => continue,
                };
                attrs.multi_part = true;
                attrs.part = format!("-{}", captured);
                break;  // Use first match only
            }
        }

        // Chinese subtitle detection
        let cn_patterns = ["-C.", "_C.", "ch.", "中文", "字幕"];
        attrs.cn_sub = cn_patterns
            .iter()
            .any(|p| path_str.to_lowercase().contains(&p.to_lowercase()));

        // Uncensored detection
        if path_str.to_uppercase().contains("-U.") || path_str.to_uppercase().contains("_U.") {
            attrs.uncensored = true;
        }

        if path_str.to_uppercase().contains("-UC.") || path_str.to_uppercase().contains("_UC.") {
            attrs.uncensored = true;
            attrs.cn_sub = true;
            attrs.uncensored_cn = true;
        }

        // 4K detection
        attrs.is_4k = path_str.to_uppercase().contains("4K");

        // ISO detection
        attrs.is_iso = path_str.to_lowercase().ends_with(".iso");

        attrs
    }

    /// Get suffix for filenames
    pub fn get_suffix(&self) -> String {
        let mut suffix = String::new();

        if self.uncensored_cn {
            suffix.push_str("-UC");
        } else if self.uncensored {
            suffix.push_str("-U");
        } else if self.cn_sub {
            suffix.push_str("-C");
        }

        suffix
    }
}

/// Naming/location rule template
#[derive(Debug, Clone)]
pub struct Template {
    template: String,
}

impl Template {
    /// Create a new template
    pub fn new(template: impl Into<String>) -> Self {
        Self {
            template: template.into(),
        }
    }

    /// Render the template with metadata
    ///
    /// Supports variables: number, title, actor, studio, director, series, year, label
    pub fn render(&self, metadata: &serde_json::Value) -> String {
        let mut result = self.template.clone();

        // Helper to get field value
        let get_field = |key: &str| -> String {
            metadata
                .get(key)
                .and_then(|v| match v {
                    serde_json::Value::String(s) => Some(s.clone()),
                    serde_json::Value::Array(arr) if !arr.is_empty() => {
                        // For arrays like actors, take first element
                        arr.first().and_then(|v| v.as_str()).map(|s| s.to_string())
                    }
                    _ => None,
                })
                .unwrap_or_default()
        };

        // Replace template variables
        // Support both direct variable names and expressions like actor + "/" + number
        let variables = [
            "number", "title", "actor", "studio", "director", "series", "year", "label",
        ];

        for var in &variables {
            let placeholder = format!("{}", var);
            let value = get_field(var);
            result = result.replace(&placeholder, &value);
        }

        // Clean up operators and whitespace
        result = result
            .replace(" + ", "")
            .replace("+", "")
            .replace("\"", "")
            .replace("'", "")
            .trim()
            .to_string();

        // Handle empty actor case (common in Python code)
        if result.is_empty() || result == "/" {
            result = get_field("number");
        }

        result
    }
}

/// Processing statistics
#[derive(Debug, Clone, Default)]
pub struct ProcessingStats {
    /// Total files processed
    pub total_processed: usize,
    /// Successfully completed
    pub succeeded: usize,
    /// Failed
    pub failed: usize,
    /// Skipped (already processed, in failed list, etc.)
    pub skipped: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_attributes_multi_part_cd() {
        // CD pattern detection (original behavior)
        let path = Path::new("/movies/TEST-001-CD1.mp4");
        let attrs = FileAttributes::from_path(path);
        assert!(attrs.multi_part);
        assert_eq!(attrs.part, "-CD1");

        let path2 = Path::new("/movies/TEST-001_CD2.mp4");
        let attrs2 = FileAttributes::from_path(path2);
        assert!(attrs2.multi_part);
        assert_eq!(attrs2.part, "-CD2");
    }

    #[test]
    fn test_file_attributes_multi_part_numeric() {
        // User's specific case - numeric suffixes
        let path = Path::new("/movies/migd-534/[456k.me]migd-534-2.mp4");
        let attrs = FileAttributes::from_path(path);
        assert!(attrs.multi_part, "Failed to detect -2 as multi-part");
        assert_eq!(attrs.part, "-2");

        // Other numeric patterns
        let path2 = Path::new("/movies/ABC-123-1.mp4");
        let attrs2 = FileAttributes::from_path(path2);
        assert!(attrs2.multi_part);
        assert_eq!(attrs2.part, "-1");

        let path3 = Path::new("/movies/XYZ-456_3.mkv");
        let attrs3 = FileAttributes::from_path(path3);
        assert!(attrs3.multi_part);
        assert_eq!(attrs3.part, "-3");
    }

    #[test]
    fn test_file_attributes_multi_part_disc() {
        let path = Path::new("/movies/ABC-123-disc1.mp4");
        let attrs = FileAttributes::from_path(path);
        assert!(attrs.multi_part);
        assert_eq!(attrs.part, "-DISC1");

        let path2 = Path::new("/movies/XYZ-456_disk2.mkv");
        let attrs2 = FileAttributes::from_path(path2);
        assert!(attrs2.multi_part);
        assert_eq!(attrs2.part, "-DISK2");
    }

    #[test]
    fn test_file_attributes_multi_part_part() {
        let path = Path::new("/movies/ABC-123-part1.mp4");
        let attrs = FileAttributes::from_path(path);
        assert!(attrs.multi_part);
        assert_eq!(attrs.part, "-PART1");

        let path2 = Path::new("/movies/XYZ-456_pt2.mkv");
        let attrs2 = FileAttributes::from_path(path2);
        assert!(attrs2.multi_part);
        assert_eq!(attrs2.part, "-PT2");
    }

    #[test]
    fn test_file_attributes_not_multi_part() {
        // Movie numbers that end in digits - should NOT detect as multi-part
        let cases = vec![
            "/movies/SSIS-123.mp4",          // Primary number
            "/movies/T28-001.mp4",           // T28 format
            "/movies/FC2-PPV-1234567.mp4",   // FC2 format
        ];

        for path_str in cases {
            let path = Path::new(path_str);
            let attrs = FileAttributes::from_path(path);
            assert!(!attrs.multi_part, "Incorrectly detected {} as multi-part", path_str);
            assert_eq!(attrs.part, "", "Part should be empty for {}", path_str);
        }
    }

    #[test]
    fn test_file_attributes_multi_part_with_attributes() {
        // Multi-part + Chinese subtitle
        let path = Path::new("/movies/MIGD-534-2-C.mp4");
        let attrs = FileAttributes::from_path(path);
        assert!(attrs.multi_part);
        assert_eq!(attrs.part, "-2");
        assert!(attrs.cn_sub);

        // Multi-part + Uncensored
        let path2 = Path::new("/movies/ABC-123-CD1-U.mp4");
        let attrs2 = FileAttributes::from_path(path2);
        assert!(attrs2.multi_part);
        assert_eq!(attrs2.part, "-CD1");
        assert!(attrs2.uncensored);
    }

    #[test]
    fn test_file_attributes_multi_part_priority() {
        // Test that CD pattern takes priority over numeric
        let path = Path::new("/movies/ABC-123-CD1-2.mp4");
        let attrs = FileAttributes::from_path(path);
        assert!(attrs.multi_part);
        // Should match CD1, not -2
        assert_eq!(attrs.part, "-CD1");
    }

    #[test]
    fn test_file_attributes_chinese_sub() {
        let path = Path::new("/movies/TEST-001-C.mp4");
        let attrs = FileAttributes::from_path(path);

        assert!(attrs.cn_sub);
        assert!(!attrs.uncensored);
    }

    #[test]
    fn test_file_attributes_uncensored() {
        let path = Path::new("/movies/TEST-001-U.mp4");
        let attrs = FileAttributes::from_path(path);

        assert!(attrs.uncensored);
        assert!(!attrs.cn_sub);
    }

    #[test]
    fn test_file_attributes_uncensored_chinese() {
        let path = Path::new("/movies/TEST-001-UC.mp4");
        let attrs = FileAttributes::from_path(path);

        assert!(attrs.uncensored);
        assert!(attrs.cn_sub);
        assert!(attrs.uncensored_cn);
    }

    #[test]
    fn test_file_attributes_4k() {
        let path = Path::new("/movies/TEST-001-4K.mp4");
        let attrs = FileAttributes::from_path(path);

        assert!(attrs.is_4k);
    }

    #[test]
    fn test_file_attributes_suffix() {
        let path = Path::new("/movies/TEST-001-C.mp4");
        let attrs = FileAttributes::from_path(path);
        assert_eq!(attrs.get_suffix(), "-C");

        let path2 = Path::new("/movies/TEST-001-U.mp4");
        let attrs2 = FileAttributes::from_path(path2);
        assert_eq!(attrs2.get_suffix(), "-U");

        let path3 = Path::new("/movies/TEST-001-UC.mp4");
        let attrs3 = FileAttributes::from_path(path3);
        assert_eq!(attrs3.get_suffix(), "-UC");
    }

    #[test]
    fn test_template_simple() {
        let template = Template::new("number");
        let metadata = serde_json::json!({
            "number": "TEST-001",
            "title": "Test Movie"
        });

        let result = template.render(&metadata);
        assert_eq!(result, "TEST-001");
    }

    #[test]
    fn test_template_with_actor() {
        let template = Template::new("actor/number");
        let metadata = serde_json::json!({
            "number": "TEST-001",
            "actor": ["John Doe", "Jane Smith"]
        });

        let result = template.render(&metadata);
        assert_eq!(result, "John Doe/TEST-001");
    }

    #[test]
    fn test_template_empty_actor() {
        let template = Template::new("actor");
        let metadata = serde_json::json!({
            "number": "TEST-001",
            "actor": []
        });

        let result = template.render(&metadata);
        // Empty actor should fall back to number
        assert_eq!(result, "TEST-001");
    }

    #[test]
    fn test_template_complex() {
        let template = Template::new("studio/number");
        let metadata = serde_json::json!({
            "number": "TEST-001",
            "studio": "Test Studio",
            "title": "Test Movie"
        });

        let result = template.render(&metadata);
        assert_eq!(result, "Test Studio/TEST-001");
    }
}
