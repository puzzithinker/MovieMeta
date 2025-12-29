//! Movie number parser
//!
//! This module extracts movie numbers from filenames using various patterns and rules.
//! It's a direct port of the Python `number_parser.py` with 100% compatibility.

use anyhow::{anyhow, Result};
use regex::Regex;
use std::collections::HashMap;
use std::sync::OnceLock;

/// Result of number parsing with dual ID support
///
/// This structure provides both human-readable and API-compatible ID formats,
/// along with detected attributes and multi-part information.
#[derive(Debug, Clone, PartialEq)]
pub struct ParsedNumber {
    /// Human-readable ID for display and most scrapers (e.g., "SSIS-123")
    pub id: String,

    /// Content ID for DMM/API queries (e.g., "ssis00123")
    /// Lowercase, zero-padded to 5 digits minimum
    pub content_id: String,

    /// Part number if multi-part detected (1, 2, etc.)
    pub part_number: Option<u8>,

    /// Detected attributes from filename
    pub attributes: ParsedAttributes,
}

impl ParsedNumber {
    /// Create a ParsedNumber from a display ID
    pub fn from_id(id: String) -> Self {
        let content_id = convert_to_content_id(&id);
        Self {
            id,
            content_id,
            part_number: None,
            attributes: ParsedAttributes::default(),
        }
    }

    /// Create a ParsedNumber from a display ID with part number
    pub fn from_id_with_part(id: String, part: Option<u8>) -> Self {
        let content_id = convert_to_content_id(&id);
        Self {
            id,
            content_id,
            part_number: part,
            attributes: ParsedAttributes::default(),
        }
    }
}

impl std::fmt::Display for ParsedNumber {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.id)
    }
}

impl From<ParsedNumber> for String {
    fn from(parsed: ParsedNumber) -> Self {
        parsed.id
    }
}

/// Attributes detected during parsing
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ParsedAttributes {
    /// Chinese subtitles detected (-C suffix)
    pub cn_sub: bool,

    /// Uncensored content detected (-U or -UC suffix)
    pub uncensored: bool,

    /// Special site detected (tokyo-hot, carib, fc2, etc.)
    pub special_site: Option<String>,
}

/// Parser configuration
#[derive(Debug, Clone)]
pub struct ParserConfig {
    /// Custom regex patterns to try before built-in patterns
    pub custom_regexs: Vec<String>,

    /// Strings to remove during filename cleaning
    pub removal_strings: Vec<String>,

    /// Enable strict mode for non-standard formats
    pub strict_mode: bool,

    /// Capture group index for ID in custom regex (default: 1)
    pub regex_id_match: usize,

    /// Capture group index for part number in custom regex (default: 2)
    pub regex_pt_match: usize,

    /// Comma-separated uncensored prefixes
    pub uncensored_prefixes: String,
}

impl Default for ParserConfig {
    fn default() -> Self {
        Self {
            custom_regexs: Vec::new(),
            removal_strings: get_default_removal_strings(),
            strict_mode: false,
            regex_id_match: 1,
            regex_pt_match: 2,
            uncensored_prefixes: String::new(),
        }
    }
}

/// Get default removal strings (from Javinizer)
fn get_default_removal_strings() -> Vec<String> {
    vec![
        "22-sht.me".to_string(),
        "1080p".to_string(),
        "720p".to_string(),
        "480p".to_string(),
        "h.264".to_string(),
        "h.265".to_string(),
        "hevc".to_string(),
        "x264".to_string(),
        "x265".to_string(),
        "uncensored".to_string(),
        "leaked".to_string(),
        "hack".to_string(),
    ]
}

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

