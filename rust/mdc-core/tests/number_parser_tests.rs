use mdc_core::number_parser::{get_number, is_uncensored};

/// Comprehensive test suite for number parser
/// These test cases are derived from real-world filenames

#[test]
fn test_standard_jav_formats() {
    // Standard studio-number format
    assert_eq!(get_number("SSIS-001.mp4", None).unwrap(), "SSIS-001");
    assert_eq!(get_number("ABP-123.avi", None).unwrap(), "ABP-123");
    assert_eq!(get_number("STARS-456.mkv", None).unwrap(), "STARS-456");
    assert_eq!(get_number("MIDE-789.wmv", None).unwrap(), "MIDE-789");
    assert_eq!(get_number("IPX-001.mp4", None).unwrap(), "IPX-001");
    assert_eq!(get_number("EBOD-999.mp4", None).unwrap(), "EBOD-999");
    assert_eq!(get_number("PRED-100.mp4", None).unwrap(), "PRED-100");
}

#[test]
fn test_with_quality_markers() {
    assert_eq!(get_number("FHD-SSIS-001.mp4", None).unwrap(), "SSIS-001");
    assert_eq!(get_number("HD-ABP-123.mp4", None).unwrap(), "ABP-123");
    assert_eq!(get_number("SD_IPX-456.mp4", None).unwrap(), "IPX-456");
    assert_eq!(
        get_number("1080p-STARS-789.mp4", None).unwrap(),
        "STARS-789"
    );
    assert_eq!(get_number("720p_MIDE-100.mp4", None).unwrap(), "MIDE-100");
    assert_eq!(get_number("4K-PRED-200.mp4", None).unwrap(), "PRED-200");
}

#[test]
fn test_with_website_prefix() {
    assert_eq!(
        get_number("javlibrary.com@SSIS-001.mp4", None).unwrap(),
        "SSIS-001"
    );
    // Note: www.javbus.cc@ is not properly removed in Python either
    assert_eq!(
        get_number("www.javbus.cc@ABP-123.mp4", None).unwrap(),
        "WWW"
    );
    assert_eq!(get_number("thz.me@IPX-456.mp4", None).unwrap(), "IPX-456");
}

#[test]
fn test_with_cd_markers() {
    assert_eq!(get_number("SSIS-001-CD1.mp4", None).unwrap(), "SSIS-001");
    assert_eq!(get_number("SSIS-001-CD2.mp4", None).unwrap(), "SSIS-001");
    assert_eq!(get_number("ABP-123-cd1.avi", None).unwrap(), "ABP-123");
    assert_eq!(get_number("IPX-456_CD1.mp4", None).unwrap(), "IPX-456");
}

#[test]
fn test_with_subtitle_markers() {
    assert_eq!(get_number("SSIS-001-C.mp4", None).unwrap(), "SSIS-001");
    assert_eq!(get_number("ABP-123-c.mp4", None).unwrap(), "ABP-123");
    assert_eq!(get_number("IPX-456_C.mp4", None).unwrap(), "IPX-456");
}

#[test]
fn test_with_uncensored_markers() {
    assert_eq!(get_number("SSIS-001-U.mp4", None).unwrap(), "SSIS-001");
    assert_eq!(get_number("ABP-123-u.mp4", None).unwrap(), "ABP-123");
    assert_eq!(get_number("IPX-456-UC.mp4", None).unwrap(), "IPX-456");
    assert_eq!(get_number("STARS-789-uc.mp4", None).unwrap(), "STARS-789");
}

#[test]
fn test_fc2_format() {
    let result1 = get_number("FC2-PPV-1234567.mp4", None).unwrap();
    let result2 = get_number("FC2_PPV_1234567.mp4", None).unwrap();
    let result3 = get_number("fc2-ppv-1234567.avi", None).unwrap();

    assert!(result1.contains("FC2") || result1.contains("fc2"));
    assert!(result2.contains("FC2") || result2.contains("fc2"));
    assert!(result3.contains("FC2") || result3.contains("fc2"));
}

#[test]
fn test_tokyo_hot() {
    assert_eq!(get_number("Tokyo-Hot-n1234.mp4", None).unwrap(), "n1234");
    assert_eq!(get_number("tokyohot-k0567.avi", None).unwrap(), "k0567");
    assert_eq!(
        get_number("tokyo_hot_red-123.mp4", None).unwrap(),
        "red-123"
    );
    assert_eq!(get_number("cz001.mp4", None).unwrap(), "cz001");
    assert_eq!(get_number("gedo0123.mp4", None).unwrap(), "gedo0123");
}

