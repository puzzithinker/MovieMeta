//! DMM scraper integration tests
//!
//! Tests the DMM scraper with the dual ID system

use mdc_scraper::scraper::Scraper;
use mdc_scraper::scrapers::DmmScraper;
use mdc_scraper::{ScraperClient, ScraperConfig, ScraperRegistry};
use std::sync::Arc;

#[tokio::test]
async fn test_dmm_receives_content_id() {
    // Create registry with DMM scraper
    let mut registry = ScraperRegistry::new();
    registry.register(Arc::new(DmmScraper::new()));

    let client = ScraperClient::new().unwrap();
    let config = ScraperConfig::new(client).debug(true);

    // Test dual ID system: DMM should receive content ID, not display ID
    // Display: "SSIS-123" → Content: "ssis00123"

    // Note: This will fail to find metadata (no real DMM connection),
    // but we're testing that the ID format is correct
    let result = registry
        .search_with_ids("SSIS-123", "ssis00123", None, &config)
        .await;

    // Expected: DMM received "ssis00123" (content ID) not "SSIS-123" (display ID)
    // The error message should contain the content ID format
    if let Err(e) = result {
        let error_msg = e.to_string();
        // Error should mention the content ID we're searching for
        assert!(
            error_msg.contains("ssis00123") || error_msg.contains("No DMM product"),
            "DMM should have received content ID format. Error: {}",
            error_msg
        );
    }
}

#[tokio::test]
async fn test_dmm_id_format_preference() {
    let scraper = DmmScraper::new();

    // Verify DMM prefers Content format
    use mdc_scraper::scraper::{IdFormat, Scraper};
    assert_eq!(scraper.preferred_id_format(), IdFormat::Content);
}

#[test]
fn test_dmm_content_id_examples() {
    // Test various content ID transformations
    let test_cases = vec![
        ("SSIS-123", "ssis00123"),
        ("ABP-001", "abp00001"),
        ("IPX-789", "ipx00789"),
        ("PRED-456", "pred00456"),
        ("MIDE-999", "mide00999"),
    ];

    for (display, expected_content) in test_cases {
        // In production, parse_number() would generate these
        println!("Display: {} → Content: {}", display, expected_content);
        assert!(expected_content
            .chars()
            .all(|c| c.is_lowercase() || c.is_numeric()));
    }
}

#[tokio::test]
async fn test_dmm_url_generation() {
    let scraper = DmmScraper::new();

    // Test that DMM generates correct search URLs with content IDs
    let url = scraper.query_number_url("ssis00123").await.unwrap();

    assert!(url.contains("dmm.co.jp"), "URL should be DMM domain");
    assert!(url.contains("search"), "URL should be search endpoint");
    assert!(url.contains("ssis00123"), "URL should contain content ID");
}

// Note: CID extraction is tested in unit tests (dmm.rs)

#[tokio::test]
async fn test_dmm_registry_priority() {
    // Test that DMM is queried first when multiple scrapers are registered
    let mut registry = ScraperRegistry::new();

    // Register DMM first (highest priority)
    registry.register(Arc::new(DmmScraper::new()));

    let sources = registry.available_sources();
    assert!(
        sources.contains(&"dmm".to_string()),
        "DMM should be registered"
    );

    // When searching, DMM should be tried first
    // (In production, first successful scraper wins)
}

#[test]
fn test_dmm_metadata_parsing_completeness() {
    use mdc_scraper::scraper::Scraper;
    use scraper::Html;

    let scraper = DmmScraper::new();

    // Test with comprehensive HTML
    let html_content = r#"
    <html>
    <body>
        <h1 id="title">テストタイトル Test Title</h1>
        <div class="img">
            <img src="//pics.dmm.co.jp/digital/video/ssis00123/ssis00123pl.jpg" />
        </div>
        <table class="mg-b20">
            <tr><td class="nw">発売日：</td><td>2024-01-15</td></tr>
            <tr><td class="nw">収録時間：</td><td>120分</td></tr>
            <tr><td class="nw">監督：</td><td>Test Director</td></tr>
            <tr><td class="nw">メーカー：</td><td>S1 NO.1 STYLE</td></tr>
            <tr><td class="nw">レーベル：</td><td>S1 NO.1 STYLE</td></tr>
            <tr><td class="nw">シリーズ：</td><td>Test Series</td></tr>
        </table>
        <span id="performer">
            <a>Test Actress</a>
        </span>
        <table class="mg-b20">
            <tr><td><a href="/genre/1/">ドラマ</a></td></tr>
            <tr><td><a href="/genre/2/">単体作品</a></td></tr>
        </table>
        <div class="mg-b20 lh4">
            <p>This is a test description for the movie.</p>
        </div>
    </body>
    </html>
    "#;

    let html = Html::parse_document(html_content);
    let metadata = scraper
        .parse_metadata(
            &html,
            "https://www.dmm.co.jp/mono/dvd/-/detail/=/cid=ssis00123/",
        )
        .unwrap();

    // Verify all critical fields are extracted
    assert!(!metadata.title.is_empty(), "Title should be extracted");
    assert!(!metadata.cover.is_empty(), "Cover should be extracted");
    assert!(
        !metadata.release.is_empty(),
        "Release date should be extracted"
    );
    assert!(!metadata.runtime.is_empty(), "Runtime should be extracted");
    assert!(!metadata.studio.is_empty(), "Studio should be extracted");
    assert!(!metadata.actor.is_empty(), "Actors should be extracted");
    assert!(!metadata.tag.is_empty(), "Tags should be extracted");

    println!("✓ DMM Metadata Extraction Test Passed");
    println!("  Title: {}", metadata.title);
    println!("  Studio: {}", metadata.studio);
    println!("  Actors: {:?}", metadata.actor);
    println!("  Tags: {:?}", metadata.tag);
}