/// Convert display ID to DMM content ID format
///
/// Transforms human-readable IDs to API-compatible format by:
/// 1. Lowercasing all letters
/// 2. Removing hyphens/underscores
/// 3. Zero-padding numeric portion to 5 digits minimum
///
/// # Examples
/// ```
/// # use mdc_core::number_parser::convert_to_content_id;
/// assert_eq!(convert_to_content_id("SSIS-123"), "ssis00123");
/// assert_eq!(convert_to_content_id("ABP-1"), "abp00001");
/// assert_eq!(convert_to_content_id("FC2-PPV-1234567"), "fc2ppv1234567");
/// ```
pub fn convert_to_content_id(display_id: &str) -> String {
    // Pattern: ^([A-Za-z]+)[-_]?(\d+)([A-Za-z]*)$
    // Extract prefix, digits, and suffix

    // Handle special formats
    let lower = display_id.to_lowercase();

    // FC2 special handling: FC2-PPV-1234567 → fc2ppv1234567
    if lower.starts_with("fc2") {
        return lower.replace("-", "").replace("_", "");
    }

    // HEYZO special handling: HEYZO-1234 → heyzo01234
    if lower.starts_with("heyzo") {
        let digits: String = lower.chars().filter(|c| c.is_numeric()).collect();
        let padded = format!("{:0>5}", digits);
        return format!("heyzo{}", padded);
    }

    // Tokyo-Hot special handling: n1234, k0123 → keep as-is (already content format)
    let re_tokyohot = Regex::new(r"^(cz|gedo|k|n|red|se)\d+$").unwrap();
    if re_tokyohot.is_match(&lower) {
        return lower;
    }

    // Standard format: ABC-123 → abc00123
    let re = Regex::new(r"^([A-Za-z]+)[-_]?(\d+)([A-Za-z]*)$").unwrap();
    if let Some(caps) = re.captures(display_id) {
        let prefix = caps[1].to_lowercase();
        let digits = &caps[2];
        let suffix = caps.get(3).map_or("", |m| m.as_str()).to_lowercase();

        // Pad digits to 5 places
        let padded = format!("{:0>5}", digits);

        return format!("{}{}{}", prefix, padded, suffix);
    }

    // Fallback: just lowercase and remove separators
    lower.replace("-", "").replace("_", "")
}

/// Convert content ID to display ID format
///
/// Transforms API-compatible IDs to human-readable format by:
/// 1. Uppercasing letters
/// 2. Inserting hyphen between prefix and digits
/// 3. Trimming leading zeros (minimum 3 digits)
///
/// # Examples
/// ```
/// # use mdc_core::number_parser::convert_to_display_id;
/// assert_eq!(convert_to_display_id("ssis00123"), "SSIS-123");
/// assert_eq!(convert_to_display_id("abp00001"), "ABP-001");
/// assert_eq!(convert_to_display_id("mide00099"), "MIDE-099");
/// ```
pub fn convert_to_display_id(content_id: &str) -> String {
    // Handle special formats
    let lower = content_id.to_lowercase();

    // FC2 special handling: fc2ppv1234567 → FC2-PPV-1234567
    if lower.starts_with("fc2") {
        let digits: String = lower.chars().filter(|c| c.is_numeric()).collect();
        if lower.contains("ppv") {
            return format!("FC2-PPV-{}", digits);
        } else {
            return format!("FC2-{}", digits);
        }
    }

    // HEYZO special handling: heyzo01234 → HEYZO-1234
    if lower.starts_with("heyzo") {
        let digits: String = lower.chars().filter(|c| c.is_numeric()).collect();
        let trimmed = digits.trim_start_matches('0');
        let final_digits = if trimmed.is_empty() { "0" } else { trimmed };
        return format!("HEYZO-{}", final_digits);
    }

    // Tokyo-Hot special handling: n01234, k0123 → n1234, k123 (no hyphen)
    let re_tokyohot = Regex::new(r"^([a-z]+)(\d+)$").unwrap();
    if let Some(caps) = re_tokyohot.captures(&lower) {
        let prefix = &caps[1];
        if matches!(prefix, "cz" | "gedo" | "k" | "n" | "red" | "se") {
            let digits = &caps[2];
            let trimmed = digits.trim_start_matches('0');
            let final_digits = if trimmed.is_empty() { "0" } else { trimmed };
            return format!("{}{}", prefix, final_digits);
        }
    }

    // Standard format: abc00123 → ABC-123
    let re = Regex::new(r"^([a-z]+)(\d+)([a-z]*)$").unwrap();
    if let Some(caps) = re.captures(&lower) {
        let prefix = caps[1].to_uppercase();
        let digits = &caps[2];
        let suffix = caps.get(3).map_or(String::new(), |m| m.as_str().to_uppercase());

        // Trim leading zeros but keep at least 3 digits
        let trimmed = digits.trim_start_matches('0');
        let num_value = trimmed.parse::<usize>().unwrap_or(0);
        let final_digits = format!("{:03}", num_value);

        return format!("{}-{}{}", prefix, final_digits, suffix);
    }

    // Fallback: just uppercase
    content_id.to_uppercase()
}

