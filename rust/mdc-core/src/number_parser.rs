//! Movie number parser
//!
//! This module extracts movie numbers from filenames using various patterns and rules.
//! It's a direct port of the Python `number_parser.py` with 100% compatibility.

use anyhow::{anyhow, Result};
use regex::Regex;
use std::collections::HashMap;
use std::sync::OnceLock;

/// Pre-processing pattern to remove website prefixes, quality markers, and suffixes
static G_SPAT: OnceLock<Regex> = OnceLock::new();

fn get_g_spat() -> &'static Regex {
    G_SPAT.get_or_init(|| {
        Regex::new(
            r"(?i)^\w+\.(cc|com|net|me|club|jp|tv|xyz|biz|wiki|info|tw|us|de)@|^22-sht\.me|^(fhd|hd|sd|1080p|720p|4K)(-|_)|(-|_)(fhd|hd|sd|1080p|720p|4K|x264|x265|uncensored|hack|leak)"
        ).unwrap()
    })
}

/// Special site extraction rules
type ExtractFn = fn(&str) -> Option<String>;

/// Get the special site extraction rules (equivalent to G_TAKE_NUM_RULES)
fn get_take_num_rules() -> HashMap<&'static str, ExtractFn> {
    let mut rules: HashMap<&'static str, ExtractFn> = HashMap::new();

    // Tokyo Hot: (cz|gedo|k|n|red-|se)\d{2,4}
    rules.insert("tokyo.*hot", extract_tokyo_hot);

    // Carib: \d{6}(-|_)\d{3}, replace _ with -
    rules.insert("carib", extract_carib);

    // 1Pondo, Muramura, Pacopacomama: \d{6}(-|_)\d{3}, replace - with _
    rules.insert("1pon|mura|paco", extract_1pon_mura_paco);

    // 10Musume: \d{6}(-|_)\d{2}, replace - with _
    rules.insert("10mu", extract_10mu);

    // X-Art: x-art.\d{2}.\d{2}.\d{2}
    rules.insert("x-art", extract_xart);

    // XXX-AV: xxx-av-\d{3,5}
    rules.insert("xxx-av", extract_xxxav);

    // Heydouga: heydouga-\d{4}-\d{3,4}
    rules.insert("heydouga", extract_heydouga);

    // HEYZO: HEYZO-\d{4}
    rules.insert("heyzo", extract_heyzo);

    // MDBK: mdbk-\d{4}
    rules.insert("mdbk", extract_mdbk);

    // MDTM: mdtm-\d{4}
    rules.insert("mdtm", extract_mdtm);

    // Caribpr: \d{6}(-|_)\d{3}, replace _ with -
    rules.insert("caribpr", extract_caribpr);

    rules
}

// Extraction functions for each special site

fn extract_tokyo_hot(filename: &str) -> Option<String> {
    let re = Regex::new(r"(?i)(cz|gedo|k|n|red-|se)\d{2,4}").ok()?;
    re.find(filename).map(|m| m.as_str().to_string())
}

fn extract_carib(filename: &str) -> Option<String> {
    let re = Regex::new(r"(?i)\d{6}(-|_)\d{3}").ok()?;
    re.find(filename).map(|m| m.as_str().replace('_', "-"))
}

fn extract_1pon_mura_paco(filename: &str) -> Option<String> {
    let re = Regex::new(r"(?i)\d{6}(-|_)\d{3}").ok()?;
    re.find(filename).map(|m| m.as_str().replace('-', "_"))
}

fn extract_10mu(filename: &str) -> Option<String> {
    let re = Regex::new(r"(?i)\d{6}(-|_)\d{2}").ok()?;
    re.find(filename).map(|m| m.as_str().replace('-', "_"))
}

fn extract_xart(filename: &str) -> Option<String> {
    let re = Regex::new(r"(?i)x-art\.\d{2}\.\d{2}\.\d{2}").ok()?;
    re.find(filename).map(|m| m.as_str().to_string())
}

fn extract_xxxav(filename: &str) -> Option<String> {
    let re = Regex::new(r"(?i)xxx-av[^\d]*(\d{3,5})[^\d]*").ok()?;
    re.captures(filename).map(|caps| {
        format!("xxx-av-{}", &caps[1])
    })
}