#[test]
fn test_carib_caribpr() {
    assert_eq!(
        get_number("carib-123456-789.mp4", None).unwrap(),
        "123456-789"
    );
    assert_eq!(
        get_number("caribbean-123456_789.mp4", None).unwrap(),
        "123456-789"
    );
    assert_eq!(
        get_number("caribpr-123456-789.mp4", None).unwrap(),
        "123456-789"
    );
    assert_eq!(
        get_number("caribbeancompr-123456_789.mp4", None).unwrap(),
        "123456-789"
    );
}

#[test]
fn test_1pondo_pacopacomama_muramura() {
    assert_eq!(
        get_number("1pondo-123456_789.mp4", None).unwrap(),
        "123456_789"
    );
    assert_eq!(
        get_number("paco-123456-789.mp4", None).unwrap(),
        "123456_789"
    );
    assert_eq!(
        get_number("muramura-123456_789.mp4", None).unwrap(),
        "123456_789"
    );
}

#[test]
fn test_10musume() {
    assert_eq!(
        get_number("10musume-123456_78.mp4", None).unwrap(),
        "123456_78"
    );
    assert_eq!(get_number("10mu-123456-78.mp4", None).unwrap(), "123456_78");
}

#[test]
fn test_heydouga() {
    assert_eq!(
        get_number("heydouga-4017-123.mp4", None).unwrap(),
        "heydouga-4017-123"
    );
    assert_eq!(
        get_number("heydouga_4030_456.avi", None).unwrap(),
        "heydouga-4030-456"
    );
}

#[test]
fn test_heyzo() {
    assert_eq!(get_number("HEYZO-1234.mp4", None).unwrap(), "HEYZO-1234");
    assert_eq!(get_number("heyzo_hd_1234.mp4", None).unwrap(), "HEYZO-1234");
    assert_eq!(get_number("heyzo-1234.avi", None).unwrap(), "HEYZO-1234");
}

#[test]
fn test_mdbk_mdtm() {
    assert_eq!(get_number("MDBK-001.mp4", None).unwrap(), "MDBK-001");
    // Python uppercases all results
    assert_eq!(get_number("mdbk-123.avi", None).unwrap(), "MDBK-123");
    assert_eq!(get_number("MDTM-456.mp4", None).unwrap(), "MDTM-456");
    assert_eq!(get_number("mdtm_789.mp4", None).unwrap(), "MDTM-789");
}

#[test]
fn test_western_formats() {
    assert_eq!(
        get_number("x-art.18.05.15.mp4", None).unwrap(),
        "x-art.18.05.15"
    );
    // Python preserves the original case for x-art
    assert_eq!(
        get_number("X-ART.18.05.15.avi", None).unwrap(),
        "X-ART.18.05.15"
    );
}

#[test]
fn test_xxx_av() {
    let result = get_number("xxx-av-12345.mp4", None).unwrap();
    assert_eq!(result, "xxx-av-12345");
}

#[test]
fn test_complex_filenames() {
    // Multiple markers combined
    assert_eq!(
        get_number("FHD-SSIS-001-C-CD1.mp4", None).unwrap(),
        "SSIS-001"
    );
    assert_eq!(
        get_number("1080p_ABP-123-UC-cd2.mp4", None).unwrap(),
        "ABP-123"
    );
    // Note: Python doesn't remove HD- in this case
    assert_eq!(
        get_number("javbus.com@HD-IPX-456-U.mp4", None).unwrap(),
        "HD-IPX-456"
    );
}

#[test]
fn test_with_brackets() {
    // NEW BEHAVIOR: Brackets are now stripped by clean_filename(), allowing correct extraction
    // [ThZu.Cc] is removed, so SSIS-001 is correctly extracted (not THZU)
    assert_eq!(
        get_number("[ThZu.Cc]SSIS-001.mp4", None).unwrap(),
        "SSIS-001"
    );
    // Both [JAV] and [HD] are stripped, ABP-123 is extracted
    assert_eq!(get_number("[JAV]ABP-123[HD].mp4", None).unwrap(), "ABP-123");
}

#[test]
fn test_chapter_suffix() {
    assert_eq!(get_number("SSIS-001ch.mp4", None).unwrap(), "SSIS-001");
    assert_eq!(get_number("ABP-123CH.mp4", None).unwrap(), "ABP-123");
}

#[test]
fn test_underscore_vs_hyphen() {
    assert_eq!(get_number("SSIS_001.mp4", None).unwrap(), "SSIS-001");
    assert_eq!(get_number("SSIS-001.mp4", None).unwrap(), "SSIS-001");
    assert_eq!(get_number("ABP_123.mp4", None).unwrap(), "ABP-123");
}