/// Clean filename by removing website tags, quality markers, and other noise
/// This runs BEFORE number extraction to improve accuracy
fn clean_filename(filename: &str) -> String {
    let mut cleaned = filename.to_string();

    // Strip parenthesized quality markers at start: (HD), (FHD), (4K), etc.
    // Fixes: (HD)avop-212A.HD.mp4 → avop-212A.HD.mp4
    if let Ok(re) = Regex::new(r"(?i)^\(?(HD|FHD|4K|1080P|720P|480P|UHD)\)?[-_]?") {
        cleaned = re.replace_all(&cleaned, "").to_string();
    }

    // Strip website tags: [xxx.com], [xxx], etc.
    if let Ok(re) = Regex::new(r"\[([^\]]+)\]") {
        cleaned = re.replace_all(&cleaned, "").to_string();
    }

    // Strip email/username prefixes: user@domain@, username@site.com@
    if let Ok(re) = Regex::new(r"^[^@]+@[^@]+@") {
        cleaned = re.replace_all(&cleaned, "").to_string();
    }

    // Strip domain prefixes: domain.com-, site.tv-, etc.
    if let Ok(re) = Regex::new(r"^[\w.-]+\.(com|net|tv|la|me|cc|club|jp|xyz|biz|wiki|info|tw|us|de)-") {
        cleaned = re.replace_all(&cleaned, "").to_string();
    }

    // Strip numeric date prefixes: 0201-, 20240201-, etc. (common in organized collections)
    // Match 4-8 digits followed by dash/underscore at the start
    if let Ok(re) = Regex::new(r"^\d{4,8}[-_]") {
        cleaned = re.replace_all(&cleaned, "").to_string();
    }

    // Strip part markers EARLY: -1, -2, A, B, _1, _2, cd1, cd2, part1, pt2, etc.
    // This MUST run before quality marker removal so TEST-FHD-CD1 becomes TEST-FHD, then TEST
    // Match before extension or at end
    if let Ok(re) = Regex::new(r"(?i)[-_]?(cd|part|pt|disk|disc)[-_]?[12AB](\.|$)") {
        cleaned = re.replace_all(&cleaned, "$2").to_string();
    }

    // Strip quality markers that directly follow digits with NO separator (e.g., "CZBD-015FULLHD.mp4")
    // This handles cases like 015FULLHD, 123HD, 456FHD, etc.
    if let Ok(re) = Regex::new(r"(?i)(\d)(FULLHD|1080P|720P|480P|H\.?265|H\.?264|X265|X264|HEVC|FHD|4K|UHD|HQ|HD)(\.|$)") {
        cleaned = re.replace_all(&cleaned, "$1$3").to_string();
    }

    // Strip quality markers that appear after dashes following numbers (after valid codes)
    // Fixes: DSAMBD-18-H265-1080P.mp4 → DSAMBD-18.mp4
    // This runs FIRST to handle multiple consecutive quality markers in one go
    // Only strips when quality marker comes AFTER digits (end of valid code)
    // This prevents stripping "heyzo_hd_1234" where hd is part of the original name
    // IMPORTANT: Longer patterns first to avoid partial matches (H265 before HD, FULLHD before HD, etc.)
    if let Ok(re) = Regex::new(r"(?i)(\d+)[-_](H\.?265|H\.?264|FULLHD|1080P|720P|480P|X265|X264|HEVC|FHD|4K|UHD|HD|HQ).*?(\.|$)") {
        cleaned = re.replace_all(&cleaned, "$1$3").to_string();
    }

    // Strip common quality markers at end using simple string replacement
    // These suffixes are always noise when at the very end
    let quality_suffixes = [
        "-1080P", "_1080P", "-720P", "_720P", "-480P", "_480P",
        "-FULLHD", "_FULLHD", "-FHD", "_FHD", "-4K", "_4K",
        "-UHD", "_UHD", "-HQ", "_HQ",
    ];
    for suffix in &quality_suffixes {
        if cleaned.to_uppercase().ends_with(suffix) {
            let len = cleaned.len() - suffix.len();
            cleaned.truncate(len);
            break;
        }
    }

    // Also strip HD at end, but only if NOT followed by dash+letters (to preserve HD-IPX, HD-SSIS)
    if cleaned.to_uppercase().ends_with("-HD") || cleaned.to_uppercase().ends_with("_HD") {
        // Check if this looks like it might be part of a code
        let temp = cleaned.to_uppercase();
        if !temp.contains("HD-") {
            // HD is a suffix, not part of code
            let len = cleaned.len() - 3; // Remove "-HD" or "_HD"
            cleaned.truncate(len);
        }
    }

    // Strip quality markers before file extension
    if let Ok(re) = Regex::new(r"(?i)[-_](FULLHD|1080P|720P|480P|FHD|4K|UHD|HQ)\.") {
        cleaned = re.replace_all(&cleaned, ".").to_string();
    }

    // Also handle HD before extension (check not part of code like HD-IPX)
    if cleaned.to_uppercase().contains("-HD.") || cleaned.to_uppercase().contains("_HD.") {
        let temp = cleaned.to_uppercase();
        if !temp.contains("HD-") {
            cleaned = cleaned.replace("-HD.", ".").replace("_HD.", ".").replace("-hd.", ".").replace("_hd.", ".");
        }
    }

    // Strip Japanese/Chinese characters (actress names, descriptions)
    // Fixes: hnd-809 神宮寺ナオ.mp4 → hnd-809.mp4
    // Keep only the code part (before first space + CJK character)
    if let Ok(re) = Regex::new(r"[\s　]+[\p{Han}\p{Hiragana}\p{Katakana}].*$") {
        cleaned = re.replace_all(&cleaned, "").to_string();
    }

    // Clean up multiple consecutive separators (-- or __)
    if let Ok(re) = Regex::new(r"[-_]{2,}") {
        cleaned = re.replace_all(&cleaned, "-").to_string();
    }

    // Clean up leading/trailing separators
    cleaned.trim_matches(|c| c == '-' || c == '_' || c == ' ').to_string()
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

    // Check if this is a Tokyo-Hot ID (should remain lowercase)
    let tokyo_hot_re = Regex::new(r"^(?i)(cz|gedo|k|n|red|se)\d{2,4}$").unwrap();
    if tokyo_hot_re.is_match(&result) {
        // Tokyo-Hot IDs remain lowercase
        return result.replace('_', "-").to_lowercase();
    }

    // Replace underscores with hyphens and uppercase
    result.replace('_', "-").to_uppercase()
}