fn extract_heydouga(filename: &str) -> Option<String> {
    let re = Regex::new(r"(?i)(\d{4})[-_](\d{3,4})[^\d]*").ok()?;
    re.captures(filename).map(|caps| {
        format!("heydouga-{}-{}", &caps[1], &caps[2])
    })
}

fn extract_heyzo(filename: &str) -> Option<String> {
    let re = Regex::new(r"(?i)heyzo[^\d]*(\d{4})").ok()?;
    re.captures(filename).map(|caps| {
        format!("HEYZO-{}", &caps[1])
    })
}

fn extract_mdbk(filename: &str) -> Option<String> {
    let re = Regex::new(r"(?i)mdbk(-|_)(\d{4})").ok()?;
    re.find(filename).map(|m| m.as_str().to_string())
}

fn extract_mdtm(filename: &str) -> Option<String> {
    let re = Regex::new(r"(?i)mdtm(-|_)(\d{4})").ok()?;
    re.find(filename).map(|m| m.as_str().to_string())
}

fn extract_caribpr(filename: &str) -> Option<String> {
    let re = Regex::new(r"(?i)\d{6}(-|_)\d{3}").ok()?;
    re.find(filename).map(|m| m.as_str().replace('_', "-"))
}

/// Try to extract number using special site rules
fn get_number_by_dict(filename: &str) -> Option<String> {
    let rules = get_take_num_rules();

    for (pattern, extract_fn) in rules.iter() {
        // Add case-insensitive flag to pattern
        let pattern_with_flag = format!("(?i){}", pattern);
        if let Ok(re) = Regex::new(&pattern_with_flag) {
            if re.is_match(filename) {
                if let Some(number) = extract_fn(filename) {
                    return Some(number);
                }
            }
        }
    }

    None
}

/// Strip common suffixes from the number and normalize
fn strip_suffix(file_number: &str) -> String {
    let mut result = file_number.to_string();

    // Check for -C suffix (Chinese subtitles)
    if let Ok(re) = Regex::new(r"(?i)(-|_)c$") {
        if re.is_match(&result) {
            result = re.replace(&result, "").to_string();
            return result.replace('_', "-").to_uppercase();
        }
    }

    // Check for -U suffix (uncensored)
    if let Ok(re) = Regex::new(r"(?i)(-|_)u$") {
        if re.is_match(&result) {
            result = re.replace(&result, "").to_string();
            return result.replace('_', "-").to_uppercase();
        }
    }

    // Check for -UC suffix (uncensored)
    if let Ok(re) = Regex::new(r"(?i)(-|_)uc$") {
        if re.is_match(&result) {
            result = re.replace(&result, "").to_string();
            return result.replace('_', "-").to_uppercase();
        }
    }

    // Check for XXXch suffix (chapter marker)
    if let Ok(re) = Regex::new(r"(?i)\d+ch$") {
        if re.is_match(&result) {
            result = result[..result.len() - 2].to_string();
            return result.replace('_', "-").to_uppercase();
        }
    }

    // Replace underscores with hyphens and uppercase
    result.replace('_', "-").to_uppercase()
}

