//! Movie number parser with dual ID support
//!
//! This module extracts movie numbers from filenames using various patterns and rules.
//! It provides a **dual ID system** that generates both human-readable display IDs
//! and API-compatible content IDs for maximum scraper compatibility.
//!
//! ## Overview
//!
//! JAV (Japanese Adult Video) movies use multiple ID formats depending on the source:
//! - **Display Format**: Human-readable IDs like `SSIS-123`, `ABP-456` (used in filenames, NFO files)
//! - **Content Format**: API-compatible IDs like `ssis00123`, `abp00456` (used by DMM, JAVLibrary)
//!
//! This module automatically generates both formats, enabling seamless integration with
//! all scraper types while maintaining backward compatibility with the original Python implementation.
//!
//! ## Quick Start
//!
//! ```rust
//! use mdc_core::number_parser::{parse_number, get_number};
//!
//! // New API: Returns dual IDs + attributes
//! let parsed = parse_number("[website]SSIS-123.mp4", None).unwrap();
//! assert_eq!(parsed.id, "SSIS-123");           // Display ID
//! assert_eq!(parsed.content_id, "ssis00123");  // Content ID
//!
//! // Legacy API: Returns display ID only (backward compatible)
//! let id = get_number("[website]SSIS-123.mp4", None).unwrap();
//! assert_eq!(id, "SSIS-123");
//! ```
//!
//! ## Dual ID System
//!
//! ### Display ID → Content ID Conversion
//!
//! The conversion follows these rules:
//!
//! 1. **Standard IDs**: `SSIS-123` → `ssis00123`
//!    - Lowercase the prefix
//!    - Remove hyphen
//!    - Zero-pad number to 5 digits
//!
//! 2. **T28/R18 IDs**: `T28-123` → `t2800123`
//!    - Special handling: insert "00" after prefix
//!    - Maintains compatibility with DMM's T28/R18 format
//!
//! 3. **FC2 IDs**: `FC2-PPV-123456` → `fc2-ppv-123456`
//!    - Preserve hyphens (FC2 uses hyphenated format)
//!    - Lowercase only
//!
//! 4. **Tokyo-Hot IDs**: `n1234`, `k5678`
//!    - Keep lowercase prefix
//!    - No zero-padding (Tokyo-Hot uses variable length)
//!
//! 5. **HEYZO IDs**: `HEYZO-1234` → `heyzo-1234`
//!    - 4-digit padding (fixed format)
//!    - Preserve hyphen
//!
//! ### Content ID → Display ID Conversion
//!
//! Reverse conversion for API responses:
//!
//! ```rust
//! use mdc_core::number_parser::{convert_to_display_id, convert_to_content_id};
//!
//! // Standard conversion
//! let display = convert_to_display_id("ssis00123");
//! assert_eq!(display, "SSIS-123");  // Uppercase, hyphen inserted, zeros trimmed
//!
//! // Roundtrip validation
//! let original = "SSIS-123";
//! let content = convert_to_content_id(original);
//! let back = convert_to_display_id(&content);
//! assert_eq!(original, back);
//! ```
//!
//! ## Special Format Handling
//!
//! ### T28/R18 Normalization
//!
//! Filenames with T28/R18 prefixes are normalized during parsing:
//! - `t28123.mp4` → `T28-123` (display) + `t2800123` (content)
//! - `r18-456.mp4` → `R18-456` (display) + `r1800456` (content)
//! - Handles variations: `t28`, `t-28`, `T28`, `T-28`
//!
//! ### Multi-Part Detection
//!
//! Letter suffixes indicate multi-part videos:
//! - `SSIS-123-A.mp4` → ID: `SSIS-123`, Part: 1
//! - `SSIS-123-B.mp4` → ID: `SSIS-123`, Part: 2
//! - Converts A→1, B→2, ..., Y→25
//!
//! ### Attribute Detection
//!
//! Automatically detects special attributes:
//! - **Chinese Subtitles**: `-C` suffix → `cn_sub = true`
//! - **Uncensored**: `-U` or `-UC` suffix → `uncensored = true`
//! - **Special Sites**: `FC2`, `Tokyo-Hot`, etc. → `special_site = Some("fc2")`
//!
//! ## Configuration
//!
//! Use `ParserConfig` for advanced parsing:
//!
//! ```rust
//! use mdc_core::number_parser::{parse_number, ParserConfig};
//! # use anyhow::Result;
//!
//! # fn example() -> Result<()> {
//! let config = ParserConfig {
//!     custom_regexs: vec![r"CUSTOM-(\d+)".to_string()],
//!     removal_strings: vec!["[unwanted]".to_string()],
//!     strict_mode: true,
//!     regex_id_match: 1,      // Capture group for ID
//!     regex_pt_match: 2,      // Capture group for part number
//!     uncensored_prefixes: "".to_string(),
//! };
//!
//! let parsed = parse_number("filename.mp4", Some(&config))?;
//! # Ok(())
//! # }
//! ```
//!
//! ## API Reference
//!
//! ### Primary Functions
//!
//! - **`parse_number(file_path, config)`** - New API, returns `ParsedNumber` with dual IDs
//! - **`get_number(file_path, custom_regexs)`** - Legacy API, returns display ID only
//! - **`convert_to_content_id(display_id)`** - Convert display → content format
//! - **`convert_to_display_id(content_id)`** - Convert content → display format
//!
//! ### When to Use Each Function
//!
//! - Use **`parse_number()`** for new code (full feature set, dual IDs)
//! - Use **`get_number()`** for backward compatibility (legacy code)
//! - Use **`convert_to_*`** when you have IDs from external sources
//!
//! ## Examples
//!
//! ### Basic Parsing
//!
//! ```rust
//! use mdc_core::number_parser::parse_number;
//!
//! let parsed = parse_number("SSIS-123.mp4", None).unwrap();
//! println!("Display: {}", parsed.id);           // "SSIS-123"
//! println!("Content: {}", parsed.content_id);   // "ssis00123"
//! ```
//!
//! ### With Attributes
//!
//! ```rust
//! use mdc_core::number_parser::parse_number;
//!
//! let parsed = parse_number("SSIS-123-C.mp4", None).unwrap();
//! assert!(parsed.attributes.cn_sub);  // Chinese subtitles detected
//! ```
//!
//! ### Multi-Part Videos
//!
//! ```rust
//! use mdc_core::number_parser::parse_number;
//!
//! let parsed = parse_number("SSIS-123-A.mp4", None).unwrap();
//! assert_eq!(parsed.part_number, Some(1));  // Part A = 1
//! ```
//!
//! ### Custom Configuration
//!
//! ```rust
//! use mdc_core::number_parser::{parse_number, ParserConfig};
//! # use anyhow::Result;
//!
//! # fn example() -> Result<()> {
//! let mut config = ParserConfig::default();
//! config.removal_strings.push("[unwanted]".to_string());
//! let parsed = parse_number("[unwanted]SSIS-123.mp4", Some(&config))?;
//! # Ok(())
//! # }
//! ```
//!
//! ## Implementation Notes
//!
//! - Direct port of Python `number_parser.py` with 100% compatibility
//! - All original regex patterns preserved
//! - 70+ test cases ensuring correctness
//! - Zero unsafe code
//! - Efficient regex compilation with `OnceLock`

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

impl ParserConfig {
    /// Create a new ParserConfig with default values
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a custom regex pattern
    ///
    /// # Example
    /// ```
    /// use mdc_core::number_parser::ParserConfig;
    ///
    /// let config = ParserConfig::new()
    ///     .with_custom_regex(r"CUSTOM-\d+");
    /// ```
    pub fn with_custom_regex(mut self, regex: &str) -> Self {
        self.custom_regexs.push(regex.to_string());
        self
    }

    /// Add multiple custom regex patterns
    pub fn with_custom_regexs(mut self, regexs: Vec<String>) -> Self {
        self.custom_regexs.extend(regexs);
        self
    }

    /// Add a removal string
    pub fn with_removal_string(mut self, removal: &str) -> Self {
        self.removal_strings.push(removal.to_string());
        self
    }

    /// Set removal strings (replaces defaults)
    pub fn with_removal_strings(mut self, removals: Vec<String>) -> Self {
        self.removal_strings = removals;
        self
    }

    /// Enable strict mode
    ///
    /// Strict mode uses more conservative matching to reduce false positives
    /// when standard DVD ID format is not detected.
    pub fn with_strict_mode(mut self, enabled: bool) -> Self {
        self.strict_mode = enabled;
        self
    }

    /// Set the capture group index for ID extraction in custom regex
    ///
    /// Default is 1 (first capture group)
    pub fn with_regex_id_match(mut self, index: usize) -> Self {
        self.regex_id_match = index;
        self
    }

    /// Set the capture group index for part number extraction in custom regex
    ///
    /// Default is 2 (second capture group)
    pub fn with_regex_pt_match(mut self, index: usize) -> Self {
        self.regex_pt_match = index;
        self
    }