/// Parse movie number from filename with dual ID support
///
/// This is the new recommended API that returns both display and content IDs,
/// along with detected attributes and multi-part information.
///
/// # Arguments
///
/// * `file_path` - Full file path or just the filename
/// * `config` - Optional parser configuration
///
/// # Examples
///
/// ```
/// use mdc_core::number_parser::parse_number;
///
/// let result = parse_number("SSIS-123.mp4", None).unwrap();
/// assert_eq!(result.id, "SSIS-123");
/// assert_eq!(result.content_id, "ssis00123");
/// ```
pub fn parse_number(file_path: &str, config: Option<&ParserConfig>) -> Result<ParsedNumber> {
    let default_config = ParserConfig::default();
    let config = config.unwrap_or(&default_config);

    // Extract just the filename from the path
    let filepath = std::path::Path::new(file_path)
        .file_name()
        .and_then(|f| f.to_str())
        .ok_or_else(|| anyhow!("Invalid file path"))?;

    // Clean the filename first to remove website tags, quality markers, etc.
    let cleaned_filepath = clean_filename(filepath);

    // Try custom regexes first (on cleaned filename) with capture group support
    if !config.custom_regexs.is_empty() {
        for regex_str in &config.custom_regexs {
            if let Ok(re) = Regex::new(regex_str) {
                if let Some(caps) = re.captures(&cleaned_filepath) {
                    // Extract ID from configured capture group
                    let id = caps.get(config.regex_id_match)
                        .map(|m| m.as_str().to_string())
                        .or_else(|| caps.get(0).map(|m| m.as_str().to_string()))
                        .unwrap_or_default();

                    // Extract part number if configured
                    let part = caps.get(config.regex_pt_match)
                        .and_then(|m| m.as_str().parse::<u8>().ok());

                    if !id.is_empty() {
                        let content_id = convert_to_content_id(&id);
                        return Ok(ParsedNumber {
                            id,
                            content_id,
                            part_number: part,
                            attributes: ParsedAttributes::default(),
                        });
                    }
                }
            }
        }
    }

    // Try special site rules (on cleaned filename)
    if let Some(number) = get_number_by_dict(&cleaned_filepath) {
        let content_id = convert_to_content_id(&number);
        let mut attrs = ParsedAttributes::default();

        // Detect special site
        if number.starts_with("n") || number.starts_with("k") || number.contains("tokyo") {
            attrs.special_site = Some("tokyo-hot".to_string());
        } else if Regex::new(r"\d{6}[-_]\d{3}").unwrap().is_match(&number) {
            if number.contains('-') {
                attrs.special_site = Some("carib".to_string());
            } else {
                attrs.special_site = Some("1pon".to_string());
            }
        } else if number.to_lowercase().contains("fc2") {
            attrs.special_site = Some("fc2".to_string());
        } else if number.to_uppercase().starts_with("HEYZO") {
            attrs.special_site = Some("heyzo".to_string());
        }

        return Ok(ParsedNumber {
            id: number,
            content_id,
            part_number: None,
            attributes: attrs,
        });
    }

    // Use the existing get_number logic to extract the base ID
    let base_id = extract_number_internal(filepath, &cleaned_filepath)?;

    // Detect attributes from suffix
    let mut attrs = ParsedAttributes::default();

    // Check for -C suffix (Chinese subtitles)
    attrs.cn_sub = Regex::new(r"(?i)[-_]c$").unwrap().is_match(&base_id);

    // Check for -U or -UC suffix (uncensored)
    attrs.uncensored = Regex::new(r"(?i)[-_]u(c)?$").unwrap().is_match(&base_id);

    // Strip the suffix to get clean ID
    let clean_id = strip_suffix(&base_id);
    let content_id = convert_to_content_id(&clean_id);

    Ok(ParsedNumber {
        id: clean_id,
        content_id,
        part_number: None,
        attributes: attrs,
    })
}