/// Extract movie number from filename
///
/// This is the main entry point for number extraction, equivalent to Python's get_number()
///
/// # Arguments
///
/// * `file_path` - Full file path or just the filename
/// * `custom_regexs` - Optional custom regex patterns from config (space-separated)
///
/// # Examples
///
/// ```
/// use mdc_core::number_parser::get_number;
///
/// assert_eq!(get_number("ABC-123.mp4", None).unwrap(), "ABC-123");
/// assert_eq!(get_number("HEYZO-1234.mp4", None).unwrap(), "HEYZO-1234");
/// assert_eq!(get_number("carib-123456-789.mp4", None).unwrap(), "123456-789");
/// ```
pub fn get_number(file_path: &str, custom_regexs: Option<&str>) -> Result<String> {
    // Extract just the filename from the path
    let filepath = std::path::Path::new(file_path)
        .file_name()
        .and_then(|f| f.to_str())
        .ok_or_else(|| anyhow!("Invalid file path"))?;

    // Try custom regexes first
    if let Some(regexs) = custom_regexs {
        for regex_str in regexs.split_whitespace() {
            if let Ok(re) = Regex::new(regex_str) {
                if let Some(m) = re.find(filepath) {
                    return Ok(m.as_str().to_string());
                }
            }
        }
    }

    // Try special site rules
    if let Some(number) = get_number_by_dict(filepath) {
        return Ok(number);
    }

    // Check for subtitle markers or Japanese characters
    let has_sub_marker = filepath.contains("字幕组")
        || filepath.to_uppercase().contains("SUB")
        || Regex::new(r"[\u30a0-\u30ff]+").unwrap().is_match(filepath);

    if has_sub_marker {
        let mut cleaned = get_g_spat().replace_all(filepath, "").to_string();
        cleaned = Regex::new(r"\[.*?\]").unwrap().replace_all(&cleaned, "").to_string();
        cleaned = cleaned.replace(".chs", "").replace(".cht", "");

        if let Some(dot_pos) = cleaned.find('.') {
            let before_dot = &cleaned[..dot_pos];
            return Ok(before_dot.trim().to_string());
        }
    }

    // Handle filenames with - or _
    if filepath.contains('-') || filepath.contains('_') {
        let mut filename = get_g_spat().replace_all(filepath, "").to_string();

        // Remove date patterns like [2024-01-01] -
        filename = Regex::new(r"\[\d{4}-\d{1,2}-\d{1,2}\] - ")
            .unwrap()
            .replace_all(&filename, "")
            .to_string();

        // Special handling for FC2
        let lower_check = filename.to_lowercase();
        if lower_check.contains("fc2") {
            filename = lower_check
                .replace("--", "-")
                .replace('_', "-")
                .to_uppercase();
        }

        // Remove -CD1, -CD2, etc.
        filename = Regex::new(r"(?i)[-_]cd\d{1,2}")
            .unwrap()
            .replace_all(&filename, "")
            .to_string();

        // After removing -CD1, check if there's still a - or _
        if !filename.contains('-') && !filename.contains('_') {
            if let Some(dot_pos) = filename.find('.') {
                let before_dot = &filename[..dot_pos];
                if let Ok(re) = Regex::new(r"\w+") {
                    if let Some(m) = re.find(before_dot) {
                        return Ok(m.as_str().to_string());
                    }
                }
            }
        }

        // Extract the number part
        let file_number = if let Some(dot_pos) = filename.find('.') {
            &filename[..dot_pos]
        } else {
            &filename
        };

        // Try to extract alphanumeric with - and _
        let file_number = if let Ok(re) = Regex::new(r"[\w\-_]+") {
            re.find(file_number)
                .map(|m| m.as_str())
                .unwrap_or(file_number)
        } else {
            file_number
        };

        // Strip suffixes and return
        return Ok(strip_suffix(file_number));
    }

    // Handle filenames without - (FANZA CID, Western formats)

    // Western format: xxx.YY.MM.DD
    if let Ok(re) = Regex::new(r"[a-zA-Z]+\.\d{2}\.\d{2}\.\d{2}") {
        if let Some(m) = re.find(filepath) {
            return Ok(m.as_str().to_string());
        }
    }

    // Extract filename before extension
    if let Some(dot_pos) = filepath.rfind('.') {
        let before_dot = &filepath[..dot_pos];
        let cleaned = before_dot.replace('_', "-");
        return Ok(cleaned);
    }

    Err(anyhow!("Could not extract number from: {}", file_path))
}