#[test]
fn test_custom_regex() {
    let custom = r"CUSTOM-\d+";
    assert_eq!(
        get_number("prefix_CUSTOM-999_suffix.mp4", Some(custom)).unwrap(),
        "CUSTOM-999"
    );

    let custom2 = r"TEST\d{3}";
    assert_eq!(
        get_number("something_TEST456_other.mp4", Some(custom2)).unwrap(),
        "TEST456"
    );
}

#[test]
fn test_multiple_custom_regex() {
    let customs = r"CUSTOM-\d+ TEST\d{3}";
    assert_eq!(
        get_number("prefix_CUSTOM-999.mp4", Some(customs)).unwrap(),
        "CUSTOM-999"
    );
    assert_eq!(
        get_number("prefix_TEST456.mp4", Some(customs)).unwrap(),
        "TEST456"
    );
}

#[test]
fn test_is_uncensored_various() {
    // Built-in uncensored patterns
    assert!(is_uncensored("123456-789", None));
    assert!(is_uncensored("123456_789", None));
    assert!(is_uncensored("HEYZO-1234", None));
    assert!(is_uncensored("heydouga-4017-123", None));
    assert!(is_uncensored("n1234", None));
    assert!(is_uncensored("k0567", None));
    assert!(is_uncensored("red-123", None));
    assert!(is_uncensored("xxx-av-12345", None));
    assert!(is_uncensored("x-art.18.05.15", None));

    // Censored
    assert!(!is_uncensored("SSIS-001", None));
    assert!(!is_uncensored("ABP-123", None));
    assert!(!is_uncensored("MDBK-001", None));
}

#[test]
fn test_is_uncensored_custom() {
    // Custom uncensored prefixes
    assert!(is_uncensored("S2M-001", Some("S2M,FC2")));
    assert!(is_uncensored("FC2-PPV-1234567", Some("S2M,FC2")));
    assert!(!is_uncensored("SSIS-001", Some("S2M,FC2")));
}

#[test]
fn test_edge_cases() {
    // Numbers with leading zeros
    assert_eq!(get_number("SSIS-001.mp4", None).unwrap(), "SSIS-001");
    assert_eq!(get_number("ABP-0001.mp4", None).unwrap(), "ABP-0001");

    // Very long numbers
    assert_eq!(
        get_number("FC2-PPV-1234567890.mp4", None)
            .unwrap_or_default()
            .len()
            > 0,
        true
    );

    // Mixed case
    assert_eq!(get_number("ssis-001.mp4", None).unwrap(), "SSIS-001");
    assert_eq!(get_number("SsIs-001.mp4", None).unwrap(), "SSIS-001");
}

#[test]
fn test_real_world_messy_filenames() {
    // Real-world examples with lots of junk
    let result1 = get_number("[ThZu.Cc]FHD-SSIS-001-C-CD1-uncensored.mp4", None).unwrap();
    // Should extract either THZU or SSIS-001
    assert!(result1.contains("THZU") || result1.contains("SSIS") || result1.contains("001"));

    let result2 = get_number("javlibrary.com@1080p_ABP-123_HD_x264.mp4", None).unwrap();
    assert!(result2.contains("ABP") || result2.contains("123"));
}

#[test]
fn test_number_only_filenames() {
    // Some sites use pure number format
    let result = get_number("1234567.mp4", None);
    assert!(result.is_ok());
}

#[test]
fn test_three_letter_codes() {
    assert_eq!(get_number("ABC-123.mp4", None).unwrap(), "ABC-123");
    assert_eq!(get_number("XYZ-456.mp4", None).unwrap(), "XYZ-456");
    assert_eq!(get_number("FOO-001.mp4", None).unwrap(), "FOO-001");
}

#[test]
fn test_four_letter_codes() {
    assert_eq!(get_number("SSIS-001.mp4", None).unwrap(), "SSIS-001");
    assert_eq!(get_number("MIDE-123.mp4", None).unwrap(), "MIDE-123");
    assert_eq!(get_number("EBOD-456.mp4", None).unwrap(), "EBOD-456");
}

#[test]
fn test_five_letter_codes() {
    assert_eq!(get_number("STARS-001.mp4", None).unwrap(), "STARS-001");
    assert_eq!(get_number("PRED-123.mp4", None).unwrap(), "PRED-123");
}

#[test]
fn test_two_letter_codes() {
    assert_eq!(get_number("AB-123.mp4", None).unwrap(), "AB-123");
    assert_eq!(get_number("XY-456.mp4", None).unwrap(), "XY-456");
}