/// Internal helper to extract number using existing logic
fn extract_number_internal(filepath: &str, cleaned_filepath: &str) -> Result<String> {
    // Check for subtitle markers or Japanese characters (check original filepath for these)
    let has_sub_marker = filepath.contains("字幕组")
        || filepath.to_uppercase().contains("SUB")
        || Regex::new(r"[\u30a0-\u30ff]+").unwrap().is_match(filepath);

    if has_sub_marker {
        // Use cleaned_filepath here since we already stripped tags
        let mut cleaned = get_g_spat().replace_all(cleaned_filepath, "").to_string();
        cleaned = cleaned.replace(".chs", "").replace(".cht", "");

        if let Some(dot_pos) = cleaned.find('.') {
            let before_dot = &cleaned[..dot_pos];
            return Ok(before_dot.trim().to_string());
        }
    }

    // Handle filenames with - or _ (use cleaned version)
    if cleaned_filepath.contains('-') || cleaned_filepath.contains('_') {
        let mut filename = get_g_spat().replace_all(cleaned_filepath, "").to_string();

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

        return Ok(file_number.to_string());
    }

    // Handle filenames without - (FANZA CID, Western formats)
    // Use cleaned version for these checks too

    // Western format: xxx.YY.MM.DD
    if let Ok(re) = Regex::new(r"[a-zA-Z]+\.\d{2}\.\d{2}\.\d{2}") {
        if let Some(m) = re.find(cleaned_filepath) {
            return Ok(m.as_str().to_string());
        }
    }

    // Extract filename before extension (from cleaned version)
    if let Some(dot_pos) = cleaned_filepath.rfind('.') {
        let before_dot = &cleaned_filepath[..dot_pos];
        let cleaned = before_dot.replace('_', "-");
        return Ok(cleaned);
    }

    // If cleaned version has no extension, just return it
    if !cleaned_filepath.is_empty() {
        return Ok(cleaned_filepath.replace('_', "-"));
    }

    Err(anyhow!("Could not extract number from filename"))
}