/// Check if a movie number represents an uncensored video
///
/// # Arguments
///
/// * `number` - The movie number to check
/// * `uncensored_prefixes` - Optional comma-separated list of uncensored prefixes from config
///
/// # Examples
///
/// ```
/// use mdc_core::number_parser::is_uncensored;
///
/// assert!(is_uncensored("123456-789", None));
/// assert!(is_uncensored("HEYZO-1234", None));
/// assert!(is_uncensored("n1234", None));
/// assert!(!is_uncensored("ABC-123", None));
/// ```
pub fn is_uncensored(number: &str, uncensored_prefixes: Option<&str>) -> bool {
    // Built-in uncensored patterns
    // Note: [\d-]{4,} should match at start of string to avoid false positives like "ABC-123"
    let builtin_pattern = Regex::new(
        r"(?i)^[\d-]{4,}$|\d{6}_\d{2,3}|(cz|gedo|k|n|red-|se)\d{2,4}|heyzo.+|xxx-av-.+|heydouga-.+|x-art\.\d{2}\.\d{2}\.\d{2}"
    ).unwrap();

    if builtin_pattern.is_match(number) {
        return true;
    }

    // Check custom prefixes from config
    if let Some(prefixes) = uncensored_prefixes {
        let prefix_list: Vec<&str> = prefixes.split(',').filter(|s| !s.is_empty()).collect();
        if !prefix_list.is_empty() {
            // Build pattern: first|second.+|third.+
            let mut pattern = prefix_list[0].to_string();
            for prefix in &prefix_list[1..] {
                pattern.push_str(&format!("|{}.+", prefix));
            }

            if let Ok(re) = Regex::new(&format!("(?i){}", pattern)) {
                return re.is_match(number);
            }
        }
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_number_basic() {
        assert_eq!(get_number("ABC-123.mp4", None).unwrap(), "ABC-123");
        assert_eq!(get_number("XYZ_456.avi", None).unwrap(), "XYZ-456");
    }

    #[test]
    fn test_get_number_with_suffix() {
        assert_eq!(get_number("ABC-123-C.mp4", None).unwrap(), "ABC-123");
        assert_eq!(get_number("ABC-123-U.mp4", None).unwrap(), "ABC-123");
        assert_eq!(get_number("ABC-123-UC.mp4", None).unwrap(), "ABC-123");
        assert_eq!(get_number("ABC-123ch.mp4", None).unwrap(), "ABC-123");
    }

    #[test]
    fn test_get_number_fc2() {
        let result = get_number("FC2-PPV-1234567.mp4", None).unwrap();
        assert!(result.contains("FC2"));
    }

    #[test]
    fn test_get_number_tokyo_hot() {
        assert_eq!(get_number("tokyo-hot-n1234.mp4", None).unwrap(), "n1234");
        assert_eq!(get_number("k0123.avi", None).unwrap(), "k0123");
    }

    #[test]
    fn test_get_number_carib() {
        assert_eq!(get_number("carib-123456-789.mp4", None).unwrap(), "123456-789");
        assert_eq!(get_number("caribbeancom-123456_789.mp4", None).unwrap(), "123456-789");
    }

    #[test]
    fn test_get_number_1pon() {
        assert_eq!(get_number("1pondo_123456_789.mp4", None).unwrap(), "123456_789");
    }

    #[test]
    fn test_get_number_heyzo() {
        assert_eq!(get_number("HEYZO-1234.mp4", None).unwrap(), "HEYZO-1234");
        assert_eq!(get_number("heyzo_hd_1234.avi", None).unwrap(), "HEYZO-1234");
    }

    #[test]
    fn test_get_number_heydouga() {
        let result = get_number("heydouga-4017-123.mp4", None).unwrap();
        assert_eq!(result, "heydouga-4017-123");
    }

    #[test]
    fn test_get_number_with_quality_prefix() {
        assert_eq!(get_number("FHD-ABC-123.mp4", None).unwrap(), "ABC-123");
        assert_eq!(get_number("1080p_XYZ-456.mkv", None).unwrap(), "XYZ-456");
    }

    #[test]
    fn test_get_number_cd_marker() {
        assert_eq!(get_number("ABC-123-CD1.mp4", None).unwrap(), "ABC-123");
        assert_eq!(get_number("ABC-123-cd2.avi", None).unwrap(), "ABC-123");
    }

    #[test]
    fn test_get_number_custom_regex() {
        let custom = r"CUSTOM-\d+";
        assert_eq!(
            get_number("prefix_CUSTOM-999_suffix.mp4", Some(custom)).unwrap(),
            "CUSTOM-999"
        );
    }

    #[test]
    fn test_is_uncensored_builtin() {
        assert!(is_uncensored("123456-789", None));
        assert!(is_uncensored("HEYZO-1234", None));
        assert!(is_uncensored("n1234", None));
        assert!(is_uncensored("red-123", None));
        assert!(is_uncensored("xxx-av-12345", None));
    }

    #[test]
    fn test_is_uncensored_custom_prefix() {
        assert!(is_uncensored("CUSTOM-123", Some("CUSTOM")));
        assert!(is_uncensored("FOO-456", Some("FOO,BAR")));
        assert!(!is_uncensored("ABC-123", Some("CUSTOM,FOO")));
    }

    #[test]
    fn test_is_censored() {
        assert!(!is_uncensored("ABC-123", None));
        assert!(!is_uncensored("MDBK-001", None));
        assert!(!is_uncensored("SSIS-456", None));
    }

    #[test]
    fn test_western_format() {
        assert_eq!(get_number("x-art.18.05.15.mp4", None).unwrap(), "x-art.18.05.15");
    }
}