    /// Set uncensored prefixes (comma-separated)
    pub fn with_uncensored_prefixes(mut self, prefixes: &str) -> Self {
        self.uncensored_prefixes = prefixes.to_string();
        self
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
    // Try to extract Tokyo Hot ID in priority order:
    // 1. Standard format: n1234, k5678, etc.
    // 2. With multiple parts: extract the n-prefixed number first

    // First try: standard n/k/etc prefix with digits
    let re = Regex::new(r"(?i)(cz|gedo|k|n|red-|se)\d{2,6}").ok()?;
    if let Some(m) = re.find(filename) {
        return Some(m.as_str().to_string());
    }

    // Second try: Extract just digits after 'n' or 'k' (for complex formats like th101-060-111077)
    // Look for n + 6 digits (common Tokyo Hot format)
    let re_n = Regex::new(r"(?i)n(\d{6})").ok()?;
    if let Some(caps) = re_n.captures(filename) {
        return Some(format!("n{}", &caps[1]));
    }

    None
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
    re.captures(filename)
        .map(|caps| format!("xxx-av-{}", &caps[1]))
}

fn extract_heydouga(filename: &str) -> Option<String> {
    let re = Regex::new(r"(?i)(\d{4})[-_](\d{3,4})[^\d]*").ok()?;
    re.captures(filename)
        .map(|caps| format!("heydouga-{}-{}", &caps[1], &caps[2]))
}

fn extract_heyzo(filename: &str) -> Option<String> {
    let re = Regex::new(r"(?i)heyzo[^\d]*(\d{4})").ok()?;
    re.captures(filename)
        .map(|caps| format!("HEYZO-{}", &caps[1]))
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

    // T28/R18 special handling: T28-123 → t2800123, R18-456 → r1800456
    // These have digits in the prefix (T28, R18), so handle them specially
    let re_t28r18 = Regex::new(r"^(t-?28|r-?18)[-_]?(\d+)$").unwrap();
    if let Some(caps) = re_t28r18.captures(&lower) {
        let prefix = caps[1].replace("-", ""); // t28 or r18
        let digits = &caps[2];
        let padded = format!("{:0>5}", digits);
        return format!("{}{}", prefix, padded);
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
        // Skip "fc2" prefix and extract only the ID digits
        let after_fc2 = &lower[3..]; // Skip "fc2"
        let digits: String = after_fc2.chars().filter(|c| c.is_numeric()).collect();
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

    // T28/R18 special handling: t2800123 → T28-123, r1800456 → R18-456
    let re_t28r18 = Regex::new(r"^(t28|r18)(\d+)$").unwrap();
    if let Some(caps) = re_t28r18.captures(&lower) {
        let prefix = caps[1].to_uppercase();
        let digits = &caps[2];
        // Trim leading zeros but keep at least 3 digits
        let trimmed = digits.trim_start_matches('0');
        let num_value = trimmed.parse::<usize>().unwrap_or(0);
        let final_digits = format!("{:03}", num_value);
        return format!("{}-{}", prefix, final_digits);
    }

    // Standard format: abc00123 → ABC-123
    let re = Regex::new(r"^([a-z]+)(\d+)([a-z]*)$").unwrap();
    if let Some(caps) = re.captures(&lower) {
        let prefix = caps[1].to_uppercase();
        let digits = &caps[2];
        let suffix = caps
            .get(3)
            .map_or(String::new(), |m| m.as_str().to_uppercase());

        // Trim leading zeros but keep at least 3 digits
        let trimmed = digits.trim_start_matches('0');
        let num_value = trimmed.parse::<usize>().unwrap_or(0);
        let final_digits = format!("{:03}", num_value);

        return format!("{}-{}{}", prefix, final_digits, suffix);
    }

    // Fallback: just uppercase
    content_id.to_uppercase()
}

/// Insert hyphens between alphabetic and numeric parts if not already present
///
/// Automatically normalizes IDs like "SSIS123" to "SSIS-123" to match standard format.
/// This is a Javinizer feature that improves parser flexibility.
///
/// # Arguments
/// * `s` - The string to process
///
/// # Returns
/// String with hyphens inserted between alpha and numeric parts
///
/// # Examples
/// ```
/// # use mdc_core::number_parser::insert_hyphens;
/// assert_eq!(insert_hyphens("SSIS123"), "SSIS-123");
/// assert_eq!(insert_hyphens("ABP1"), "ABP-1");
/// assert_eq!(insert_hyphens("SSIS-123"), "SSIS-123"); // Already has hyphen
/// ```
pub fn insert_hyphens(s: &str) -> String {
    // If already has hyphen between alpha and digit, return as-is
    if s.contains('-') {
        return s.to_string();
    }

    // Tokyo-Hot IDs should remain without hyphens (lowercase letter prefix + digits)
    // Patterns: n1234, k5678, cz1234, red001, gedo123, se456
    let tokyo_hot_re = Regex::new(r"^(?i)(cz|gedo|k|n|red|se)\d{2,4}$").unwrap();
    if tokyo_hot_re.is_match(s) {
        return s.to_string();
    }

    // Pattern: ([A-Za-z]+)(\d+) → $1-$2
    // Insert hyphen between alphabetic prefix and numeric part
    let re = Regex::new(r"^([A-Za-z]+)(\d+)(.*)$").unwrap();
    if let Some(caps) = re.captures(s) {
        let prefix = &caps[1];
        let digits = &caps[2];
        let suffix = caps.get(3).map_or("", |m| m.as_str());
        return format!("{}-{}{}", prefix, digits, suffix);
    }

    // No match, return as-is
    s.to_string()
}

/// Extract part number from letter suffix and return cleaned ID
///
/// Converts multi-part movie letter suffixes to numeric part numbers:
/// - A → 1, B → 2, C → 3, ... Y → 25
/// - Z is excluded (special marker, not a part number)
///
/// # Arguments
/// * `id` - The movie ID to process
///
/// # Returns
/// Tuple of (cleaned_id, optional_part_number)
///
/// # Examples
/// ```
/// # use mdc_core::number_parser::extract_part_from_suffix;
/// let (id, part) = extract_part_from_suffix("SSIS-123-A");
/// assert_eq!(id, "SSIS-123");
/// assert_eq!(part, Some(1));
///
/// let (id2, part2) = extract_part_from_suffix("SSIS-123-B");
/// assert_eq!(id2, "SSIS-123");
/// assert_eq!(part2, Some(2));
///
/// let (id3, part3) = extract_part_from_suffix("SSIS-123-Z");
/// assert_eq!(id3, "SSIS-123-Z"); // Z is not a part marker
/// assert_eq!(part3, None);
/// ```
pub fn extract_part_from_suffix(id: &str) -> (String, Option<u8>) {
    // Pattern 1: With separator: ABC-123-A, XYZ_456_B
    // Match letter suffixes, but EXCLUDE C when separated (C is reserved for Chinese subtitles)
    // Also exclude U (uncensored) and Z (special marker)
    // This allows "-A", "-B", "-D" through "-Y" but not "-C"
    let re = Regex::new(r"^(.+?)[-_]([ABD-TV-Y])$").unwrap(); // A-B, D-T, V-Y (excludes C, U, Z)

    if let Some(caps) = re.captures(id) {
        let base_id = caps[1].to_string();
        let letter = &caps[2];

        // Convert letter to part number: A=1, B=2, D=4, ..., Y=25
        // (C=3 and U=21 are reserved for attributes)
        let part_num = (letter.chars().next().unwrap() as u8) - b'A' + 1;

        return (base_id, Some(part_num));
    }

    // Pattern 2: Without separator (directly attached): ABC123A, AVOP212A, IPZZ077C
    // Only match if ID ends with digit+letter (to avoid false positives)
    // This handles cases like "AVOP-212A" → "AVOP-212" + part 1
    let re_attached = Regex::new(r"^(.+\d)([A-TV-Y])$").unwrap(); // A-T, V-Y (excludes U, Z)
    if let Some(caps) = re_attached.captures(id) {
        let base_id = caps[1].to_string();
        let letter = &caps[2];

        let part_num = (letter.chars().next().unwrap() as u8) - b'A' + 1;

        return (base_id, Some(part_num));
    }

    // Pattern 3: Lowercase with separator (also excludes c for Chinese subtitles)
    let re_lower = Regex::new(r"^(.+?)[-_]([abd-tv-y])$").unwrap(); // Lowercase a-b, d-t, v-y (excludes c, u, z)
    if let Some(caps) = re_lower.captures(id) {
        let base_id = caps[1].to_string();
        let letter = &caps[2];

        // Convert letter to part number: a=1, b=2, d=4, ..., y=25
        let part_num = (letter.chars().next().unwrap() as u8) - b'a' + 1;

        return (base_id, Some(part_num));
    }

    // Pattern 4: Lowercase directly attached
    let re_lower_attached = Regex::new(r"^(.+\d)([a-tv-y])$").unwrap(); // Lowercase A-T, V-Y
    if let Some(caps) = re_lower_attached.captures(id) {
        let base_id = caps[1].to_string();
        let letter = &caps[2];

        let part_num = (letter.chars().next().unwrap() as u8) - b'a' + 1;

        return (base_id, Some(part_num));
    }

    // No letter suffix found, return as-is
    (id.to_string(), None)
}

/// Clean filename by removing website tags, quality markers, and other noise
/// This runs BEFORE number extraction to improve accuracy
///
/// # Arguments
/// * `filename` - The filename to clean
/// * `config` - Optional configuration with custom removal strings
fn clean_filename(filename: &str, config: Option<&ParserConfig>) -> String {
    let mut cleaned = filename.to_string();

    // Apply configurable removal strings FIRST (before other processing)
    // This allows T28/R18 normalization to work on cleaned names
    if let Some(cfg) = config {
        for removal_str in &cfg.removal_strings {
            if !removal_str.is_empty() {
                cleaned = cleaned.replace(removal_str, "");
            }
        }
    }

    // Strip website tags: [xxx.com], [xxx], 【xxx.com】, etc. - EARLY to allow T28/R18 normalization
    // Handles both ASCII brackets [] and fullwidth brackets 【】 (common in Asian watermarks)
    if let Ok(re) = Regex::new(r"[\[【]([^\]】]+)[\]】]") {
        cleaned = re.replace_all(&cleaned, "").to_string();
    }

    // Strip common watermark patterns at the beginning
    // Matches: "domain.com_", "domain.com@", "【domain.com】@", etc.
    // This handles cases like: AVFAP.NET_okp-103.mp4, gg5.co@IPZZ-227-C.mp4
    let watermark_patterns = [
        r"^[\[\【]?[a-z0-9]+\.(com|net|org|co|tv|in|me|cc)[\]\】]?[@_\-\s]+",
        r"^[a-z0-9]+\.(com|net|org|co|tv|in|me|cc)[@_\-\s]+",
    ];

    for pattern in &watermark_patterns {
        if let Ok(re) = Regex::new(&format!("(?i){}", pattern)) {
            cleaned = re.replace(&cleaned, "").to_string();
        }
    }

    // Strip watermark domains in special formats: ses23.com, javhd.com, etc.
    // Pattern: domain.tld followed by hyphen or underscore (common watermark format)
    // Examples: ses23.com-NHDTA-609, javhd.com_IPX-001
    if let Ok(re) = Regex::new(r"^[\w.-]+\.(com|net|tv|la|me|cc|club|jp|xyz|biz|wiki|info|tw|us|de|cn|to)[-_]") {
        cleaned = re.replace_all(&cleaned, "").to_string();
    }

    // Strip numeric date prefixes: 0201-, 20240201-, etc. - EARLY to allow T28/R18 normalization
    // Match 4-8 digits followed by dash/underscore at the start
    if let Ok(re) = Regex::new(r"^\d{4,8}[-_]") {
        cleaned = re.replace_all(&cleaned, "").to_string();
    }

    // Strip Tokyo_Hot_ prefix to allow proper ID extraction
    // Handles: Tokyo_Hot_n1234.mp4, Tokyo-Hot-k5678.mp4, TokyoHot-n1234.mp4
    if let Ok(re) = Regex::new(r"(?i)^tokyo[-_]?hot[-_]") {
        cleaned = re.replace_all(&cleaned, "").to_string();
    }

    // Strip parenthesized quality markers at start: (HD), (FHD), (4K), etc.
    // Fixes: (HD)avop-212A.HD.mp4 → avop-212A.HD.mp4
    if let Ok(re) = Regex::new(r"(?i)^\(?(HD|FHD|4K|1080P|720P|480P|UHD)\)?[-_]?") {
        cleaned = re.replace_all(&cleaned, "").to_string();
    }

    // Normalize T28/R18 prefixes: "t28123" → "T28-123", "t-28-001" → "T28-001", "r18-001" → "R18-001"
    // This handles variations: t28, t-28, T28, T-28, r18, r-18, R18, R-18
    // Javinizer feature: Standardize these studio IDs to consistent format
    if let Ok(re) = Regex::new(r"(?i)^(t-?28|r-?18)[-_]?(\d+)") {
        cleaned = re
            .replace(&cleaned, |caps: &regex::Captures| {
                let prefix = caps[1].to_uppercase().replace("-", "");
                let digits = &caps[2];
                format!("{}-{}", prefix, digits)
            })
            .to_string();
    }

    // Strip email/username prefixes: user@domain@, username@site.com@
    if let Ok(re) = Regex::new(r"^[^@]+@[^@]+@") {
        cleaned = re.replace_all(&cleaned, "").to_string();
    }

    // Strip domain prefixes: domain.com-, site.tv-, etc.
    if let Ok(re) =
        Regex::new(r"^[\w.-]+\.(com|net|tv|la|me|cc|club|jp|xyz|biz|wiki|info|tw|us|de)-")
    {
        cleaned = re.replace_all(&cleaned, "").to_string();
    }

    // Strip part markers EARLY: -1, -2, cd1, cd2, part1, pt2, disc3, etc.
    // Enhanced Javinizer patterns for better multi-part detection
    // This MUST run before quality marker removal so TEST-FHD-CD1 becomes TEST-FHD, then TEST
    // Match before extension or at end
    //
    // Pattern 1: Explicit markers with digits (cd1, part2, pt3, disc4, etc.)
    if let Ok(re) = Regex::new(r"(?i)[-_]?(cd|part|pt|disk|disc)[-_]?\d{1,3}(\.|$)") {
        cleaned = re.replace_all(&cleaned, "$2").to_string();
    }

    // Pattern 2: Standalone part numbers at end (-1, -2, _3, etc.) before extension
    // Only strip if it's clearly a part marker (single digit or -pt\d pattern)
    if let Ok(re) = Regex::new(r"(?i)[-_](pt)?[-_]?([1-9]|1[0-9])(\.|$)") {
        // Only replace if it looks like a part marker, not part of the actual ID
        // Check that there's already content before the part marker
        if cleaned.len() > 5 {
            // Ensure we have an ID before the part marker
            cleaned = re.replace_all(&cleaned, "$3").to_string();
        }
    }

    // Strip quality markers that directly follow digits (e.g., "CZBD-015FULLHD.mp4", "MVSD267.HD Semen")
    // This handles cases like 015FULLHD, 123HD, 456FHD, and quality tags followed by spaces
    // Updated to handle spaces after the quality tag (e.g., "MVSD267.HD Semen Eat.mkv")
    if let Ok(re) = Regex::new(
        r"(?i)(\d)(\.?(?:FULLHD|1080P|720P|480P|H\.?265|H\.?264|X265|X264|HEVC|FHD|4K|UHD|HQ|HD))[\s\.]",
    ) {
        cleaned = re.replace_all(&cleaned, "$1 ").to_string();
    }

    // Strip quality markers that appear after dashes following numbers (after valid codes)
    // Fixes: DSAMBD-18-H265-1080P.mp4 → DSAMBD-18.mp4
    // This runs FIRST to handle multiple consecutive quality markers in one go
    // Only strips when quality marker comes AFTER digits (end of valid code)
    // This prevents stripping "heyzo_hd_1234" where hd is part of the original name
    // IMPORTANT: Longer patterns first to avoid partial matches (H265 before HD, FULLHD before HD, etc.)
    if let Ok(re) = Regex::new(
        r"(?i)(\d+)[-_](H\.?265|H\.?264|FULLHD|1080P|720P|480P|X265|X264|HEVC|FHD|4K|UHD|HD|HQ).*?(\.|$)",
    ) {
        cleaned = re.replace_all(&cleaned, "$1$3").to_string();
    }

    // Strip common quality markers at end using simple string replacement
    // These suffixes are always noise when at the very end
    let quality_suffixes = [
        "-1080P", "_1080P", ".1080P",
        "-720P", "_720P", ".720P",
        "-480P", "_480P", ".480P",
        "-FULLHD", "_FULLHD", ".FULLHD",
        "-FHD", "_FHD", ".FHD",
        "-4K", "_4K", ".4K",
        "-UHD", "_UHD", ".UHD",
        "-HQ", "_HQ", ".HQ",
        "-HD", "_HD", ".HD",
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
            cleaned = cleaned
                .replace("-HD.", ".")
                .replace("_HD.", ".")
                .replace("-hd.", ".")
                .replace("_hd.", ".");
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
    cleaned
        .trim_matches(|c| c == '-' || c == '_' || c == ' ')
        .to_string()
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

    // Replace underscores with hyphens
    result = result.replace('_', "-");

    // Normalize multiple spaces to single space
    // This handles cases like "WANZ-220  TSUBOMI" → "WANZ-220 TSUBOMI"
    result = result.split_whitespace().collect::<Vec<_>>().join(" ");

    // Remove trailing single-word names (actress names, descriptors)
    // Only if they appear AFTER the valid ID format
    // This handles cases like "WANZ-220 TSUBOMI" → "WANZ-220"
    if let Ok(re) = Regex::new(r"(?i)^([A-Z]+-\d+)\s+[A-Z]+$") {
        if let Some(caps) = re.captures(&result) {
            result = caps[1].to_string();
        }
    }

    // Uppercase the result
    result.to_uppercase()
}

/// Check if filename contains a standard DVD ID format
///
/// Standard format: 2-5 letters, hyphen, 2-5 digits (e.g., ABC-123, SSIS-001)
/// Returns true if standard format detected, false otherwise.
fn has_standard_dvd_format(filename: &str) -> bool {
    // Standard JAV DVD ID pattern: [A-Z]{2,5}-\d{2,5}
    let standard_pattern = Regex::new(r"(?i)[A-Z]{2,5}[-_]\d{2,5}").unwrap();
    standard_pattern.is_match(filename)
}

/// Apply strict mode filtering to extracted ID
///
/// In strict mode, only allow IDs that match conservative patterns
/// to reduce false positives for non-standard filenames.
fn apply_strict_mode_filter(id: &str) -> bool {
    let upper_id = id.to_uppercase();

    // Allow standard DVD format: ABC-123, SSIS-001, etc.
    // 2-5 letters, hyphen, 2-5 digits, optional suffix letter
    let standard_pattern = Regex::new(r"^[A-Z]{2,5}-\d{2,5}[A-Z]?$").unwrap();
    if standard_pattern.is_match(&upper_id) {
        return true;
    }

    // Allow T28/R18 special formats: T28-123, R18-456
    let t28_r18_pattern = Regex::new(r"^(T28|R18)-\d{2,5}$").unwrap();
    if t28_r18_pattern.is_match(&upper_id) {
        return true;
    }

    // Allow Tokyo-Hot IDs: k0123, n1234, etc. (lowercase)
    let tokyo_hot_pattern = Regex::new(r"^(cz|gedo|k|n|red|se)\d{2,4}$").unwrap();
    if tokyo_hot_pattern.is_match(id) {
        return true;
    }

    // Allow IDs without hyphens that follow standard pattern (e.g., BEB077, SNIS091)
    // These will be normalized to ABC-123 format later
    let no_hyphen_pattern = Regex::new(r"^[A-Z]{2,5}\d{2,5}[A-Z]?$").unwrap();
    if no_hyphen_pattern.is_match(&upper_id) {
        return true;
    }

    // Allow pure numeric IDs (e.g., FC2 numbers: 1234567)
    // Some sites use pure numbers
    let numeric_pattern = Regex::new(r"^\d{5,}$").unwrap();
    if numeric_pattern.is_match(id) {
        return true;
    }

    // Reject everything else in strict mode
    false
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
    let cleaned_filepath = clean_filename(filepath, Some(config));

    // Try custom regexes first (on cleaned filename) with capture group support
    if !config.custom_regexs.is_empty() {
        for regex_str in &config.custom_regexs {
            if let Ok(re) = Regex::new(regex_str) {
                if let Some(caps) = re.captures(&cleaned_filepath) {
                    // Extract ID from configured capture group
                    let id = caps
                        .get(config.regex_id_match)
                        .map(|m| m.as_str().to_string())
                        .or_else(|| caps.get(0).map(|m| m.as_str().to_string()))
                        .unwrap_or_default();

                    // Extract part number if configured
                    let part = caps
                        .get(config.regex_pt_match)
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

    // Apply strict mode filtering if enabled or auto-activated
    // Auto-strict: activates when standard DVD format not detected
    let should_use_strict = config.strict_mode || !has_standard_dvd_format(&cleaned_filepath);

    if should_use_strict && !apply_strict_mode_filter(&base_id) {
        // In strict mode, reject IDs that don't match conservative patterns
        // This reduces false positives for non-standard filenames
        return Err(anyhow!(
            "Strict mode: extracted ID '{}' does not match standard DVD format",
            base_id
        ));
    }

    // Extract part number from letter suffix FIRST (A→1, B→2, etc.)
    // This must happen before attribute detection to handle conflicts:
    // - "SSIS-123-C" is part 3, not Chinese subtitles
    // - "SSIS-123-UC-A" is uncensored + part 1
    let (id_without_part, part_number) = extract_part_from_suffix(&base_id);

    // Detect attributes from suffix (after part extraction)
    let mut attrs = ParsedAttributes::default();

    // Check for -C suffix (Chinese subtitles) - only multi-char patterns to avoid conflict with part C
    // Accept: -CH, -CHN, -CHS, -CHT, -CN_SUB, etc.
    attrs.cn_sub = Regex::new(r"(?i)[-_](ch|chn|chs|cht|cn[-_]?sub)$")
        .unwrap()
        .is_match(&id_without_part)
        || Regex::new(r"(?i)[-_]c$")
            .unwrap()
            .is_match(&id_without_part);

    // Check for -U or -UC suffix (uncensored)
    attrs.uncensored = Regex::new(r"(?i)[-_]u(c)?$")
        .unwrap()
        .is_match(&id_without_part);

    // Strip the suffix to get clean ID
    let clean_id = strip_suffix(&id_without_part);

    // Normalize ID by inserting hyphens if needed (e.g., "MVSD267" → "MVSD-267")
    // This improves compatibility with metadata sources that expect standard format
    let normalized_id = insert_hyphens(&clean_id);

    // Part number already extracted above
    let final_id = normalized_id;

    let content_id = convert_to_content_id(&final_id);

    Ok(ParsedNumber {
        id: final_id,
        content_id,
        part_number,
        attributes: attrs,
    })
}

/// Clean up extracted ID by removing trailing Japanese text and English descriptions
/// This handles cases like "AVOP-212-kawaii_10周年SPECIAL企画" → "AVOP-212"
/// and "WANZ-220  TSUBOMI" → "WANZ-220"
/// Note: Attribute suffixes (-C, -U, -UC) are handled by later processing stages
fn cleanup_extracted_id(mut extracted_id: String) -> String {
    // If we can extract a clean DVD ID (letters-digits), keep just that
    // This handles "AVOP-212-kawaii_10周年" → "AVOP-212" by extracting the core pattern
    if let Ok(re) = Regex::new(r"^([A-Z]{2,5}[-_]\d{2,5})[-_]") {
        if let Some(caps) = re.captures(&extracted_id.to_uppercase()) {
            // Found a standard ID pattern, check if there's junk after it
            let core_id = caps[1].to_string();
            let rest = &extracted_id[core_id.len()..];

            // If the rest contains Japanese/Chinese or looks like junk, keep just the core ID
            // Junk patterns:
            // 1. Japanese text anywhere
            // 2. Lowercase descriptions (kawaii, etc.)
            // 3. Single-letter + delimiter + 2+ chars (C_GG5, C-GG5) - watermark artifacts
            // Note: We preserve short suffixes like -UC, -C (2 chars) as they're likely attributes
            if Regex::new(r"[\p{Han}\p{Hiragana}\p{Katakana}]").unwrap().is_match(rest)
                || Regex::new(r"(?i)^[-_][a-z]{2,}").unwrap().is_match(rest)
                || Regex::new(r"(?i)^[-_][A-Z][-_][A-Z0-9]{2,}").unwrap().is_match(rest) {
                return core_id;
            }
        }
    }

    // Also remove trailing English descriptions after ID when separated by space/dot
    // This handles cases like "WANZ-220  TSUBOMI" or "MVSD267.HD Semen"
    // Only matches if there's a digit followed by spaces/dots and then 2+ English letters
    if let Ok(re) = Regex::new(r"(?i)(\d)[\s\.]+[a-z]{2,}.*$") {
        extracted_id = re.replace_all(&extracted_id, "$1").to_string();
    }

    // Trim any trailing/leading whitespace
    extracted_id.trim().to_string()
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
            return Ok(cleanup_extracted_id(before_dot.trim().to_string()));
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
                        return Ok(cleanup_extracted_id(m.as_str().to_string()));
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

        return Ok(cleanup_extracted_id(file_number.to_string()));
    }

    // Handle filenames without - (FANZA CID, Western formats)
    // Use cleaned version for these checks too

    // Western format: xxx.YY.MM.DD
    if let Ok(re) = Regex::new(r"[a-zA-Z]+\.\d{2}\.\d{2}\.\d{2}") {
        if let Some(m) = re.find(cleaned_filepath) {
            return Ok(cleanup_extracted_id(m.as_str().to_string()));
        }
    }

    // Extract filename before extension (from cleaned version)
    if let Some(dot_pos) = cleaned_filepath.rfind('.') {
        let before_dot = &cleaned_filepath[..dot_pos];
        let cleaned = before_dot.replace('_', "-");
        return Ok(cleanup_extracted_id(cleaned));
    }

    // If cleaned version has no extension, just return it
    if !cleaned_filepath.is_empty() {
        return Ok(cleanup_extracted_id(cleaned_filepath.replace('_', "-")));
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
    let cleaned_filepath = clean_filename(filepath, None);

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
        assert_eq!(
            get_number("carib-123456-789.mp4", None).unwrap(),
            "123456-789"
        );
        assert_eq!(
            get_number("caribbeancom-123456_789.mp4", None).unwrap(),
            "123456-789"
        );
    }

    #[test]
    fn test_get_number_1pon() {
        assert_eq!(
            get_number("1pondo_123456_789.mp4", None).unwrap(),
            "123456_789"
        );
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
        assert_eq!(
            get_number("x-art.18.05.15.mp4", None).unwrap(),
            "x-art.18.05.15"
        );
    }

    // New tests for filename cleaning functionality

    #[test]
    fn test_clean_website_tags() {
        // Test various website tag patterns
        assert_eq!(
            get_number("[Thz.la]jufd-643.mp4", None).unwrap(),
            "JUFD-643"
        );
        assert_eq!(
            get_number("[7sht.me]SSIS-123.mp4", None).unwrap(),
            "SSIS-123"
        );
        assert_eq!(
            get_number("[ses23.com]ABC-456.avi", None).unwrap(),
            "ABC-456"
        );
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
        // Note: BEB077 is automatically normalized to BEB-077 for better metadata matching
        assert_eq!(get_number("jp.myav.tv-BEB077.avi", None).unwrap(), "BEB-077");
        assert_eq!(
            get_number("www.site.com-ABC-123.mp4", None).unwrap(),
            "ABC-123"
        );
        // With dash in original, it's preserved
        assert_eq!(
            get_number("jp.myav.tv-BEB-077.avi", None).unwrap(),
            "BEB-077"
        );
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
        assert_eq!(
            get_number("[Thz.la]jufd-643-FHD.mp4", None).unwrap(),
            "JUFD-643"
        );
        assert_eq!(
            get_number("[site.com]ABC-123-1080P-C.mp4", None).unwrap(),
            "ABC-123"
        );
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
        assert_eq!(
            get_number("[Thz.la]jufd-643.mp4", None).unwrap(),
            "JUFD-643"
        );

        let result1 = get_number("roger92402094@www.sexinsex.net@AVGL-012.avi", None).unwrap();
        assert_eq!(result1, "AVGL-012");

        // BEB077 without dash stays as BEB077
        assert_eq!(get_number("jp.myav.tv-BEB077.avi", None).unwrap(), "BEB-077");
        assert_eq!(get_number("CZBD-015FULLHD.mp4", None).unwrap(), "CZBD-015");
    }

    #[test]
    fn test_clean_filename_function() {
        // Direct tests of the clean_filename function
        assert_eq!(clean_filename("[Thz.la]jufd-643", None), "jufd-643");
        assert_eq!(clean_filename("site.com-ABC-123", None), "ABC-123");
        assert_eq!(clean_filename("MOVIE-1080P", None), "MOVIE");
        // TEST-FHD-CD1: FHD is stripped as quality marker, then CD1 is stripped as part marker
        assert_eq!(clean_filename("TEST-FHD-CD1", None), "TEST");
        assert_eq!(clean_filename("MOVIE-PART2", None), "MOVIE");
        // Date prefix stripping
        assert_eq!(clean_filename("0201-SNIS091", None), "SNIS091");
        assert_eq!(clean_filename("20240201-ABC-123", None), "ABC-123");
    }

    #[test]
    fn test_date_prefix_stripping() {
        // Test files with date prefixes
        assert_eq!(get_number("0201-SNIS-091.mp4", None).unwrap(), "SNIS-091");
        // Without dash in code - the date prefix should be stripped
        let result = get_number("0201-SNIS091.mp4", None).unwrap();
        println!("Extracted from '0201-SNIS091.mp4': {}", result);
        // Should extract SNIS091, not 0201-SNIS091
        assert!(
            !result.contains("0201"),
            "Should not contain date prefix '0201'"
        );
        assert!(result.contains("SNIS"), "Should contain 'SNIS'");
    }

    #[test]
    fn test_parentheses_quality_markers() {
        // Priority 1: Strip parenthesized quality markers at start
        // Note: A suffix is now stripped as multi-disc marker
        assert_eq!(
            get_number("(HD)avop-212A.HD.mp4", None).unwrap(),
            "AVOP-212" // A suffix stripped for multi-disc support
        );

        // Verify part_number is extracted correctly
        let parsed = parse_number("(HD)avop-212A.HD.mp4", None).unwrap();
        assert_eq!(parsed.id, "AVOP-212");
        assert_eq!(parsed.part_number, Some(1)); // A = part 1

        assert_eq!(get_number("(FHD)ABC-123.mp4", None).unwrap(), "ABC-123");
        assert_eq!(get_number("(4K)XYZ-456.mp4", None).unwrap(), "XYZ-456");
        assert_eq!(get_number("(1080P)TEST-001.mkv", None).unwrap(), "TEST-001");
    }

    #[test]
    fn test_japanese_text_stripping() {
        // Priority 2: Strip Japanese/Chinese characters (actress names, descriptions)
        // Space + Japanese should be stripped
        assert_eq!(
            get_number("hnd-809 神宮寺ナオ.mp4", None).unwrap(),
            "HND-809"
        );
        assert_eq!(
            get_number("ABC-123 波多野結衣.mp4", None).unwrap(),
            "ABC-123"
        );
        assert_eq!(
            get_number("SSIS-001 明日花キララ.mp4", None).unwrap(),
            "SSIS-001"
        );
        // Full-width space (　) should also be handled
        assert_eq!(
            get_number("IPX-456　桜空もも.mp4", None).unwrap(),
            "IPX-456"
        );
    }

    #[test]
    fn test_quality_markers_mid_string() {
        // Priority 3: Strip quality markers that appear after dashes mid-string
        assert_eq!(
            get_number("DSAMBD-18-H265-1080P.mp4", None).unwrap(),
            "DSAMBD-18"
        );
        assert_eq!(get_number("ABC-123-FHD-720P.mp4", None).unwrap(), "ABC-123");
        assert_eq!(get_number("XYZ-456-X264-HD.mkv", None).unwrap(), "XYZ-456");
        assert_eq!(
            get_number("TEST-001-HEVC-4K.mp4", None).unwrap(),
            "TEST-001"
        );
    }

    // ===== Dual ID Conversion Tests (Phase 1) =====

    #[test]
    fn test_convert_to_content_id_standard() {
        // Standard JAV format: ABC-123 → abc00123
        assert_eq!(convert_to_content_id("SSIS-123"), "ssis00123");
        assert_eq!(convert_to_content_id("ABP-1"), "abp00001");
        assert_eq!(convert_to_content_id("MIDE-999"), "mide00999");
        assert_eq!(convert_to_content_id("IPX-456"), "ipx00456");
        assert_eq!(convert_to_content_id("STARS-12345"), "stars12345");
    }

    #[test]
    fn test_convert_to_content_id_with_suffix() {
        // IDs with alphabetic suffixes
        assert_eq!(convert_to_content_id("IPX-123A"), "ipx00123a");
        assert_eq!(convert_to_content_id("SSIS-456Z"), "ssis00456z");
        assert_eq!(convert_to_content_id("ABP-789E"), "abp00789e");
    }

    #[test]
    fn test_convert_to_content_id_fc2() {
        // FC2 special handling
        assert_eq!(convert_to_content_id("FC2-PPV-1234567"), "fc2ppv1234567");
        assert_eq!(convert_to_content_id("FC2-1234567"), "fc21234567");
        assert_eq!(convert_to_content_id("fc2-ppv-123"), "fc2ppv123");
    }

    #[test]
    fn test_convert_to_content_id_heyzo() {
        // HEYZO special handling
        assert_eq!(convert_to_content_id("HEYZO-1234"), "heyzo01234");
        assert_eq!(convert_to_content_id("HEYZO-12"), "heyzo00012");
        assert_eq!(convert_to_content_id("heyzo-999"), "heyzo00999");
    }

    #[test]
    fn test_convert_to_content_id_tokyo_hot() {
        // Tokyo-Hot IDs remain as-is (already in content format)
        assert_eq!(convert_to_content_id("n1234"), "n1234");
        assert_eq!(convert_to_content_id("k0123"), "k0123");
        assert_eq!(convert_to_content_id("red123"), "red123");
    }

    #[test]
    fn test_convert_to_display_id_standard() {
        // Standard content ID to display: abc00123 → ABC-123
        assert_eq!(convert_to_display_id("ssis00123"), "SSIS-123");
        assert_eq!(convert_to_display_id("abp00001"), "ABP-001");
        assert_eq!(convert_to_display_id("mide00999"), "MIDE-999");
        assert_eq!(convert_to_display_id("ipx00012"), "IPX-012");
    }

    #[test]
    fn test_convert_to_display_id_zero_trimming() {
        // Leading zeros trimmed but minimum 3 digits preserved
        assert_eq!(convert_to_display_id("ssis00123"), "SSIS-123");
        assert_eq!(convert_to_display_id("abp00001"), "ABP-001");
        assert_eq!(convert_to_display_id("mide00099"), "MIDE-099");
        assert_eq!(convert_to_display_id("ipx00012"), "IPX-012");
    }

    #[test]
    fn test_convert_to_display_id_fc2() {
        // FC2 special handling
        assert_eq!(convert_to_display_id("fc2ppv1234567"), "FC2-PPV-1234567");
        assert_eq!(convert_to_display_id("fc21234567"), "FC2-1234567");
    }

    #[test]
    fn test_convert_to_display_id_heyzo() {
        // HEYZO special handling
        assert_eq!(convert_to_display_id("heyzo01234"), "HEYZO-1234");
        assert_eq!(convert_to_display_id("heyzo00012"), "HEYZO-12");
    }

    #[test]
    fn test_convert_to_display_id_tokyo_hot() {
        // Tokyo-Hot IDs remain lowercase without hyphen
        assert_eq!(convert_to_display_id("n1234"), "n1234");
        assert_eq!(convert_to_display_id("k0123"), "k123");
        assert_eq!(convert_to_display_id("n01234"), "n1234");
    }

    #[test]
    fn test_roundtrip_conversion_standard() {
        // Display → Content → Display should be identical
        let ids = vec!["SSIS-123", "ABP-001", "MIDE-999", "IPX-456", "STARS-001"];
        for id in ids {
            let content_id = convert_to_content_id(id);
            let back = convert_to_display_id(&content_id);
            assert_eq!(id, back, "Roundtrip failed for {}", id);
        }
    }

    #[test]
    fn test_roundtrip_conversion_special() {
        // Special formats roundtrip
        let ids = vec![
            ("HEYZO-1234", "HEYZO-1234"),
            ("FC2-PPV-1234567", "FC2-PPV-1234567"),
        ];
        for (input, expected) in ids {
            let content_id = convert_to_content_id(input);
            let back = convert_to_display_id(&content_id);
            assert_eq!(expected, back, "Roundtrip failed for {}", input);
        }
    }

    #[test]
    fn test_parse_number_dual_ids() {
        // parse_number() should return both IDs
        let result = parse_number("SSIS-123.mp4", None).unwrap();
        assert_eq!(result.id, "SSIS-123");
        assert_eq!(result.content_id, "ssis00123");
        assert_eq!(result.part_number, None);
    }

    #[test]
    fn test_parse_number_fc2_dual_ids() {
        let result = parse_number("FC2-PPV-1234567.mp4", None).unwrap();
        assert_eq!(result.id, "FC2-PPV-1234567");
        assert_eq!(result.content_id, "fc2ppv1234567");
    }

    #[test]
    fn test_parse_number_heyzo_dual_ids() {
        let result = parse_number("HEYZO-1234.mp4", None).unwrap();
        assert_eq!(result.id, "HEYZO-1234");
        assert_eq!(result.content_id, "heyzo01234");
    }

    #[test]
    fn test_parse_number_tokyo_hot_dual_ids() {
        let result = parse_number("tokyo-hot-n1234.mp4", None).unwrap();
        assert_eq!(result.id, "n1234");
        assert_eq!(result.content_id, "n1234");
        // Note: special_site detection depends on get_number_by_dict matching
        // After Tokyo_Hot_ prefix stripping, the detection path may vary
        // The important part is that ID extraction works correctly
    }

    #[test]
    fn test_parse_number_carib_dual_ids() {
        let result = parse_number("carib-123456-789.mp4", None).unwrap();
        assert_eq!(result.id, "123456-789");
        assert_eq!(result.attributes.special_site, Some("carib".to_string()));
    }

    #[test]
    fn test_parse_number_attributes_cn_sub() {
        let result = parse_number("SSIS-123-C.mp4", None).unwrap();
        assert_eq!(result.id, "SSIS-123");
        assert_eq!(result.attributes.cn_sub, true);
    }

    #[test]
    fn test_parse_number_attributes_uncensored() {
        let result = parse_number("SSIS-123-U.mp4", None).unwrap();
        assert_eq!(result.id, "SSIS-123");
        assert_eq!(result.attributes.uncensored, true);

        let result2 = parse_number("ABP-456-UC.mp4", None).unwrap();
        assert_eq!(result2.id, "ABP-456");
        assert_eq!(result2.attributes.uncensored, true);
    }

    #[test]
    fn test_parse_number_with_config() {
        let mut config = ParserConfig::default();
        // Use regex without capture groups, or set regex_id_match to 0 for whole match
        config.custom_regexs = vec![r"CUSTOM-\d+".to_string()];

        let result = parse_number("prefix_CUSTOM-999_suffix.mp4", Some(&config)).unwrap();
        assert_eq!(result.id, "CUSTOM-999");
    }

    #[test]
    fn test_parse_number_with_capture_groups() {
        let mut config = ParserConfig::default();
        // Regex with capture groups - group 0 is whole match, group 1 is digits
        config.custom_regexs = vec![r"(CUSTOM-\d+)".to_string()];
        config.regex_id_match = 1; // Extract from capture group 1

        let result = parse_number("prefix_CUSTOM-999_suffix.mp4", Some(&config)).unwrap();
        assert_eq!(result.id, "CUSTOM-999");
    }

    #[test]
    fn test_parsed_number_display() {
        // Test Display trait implementation
        let parsed = ParsedNumber::from_id("SSIS-123".to_string());
        assert_eq!(format!("{}", parsed), "SSIS-123");
    }

    #[test]
    fn test_parsed_number_into_string() {
        // Test From<ParsedNumber> for String
        let parsed = ParsedNumber::from_id("SSIS-123".to_string());
        let s: String = parsed.into();
        assert_eq!(s, "SSIS-123");
    }

    #[test]
    fn test_content_id_edge_cases() {
        // Very long numbers
        assert_eq!(convert_to_content_id("STARS-123456"), "stars123456");

        // Single digit
        assert_eq!(convert_to_content_id("TEST-1"), "test00001");

        // Underscore instead of hyphen
        assert_eq!(convert_to_content_id("SSIS_123"), "ssis00123");
    }

    #[test]
    fn test_display_id_edge_cases() {
        // Already padded to more than 5
        assert_eq!(convert_to_display_id("stars123456"), "STARS-123456");

        // Minimum padding (3 digits)
        assert_eq!(convert_to_display_id("test00001"), "TEST-001");

        // Large numbers
        assert_eq!(convert_to_display_id("ssis99999"), "SSIS-99999");
    }

    #[test]
    fn test_backward_compatibility_get_number() {
        // get_number() should still work exactly as before but use parse_number() internally
        assert_eq!(get_number("SSIS-123.mp4", None).unwrap(), "SSIS-123");
        assert_eq!(get_number("ABP-001.avi", None).unwrap(), "ABP-001");
        assert_eq!(
            get_number("FC2-PPV-1234567.mp4", None).unwrap(),
            "FC2-PPV-1234567"
        );

        // Custom regex still works
        let custom = r"CUSTOM-\d+";
        assert_eq!(
            get_number("prefix_CUSTOM-999.mp4", Some(custom)).unwrap(),
            "CUSTOM-999"
        );
    }

    // ===== Phase 2 Enhancement Tests =====

    #[test]
    fn test_t28_normalization() {
        // T28 prefix normalization: t28, t-28, T28, T-28 → T28-XXX
        assert_eq!(get_number("t28123.mp4", None).unwrap(), "T28-123");
        assert_eq!(get_number("t-28-001.mp4", None).unwrap(), "T28-001");
        assert_eq!(get_number("T28-456.mp4", None).unwrap(), "T28-456");
        assert_eq!(get_number("T-28789.mp4", None).unwrap(), "T28-789");

        // With quality markers
        assert_eq!(get_number("t28123-1080P.mp4", None).unwrap(), "T28-123");
        assert_eq!(get_number("t-28-001-FHD.mkv", None).unwrap(), "T28-001");
    }

    #[test]
    fn test_r18_normalization() {
        // R18 prefix normalization: r18, r-18, R18, R-18 → R18-XXX
        assert_eq!(get_number("r18001.mp4", None).unwrap(), "R18-001");
        assert_eq!(get_number("r-18-123.mp4", None).unwrap(), "R18-123");
        assert_eq!(get_number("R18-456.mp4", None).unwrap(), "R18-456");
        assert_eq!(get_number("R-18789.mp4", None).unwrap(), "R18-789");

        // With quality markers
        assert_eq!(get_number("r18001-HD.mp4", None).unwrap(), "R18-001");
        assert_eq!(get_number("r-18-123-720P.mkv", None).unwrap(), "R18-123");
    }

    #[test]
    fn test_insert_hyphens_function() {
        // Test the insert_hyphens() function directly
        assert_eq!(insert_hyphens("SSIS123"), "SSIS-123");
        assert_eq!(insert_hyphens("ABP1"), "ABP-1");
        assert_eq!(insert_hyphens("IPX456Z"), "IPX-456Z");

        // Already has hyphen - no change
        assert_eq!(insert_hyphens("SSIS-123"), "SSIS-123");
        assert_eq!(insert_hyphens("ABP-1"), "ABP-1");

        // With suffixes
        assert_eq!(insert_hyphens("SSIS123A"), "SSIS-123A");
        assert_eq!(insert_hyphens("ABP999Z"), "ABP-999Z");

        // No alphabetic prefix - return as-is
        assert_eq!(insert_hyphens("12345"), "12345");

        // No numeric part - return as-is
        assert_eq!(insert_hyphens("SSIS"), "SSIS");
    }

    #[test]
    fn test_configurable_removal_strings() {
        // Test configurable removal strings via ParserConfig
        let mut config = ParserConfig::default();
        config.removal_strings = vec![
            "CUSTOM".to_string(),
            "TAG".to_string(),
            "-SPECIAL".to_string(),
        ];

        let result = parse_number("CUSTOMSSIS-123.mp4", Some(&config)).unwrap();
        assert_eq!(result.id, "SSIS-123");

        let result2 = parse_number("TAGIPM-456.mp4", Some(&config)).unwrap();
        assert_eq!(result2.id, "IPM-456");

        let result3 = parse_number("ABP-789-SPECIAL.mp4", Some(&config)).unwrap();
        assert_eq!(result3.id, "ABP-789");
    }

    #[test]
    fn test_t28_r18_with_parse_number() {
        // T28/R18 normalization through parse_number()
        let result = parse_number("t28123.mp4", None).unwrap();
        assert_eq!(result.id, "T28-123");
        assert_eq!(result.content_id, "t2800123");

        let result2 = parse_number("r-18-456.mp4", None).unwrap();
        assert_eq!(result2.id, "R18-456");
        assert_eq!(result2.content_id, "r1800456");
    }

    #[test]
    fn test_clean_filename_t28_r18() {
        // Direct clean_filename tests for T28/R18
        assert_eq!(clean_filename("t28123", None), "T28-123");
        assert_eq!(clean_filename("t-28-001", None), "T28-001");
        assert_eq!(clean_filename("r18456", None), "R18-456");
        assert_eq!(clean_filename("r-18-789", None), "R18-789");
    }

    #[test]
    fn test_clean_filename_with_config() {
        // Test clean_filename with configurable removal strings
        let mut config = ParserConfig::default();
        config.removal_strings = vec!["UNWANTED".to_string(), "-JUNK".to_string()];

        assert_eq!(
            clean_filename("UNWANTEDSSIS-123", Some(&config)),
            "SSIS-123"
        );
        assert_eq!(clean_filename("ABP-456-JUNK", Some(&config)), "ABP-456");
        assert_eq!(
            clean_filename("UNWANTEDIPM-789-JUNK", Some(&config)),
            "IPM-789"
        );
    }

    #[test]
    fn test_multiple_phase2_features() {
        // Test multiple Phase 2 features working together
        let mut config = ParserConfig::default();
        config.removal_strings = vec!["SITE".to_string()];

        // T28 normalization + removal string
        let result = parse_number("SITEt28123.mp4", Some(&config)).unwrap();
        assert_eq!(result.id, "T28-123");

        // R18 normalization + removal string + quality marker
        let result2 = parse_number("SITEr18-456-HD.mp4", Some(&config)).unwrap();
        assert_eq!(result2.id, "R18-456");
    }

    #[test]
    fn test_t28_r18_edge_cases() {
        // Edge cases for T28/R18 normalization

        // With website tags
        assert_eq!(get_number("[site.com]t28123.mp4", None).unwrap(), "T28-123");
        assert_eq!(get_number("[xxx]r18-456.mp4", None).unwrap(), "R18-456");

        // With date prefixes
        assert_eq!(get_number("20240101-t28123.mp4", None).unwrap(), "T28-123");
        assert_eq!(get_number("0201-r18-456.mp4", None).unwrap(), "R18-456");

        // With part markers
        assert_eq!(get_number("t28123-cd1.mp4", None).unwrap(), "T28-123");
        assert_eq!(get_number("r18-456-part2.mp4", None).unwrap(), "R18-456");
    }

    #[test]
    fn test_removal_strings_empty_handling() {
        // Test that empty removal strings are ignored
        let mut config = ParserConfig::default();
        config.removal_strings = vec!["".to_string(), "VALID".to_string(), "".to_string()];

        let result = parse_number("VALIDSSIS-123.mp4", Some(&config)).unwrap();
        assert_eq!(result.id, "SSIS-123");
    }

    // ===== Phase 3 Multi-Part Detection Tests =====

    #[test]
    fn test_extract_part_from_suffix_uppercase() {
        // Test letter suffix extraction - uppercase
        let (id, part) = extract_part_from_suffix("SSIS-123-A");
        assert_eq!(id, "SSIS-123");
        assert_eq!(part, Some(1));

        let (id2, part2) = extract_part_from_suffix("ABP-456-B");
        assert_eq!(id2, "ABP-456");
        assert_eq!(part2, Some(2));

        // C is reserved for Chinese subtitles, not treated as part 3
        let (id3, part3) = extract_part_from_suffix("IPX-789-C");
        assert_eq!(id3, "IPX-789-C"); // C not extracted
        assert_eq!(part3, None);

        // D is part 4
        let (id4, part4) = extract_part_from_suffix("IPX-789-D");
        assert_eq!(id4, "IPX-789");
        assert_eq!(part4, Some(4));

        // Y is the last valid part marker (25)
        let (id5, part5) = extract_part_from_suffix("MIDE-001-Y");
        assert_eq!(id5, "MIDE-001");
        assert_eq!(part5, Some(25));
    }

    #[test]
    fn test_extract_part_from_suffix_lowercase() {
        // Test letter suffix extraction - lowercase
        let (id, part) = extract_part_from_suffix("ssis-123-a");
        assert_eq!(id, "ssis-123");
        assert_eq!(part, Some(1));

        let (id2, part2) = extract_part_from_suffix("abp-456-b");
        assert_eq!(id2, "abp-456");
        assert_eq!(part2, Some(2));
    }

    #[test]
    fn test_extract_part_from_suffix_z_excluded() {
        // Z is a special marker, not a part number
        let (id, part) = extract_part_from_suffix("SSIS-123-Z");
        assert_eq!(id, "SSIS-123-Z"); // Z not stripped
        assert_eq!(part, None);

        let (id2, part2) = extract_part_from_suffix("ABP-456-z");
        assert_eq!(id2, "ABP-456-z"); // lowercase z also not stripped
        assert_eq!(part2, None);
    }

    #[test]
    fn test_extract_part_from_suffix_underscore() {
        // Test with underscore separator
        let (id, part) = extract_part_from_suffix("SSIS-123_A");
        assert_eq!(id, "SSIS-123");
        assert_eq!(part, Some(1));

        let (id2, part2) = extract_part_from_suffix("ABP_456_B");
        assert_eq!(id2, "ABP_456");
        assert_eq!(part2, Some(2));
    }

    #[test]
    fn test_extract_part_from_suffix_no_suffix() {
        // No letter suffix
        let (id, part) = extract_part_from_suffix("SSIS-123");
        assert_eq!(id, "SSIS-123");
        assert_eq!(part, None);

        let (id2, part2) = extract_part_from_suffix("FC2-PPV-1234567");
        assert_eq!(id2, "FC2-PPV-1234567");
        assert_eq!(part2, None);
    }

    #[test]
    fn test_parse_number_with_letter_suffix() {
        // Test parse_number extracts part numbers from letter suffixes
        let result = parse_number("SSIS-123-A.mp4", None).unwrap();
        assert_eq!(result.id, "SSIS-123");
        assert_eq!(result.part_number, Some(1));

        let result2 = parse_number("ABP-456-B.mkv", None).unwrap();
        assert_eq!(result2.id, "ABP-456");
        assert_eq!(result2.part_number, Some(2));

        let result3 = parse_number("IPX-789-D.avi", None).unwrap();
        assert_eq!(result3.id, "IPX-789");
        assert_eq!(result3.part_number, Some(4)); // D = 4 (C=3 reserved for attributes)
    }

    #[test]
    fn test_parse_number_letter_suffix_with_attributes() {
        // Test that letter suffix works with other attributes
        // e.g., "SSIS-123-UC-A" → ID="SSIS-123", uncensored=true, part=1
        let result = parse_number("SSIS-123-U-A.mp4", None).unwrap();
        assert_eq!(result.id, "SSIS-123");
        assert_eq!(result.part_number, Some(1));
        assert_eq!(result.attributes.uncensored, true);

        let result2 = parse_number("ABP-456-C-B.mp4", None).unwrap();
        assert_eq!(result2.id, "ABP-456");
        assert_eq!(result2.part_number, Some(2));
        assert_eq!(result2.attributes.cn_sub, true);
    }

    #[test]
    fn test_enhanced_part_markers() {
        // Test enhanced part marker stripping with more flexible patterns
        assert_eq!(get_number("SSIS-123-cd1.mp4", None).unwrap(), "SSIS-123");
        assert_eq!(get_number("ABP-456-cd2.mp4", None).unwrap(), "ABP-456");
        assert_eq!(get_number("IPX-789-part3.mp4", None).unwrap(), "IPX-789");
        assert_eq!(get_number("MIDE-001-pt10.mp4", None).unwrap(), "MIDE-001");
        assert_eq!(
            get_number("STARS-100-disc15.mp4", None).unwrap(),
            "STARS-100"
        );
    }

    #[test]
    fn test_enhanced_part_markers_standalone() {
        // Test standalone part numbers (but only when there's already an ID)
        assert_eq!(get_number("SSIS-123-1.mp4", None).unwrap(), "SSIS-123");
        assert_eq!(get_number("ABP-456-2.mp4", None).unwrap(), "ABP-456");
        assert_eq!(get_number("IPX-789_3.mp4", None).unwrap(), "IPX-789");
    }

    #[test]
    fn test_letter_suffix_edge_cases() {
        // Test edge cases for letter suffix detection

        // Multiple letter suffixes (only last one counts)
        let (id, part) = extract_part_from_suffix("SSIS-123-A-B");
        assert_eq!(id, "SSIS-123-A");
        assert_eq!(part, Some(2)); // B = 2

        // No separator before letter (NOW SUPPORTED - directly attached)
        let (id2, part2) = extract_part_from_suffix("SSIS123A");
        assert_eq!(id2, "SSIS123"); // A is now stripped
        assert_eq!(part2, Some(1)); // A = part 1

        // Letter in middle of ID (not a suffix - no digit before letter)
        let (id3, part3) = extract_part_from_suffix("SSIS-A-123");
        assert_eq!(id3, "SSIS-A-123");
        assert_eq!(part3, None);
    }

    #[test]
    fn test_parse_number_preserves_z_marker() {
        // Z suffix should be preserved as part of the ID, not converted to part number
        let result = parse_number("SSIS-123-Z.mp4", None).unwrap();
        assert_eq!(result.id, "SSIS-123-Z");
        assert_eq!(result.part_number, None);

        let result2 = parse_number("ABP-456Z.mp4", None).unwrap();
        assert_eq!(result2.id, "ABP-456Z");
        assert_eq!(result2.part_number, None);
    }

    #[test]
    fn test_multi_part_with_quality_markers() {
        // Letter suffix with quality markers
        let result = parse_number("SSIS-123-A-1080P.mp4", None).unwrap();
        assert_eq!(result.id, "SSIS-123");
        assert_eq!(result.part_number, Some(1));

        let result2 = parse_number("ABP-456-B-FHD.mkv", None).unwrap();
        assert_eq!(result2.id, "ABP-456");
        assert_eq!(result2.part_number, Some(2));
    }

    // ===== Phase 4: Configuration Tests =====

    #[test]
    fn test_parser_config_builder_pattern() {
        // Test builder pattern for ParserConfig
        let config = ParserConfig::new()
            .with_custom_regex(r"CUSTOM-\d+")
            .with_strict_mode(true)
            .with_regex_id_match(1)
            .with_regex_pt_match(2);

        assert_eq!(config.custom_regexs.len(), 1);
        assert_eq!(config.custom_regexs[0], r"CUSTOM-\d+");
        assert_eq!(config.strict_mode, true);
        assert_eq!(config.regex_id_match, 1);
        assert_eq!(config.regex_pt_match, 2);
    }

    #[test]
    fn test_parser_config_multiple_custom_regexs() {
        // Test adding multiple custom regexes
        let config = ParserConfig::new()
            .with_custom_regex(r"CUSTOM1-\d+")
            .with_custom_regex(r"CUSTOM2-\d+")
            .with_custom_regexs(vec![r"CUSTOM3-\d+".to_string(), r"CUSTOM4-\d+".to_string()]);

        assert_eq!(config.custom_regexs.len(), 4);
        assert_eq!(config.custom_regexs[0], r"CUSTOM1-\d+");
        assert_eq!(config.custom_regexs[3], r"CUSTOM4-\d+");
    }

    #[test]
    fn test_parser_config_removal_strings() {
        // Test removal strings configuration
        let config = ParserConfig::new()
            .with_removal_string("UNWANTED")
            .with_removal_string("JUNK");

        // Default removal strings + 2 custom ones
        assert!(config.removal_strings.len() > 2);
        assert!(config.removal_strings.contains(&"UNWANTED".to_string()));
        assert!(config.removal_strings.contains(&"JUNK".to_string()));
    }

    #[test]
    fn test_parser_config_replace_removal_strings() {
        // Test replacing default removal strings
        let config = ParserConfig::new()
            .with_removal_strings(vec!["ONLY1".to_string(), "ONLY2".to_string()]);

        assert_eq!(config.removal_strings.len(), 2);
        assert_eq!(config.removal_strings[0], "ONLY1");
        assert_eq!(config.removal_strings[1], "ONLY2");
    }

    #[test]
    fn test_parser_config_uncensored_prefixes() {
        // Test uncensored prefixes configuration
        let config = ParserConfig::new().with_uncensored_prefixes("CUSTOM,FOO,BAR");

        assert_eq!(config.uncensored_prefixes, "CUSTOM,FOO,BAR");
    }

    #[test]
    fn test_custom_regex_with_different_capture_groups() {
        // Test custom regex with non-default capture group indices
        let mut config = ParserConfig::new();
        config.custom_regexs = vec![r"PREFIX-([A-Z]+-\d+)-SUFFIX".to_string()];
        config.regex_id_match = 1; // First capture group

        let result = parse_number("PREFIX-SSIS-123-SUFFIX.mp4", Some(&config)).unwrap();
        assert_eq!(result.id, "SSIS-123");
    }

    #[test]
    fn test_custom_regex_with_part_number_capture() {
        // Test custom regex that captures both ID and part number
        // Use a pattern that matches the whole filename including extension
        let mut config = ParserConfig::new();
        config.custom_regexs = vec![r"([A-Z]+-\d+)[-_]VOL(\d+)".to_string()];
        config.regex_id_match = 1;
        config.regex_pt_match = 2;
        config.strict_mode = false; // Disable strict mode for this test

        let result = parse_number("SSIS-123-VOL2.mp4", Some(&config)).unwrap();
        assert_eq!(result.id, "SSIS-123");
        assert_eq!(result.part_number, Some(2));
    }

    #[test]
    fn test_strict_mode_manual_enable() {
        // Test manual strict mode - should reject non-standard IDs
        let config = ParserConfig::new().with_strict_mode(true);

        // Standard format should pass
        let result = parse_number("SSIS-123.mp4", Some(&config)).unwrap();
        assert_eq!(result.id, "SSIS-123");

        // Non-standard format with strict mode should fail
        // (assuming the extract logic would produce something non-standard)
        // This is hard to test without a truly non-standard filename that extracts weird IDs
    }

    #[test]
    fn test_auto_strict_mode_activation() {
        // Test auto-strict mode: activates when standard DVD format not detected
        let config = ParserConfig::new(); // strict_mode = false

        // Filename with standard DVD format - strict mode should NOT activate
        let result = parse_number("SSIS-123.mp4", Some(&config)).unwrap();
        assert_eq!(result.id, "SSIS-123");

        // Note: Auto-strict mode is hard to test without filenames that:
        // 1. Don't have standard DVD format in the name
        // 2. Extract to an ID that doesn't match strict pattern
        // Most real-world filenames either have standard format or extract to standard IDs
    }

    #[test]
    fn test_has_standard_dvd_format() {
        // Test the helper function indirectly via parse_number)
        // Filenames with standard format should parse successfully
        assert!(parse_number("ABC-123.mp4", None).is_ok());
        assert!(parse_number("SSIS-001.mp4", None).is_ok());
        assert!(parse_number("MIDE-999.avi", None).is_ok());

        // Filenames with standard format in them should also work
        assert!(parse_number("PREFIX-ABC-123-SUFFIX.mp4", None).is_ok());
    }

    #[test]
    fn test_strict_mode_with_suffix_characters() {
        // Test that strict mode works with multi-disc suffix stripping
        // A/B suffixes are now stripped for multi-disc support
        // Z suffix is preserved (not a disc marker)
        let config = ParserConfig::new().with_strict_mode(true);

        let result = parse_number("SSIS-123A.mp4", Some(&config)).unwrap();
        assert_eq!(result.id, "SSIS-123"); // A stripped as disc marker
        assert_eq!(result.part_number, Some(1)); // A = part 1

        let result2 = parse_number("ABP-456Z.mp4", Some(&config)).unwrap();
        assert_eq!(result2.id, "ABP-456Z"); // Z preserved (not a disc marker)
        assert_eq!(result2.part_number, None);
    }

    #[test]
    fn test_combined_config_features() {
        // Test multiple configuration features working together
        let config = ParserConfig::new()
            .with_custom_regex(r"SPECIAL-([A-Z]+-\d+)")
            .with_removal_string("UNWANTED")
            .with_strict_mode(false) // Disable strict for this test
            .with_regex_id_match(1);

        // Should use custom regex to extract ID
        let result = parse_number("SPECIAL-SSIS-123.mp4", Some(&config)).unwrap();
        assert_eq!(result.id, "SSIS-123");
    }
}

#[cfg(test)]
mod parser_fix_tests {
    use super::*;

    #[test]
    fn test_parser_fixes() {
        // Test 1: Japanese title should be stripped
        let result1 = parse_number("AVOP-212-kawaii_10周年SPECIAL企画.mp4", None).unwrap();
        assert_eq!(result1.id, "AVOP-212", "Japanese title not stripped");

        // Test 2: Watermark domain with _
        let result2 = parse_number("AVFAP.NET_okp-103.mp4", None).unwrap();
        assert_eq!(result2.id, "OKP-103", "Watermark with _ not removed");

        // Test 3: Watermark domain with @
        let result3 = parse_number("gg5.co@IPZZ-227-C_GG5.mp4", None).unwrap();
        assert_eq!(result3.id, "IPZZ-227", "Watermark with @ not removed");

        // Test 4: Quality tag in middle with space
        let result4 = parse_number("MVSD267.HD Semen Eat.mkv", None).unwrap();
        assert_eq!(result4.id, "MVSD-267", "Quality tag with space not removed");

        // Test 5: Double spaces and trailing name
        let result5 = parse_number("WANZ-220  TSUBOMI.mkv", None).unwrap();
        assert_eq!(result5.id, "WANZ-220", "Double spaces and name not handled");

        // Test 6: Attached disc C suffix (should be extracted)
        let result6 = parse_number("RCT515C.mp4", None).unwrap();
        assert_eq!(result6.id, "RCT-515", "Attached disc C not extracted");
        assert_eq!(result6.part_number, Some(3), "Disc C part number wrong");

        // Test 7: Separated -C suffix (should NOT be extracted, it's Chinese subtitle)
        let result7 = parse_number("IPZZ-227-C.mp4", None).unwrap();
        assert_eq!(result7.id, "IPZZ-227", "-C suffix should be handled as Chinese subtitle");
        assert!(result7.attributes.cn_sub, "Chinese subtitle not detected");
    }
}