/// Extract movie number from filename
///
/// This is the main entry point for number extraction, equivalent to Python's get_number()
///
/// **DEPRECATED**: Use `parse_number()` instead for dual ID support and attribute detection.
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
    // Convert old-style custom_regexs string to new config format
    let config = if let Some(regexs) = custom_regexs {
        let mut cfg = ParserConfig::default();
        cfg.custom_regexs = regexs.split_whitespace().map(|s| s.to_string()).collect();
        Some(cfg)
    } else {
        None
    };

    // Use parse_number and return just the ID for backward compatibility
    let parsed = parse_number(file_path, config.as_ref())?;
    Ok(parsed.id)
}

/// Legacy implementation preserved for reference
#[allow(dead_code)]
fn get_number_legacy(file_path: &str, custom_regexs: Option<&str>) -> Result<String> {
    // Extract just the filename from the path
    let filepath = std::path::Path::new(file_path)
        .file_name()
        .and_then(|f| f.to_str())
        .ok_or_else(|| anyhow!("Invalid file path"))?;

    // Clean the filename first to remove website tags, quality markers, etc.
    let cleaned_filepath = clean_filename(filepath);

    // Try custom regexes first (on cleaned filename)
    if let Some(regexs) = custom_regexs {
        for regex_str in regexs.split_whitespace() {
            if let Ok(re) = Regex::new(regex_str) {
                if let Some(m) = re.find(&cleaned_filepath) {
                    return Ok(m.as_str().to_string());
                }
            }
        }
    }

    // Try special site rules (on cleaned filename)
    if let Some(number) = get_number_by_dict(&cleaned_filepath) {
        return Ok(number);
    }

    // Check for subtitle markers or Japanese characters (check original filepath for these)
    let has_sub_marker = filepath.contains("字幕组")
        || filepath.to_uppercase().contains("SUB")
        || Regex::new(r"[\u30a0-\u30ff]+").unwrap().is_match(filepath);

    if has_sub_marker {
        // Use cleaned_filepath here since we already stripped tags
        let mut cleaned = get_g_spat().replace_all(&cleaned_filepath, "").to_string();
        cleaned = cleaned.replace(".chs", "").replace(".cht", "");

        if let Some(dot_pos) = cleaned.find('.') {
            let before_dot = &cleaned[..dot_pos];
            return Ok(before_dot.trim().to_string());
        }
    }

    // Handle filenames with - or _ (use cleaned version)
    if cleaned_filepath.contains('-') || cleaned_filepath.contains('_') {
        let mut filename = get_g_spat().replace_all(&cleaned_filepath, "").to_string();

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
    // Use cleaned version for these checks too

    // Western format: xxx.YY.MM.DD
    if let Ok(re) = Regex::new(r"[a-zA-Z]+\.\d{2}\.\d{2}\.\d{2}") {
        if let Some(m) = re.find(&cleaned_filepath) {
            return Ok(m.as_str().to_string());
        }
    }

    // Extract filename before extension (from cleaned version)
    if let Some(dot_pos) = cleaned_filepath.rfind('.') {
        let before_dot = &cleaned_filepath[..dot_pos];
        let cleaned = before_dot.replace('_', "-");
        return Ok(cleaned);
    }

    // If cleaned version has no extension, just return it
    if !cleaned_filepath.is_empty() {
        return Ok(cleaned_filepath.replace('_', "-"));
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

    // New tests for filename cleaning functionality

    #[test]
    fn test_clean_website_tags() {
        // Test various website tag patterns
        assert_eq!(get_number("[Thz.la]jufd-643.mp4", None).unwrap(), "JUFD-643");
        assert_eq!(get_number("[7sht.me]SSIS-123.mp4", None).unwrap(), "SSIS-123");
        assert_eq!(get_number("[ses23.com]ABC-456.avi", None).unwrap(), "ABC-456");
    }

    #[test]
    fn test_clean_email_username_prefixes() {
        // Test email/username@ patterns
        let result = get_number("roger92402094@www.sexinsex.net@AVGL-012.avi", None).unwrap();
        assert_eq!(result, "AVGL-012");
    }

    #[test]
    fn test_clean_domain_prefixes() {
        // Test domain.com- patterns
        // Note: BEB077 without dash stays as BEB077 (no automatic dash insertion)
        assert_eq!(get_number("jp.myav.tv-BEB077.avi", None).unwrap(), "BEB077");
        assert_eq!(get_number("www.site.com-ABC-123.mp4", None).unwrap(), "ABC-123");
        // With dash in original, it's preserved
        assert_eq!(get_number("jp.myav.tv-BEB-077.avi", None).unwrap(), "BEB-077");
    }

    #[test]
    fn test_clean_quality_suffixes() {
        // Test quality marker removal
        assert_eq!(get_number("CZBD-015FULLHD.mp4", None).unwrap(), "CZBD-015");
        assert_eq!(get_number("ABC-123-FHD.mp4", None).unwrap(), "ABC-123");
        assert_eq!(get_number("XYZ-456-1080P.mkv", None).unwrap(), "XYZ-456");
        assert_eq!(get_number("TEST-789-4K.mp4", None).unwrap(), "TEST-789");
        assert_eq!(get_number("MOVIE-001-H265.mp4", None).unwrap(), "MOVIE-001");
    }

    #[test]
    fn test_clean_combined_patterns() {
        // Test multiple cleaning patterns at once
        assert_eq!(get_number("[Thz.la]jufd-643-FHD.mp4", None).unwrap(), "JUFD-643");
        assert_eq!(get_number("[site.com]ABC-123-1080P-C.mp4", None).unwrap(), "ABC-123");
    }

    #[test]
    fn test_clean_part_markers() {
        // Test part marker removal (cd1, cd2, etc.)
        assert_eq!(get_number("ABC-123-cd1.mp4", None).unwrap(), "ABC-123");
        assert_eq!(get_number("XYZ-456-part2.avi", None).unwrap(), "XYZ-456");
    }

    #[test]
    fn test_real_world_problematic_files() {
        // Real filenames from user's failed list
        assert_eq!(get_number("[Thz.la]jufd-643.mp4", None).unwrap(), "JUFD-643");

        let result1 = get_number("roger92402094@www.sexinsex.net@AVGL-012.avi", None).unwrap();
        assert_eq!(result1, "AVGL-012");

        // BEB077 without dash stays as BEB077
        assert_eq!(get_number("jp.myav.tv-BEB077.avi", None).unwrap(), "BEB077");
        assert_eq!(get_number("CZBD-015FULLHD.mp4", None).unwrap(), "CZBD-015");
    }

    #[test]
    fn test_clean_filename_function() {
        // Direct tests of the clean_filename function
        assert_eq!(clean_filename("[Thz.la]jufd-643"), "jufd-643");
        assert_eq!(clean_filename("site.com-ABC-123"), "ABC-123");
        assert_eq!(clean_filename("MOVIE-1080P"), "MOVIE");
        // TEST-FHD-CD1: FHD is stripped as quality marker, then CD1 is stripped as part marker
        assert_eq!(clean_filename("TEST-FHD-CD1"), "TEST");
        assert_eq!(clean_filename("MOVIE-PART2"), "MOVIE");
        // Date prefix stripping
        assert_eq!(clean_filename("0201-SNIS091"), "SNIS091");
        assert_eq!(clean_filename("20240201-ABC-123"), "ABC-123");
    }

    #[test]
    fn test_date_prefix_stripping() {
        // Test files with date prefixes
        assert_eq!(get_number("0201-SNIS-091.mp4", None).unwrap(), "SNIS-091");
        // Without dash in code - the date prefix should be stripped
        let result = get_number("0201-SNIS091.mp4", None).unwrap();
        println!("Extracted from '0201-SNIS091.mp4': {}", result);
        // Should extract SNIS091, not 0201-SNIS091
        assert!(!result.contains("0201"), "Should not contain date prefix '0201'");
        assert!(result.contains("SNIS"), "Should contain 'SNIS'");
    }

    #[test]
    fn test_parentheses_quality_markers() {
        // Priority 1: Strip parenthesized quality markers at start
        assert_eq!(get_number("(HD)avop-212A.HD.mp4", None).unwrap(), "AVOP-212A");
        assert_eq!(get_number("(FHD)ABC-123.mp4", None).unwrap(), "ABC-123");
        assert_eq!(get_number("(4K)XYZ-456.mp4", None).unwrap(), "XYZ-456");
        assert_eq!(get_number("(1080P)TEST-001.mkv", None).unwrap(), "TEST-001");
    }

    #[test]
    fn test_japanese_text_stripping() {
        // Priority 2: Strip Japanese/Chinese characters (actress names, descriptions)
        // Space + Japanese should be stripped
        assert_eq!(get_number("hnd-809 神宮寺ナオ.mp4", None).unwrap(), "HND-809");
        assert_eq!(get_number("ABC-123 波多野結衣.mp4", None).unwrap(), "ABC-123");
        assert_eq!(get_number("SSIS-001 明日花キララ.mp4", None).unwrap(), "SSIS-001");
        // Full-width space (　) should also be handled
        assert_eq!(get_number("IPX-456　桜空もも.mp4", None).unwrap(), "IPX-456");
    }

    #[test]
    fn test_quality_markers_mid_string() {
        // Priority 3: Strip quality markers that appear after dashes mid-string
        assert_eq!(get_number("DSAMBD-18-H265-1080P.mp4", None).unwrap(), "DSAMBD-18");
        assert_eq!(get_number("ABC-123-FHD-720P.mp4", None).unwrap(), "ABC-123");
        assert_eq!(get_number("XYZ-456-X264-HD.mkv", None).unwrap(), "XYZ-456");
        assert_eq!(get_number("TEST-001-HEVC-4K.mp4", None).unwrap(), "TEST-001");
    }
}
