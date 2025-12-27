use mdc_scraper::{HttpBackend, HttpClientBuilder, ScraperClient};
use std::time::Duration;

#[tokio::test]
async fn test_http_client_builder() {
    let client = HttpClientBuilder::new()
        .user_agent("TestBot/1.0")
        .timeout(Duration::from_secs(30))
        .retries(5)
        .debug(true)
        .build();

    assert!(client.is_ok());
}

#[tokio::test]
async fn test_scraper_client_creation() {
    let client = ScraperClient::new();
    assert!(client.is_ok());

    let client = client.unwrap();
    assert_eq!(client.backend(), HttpBackend::Reqwest);
}

#[tokio::test]
async fn test_backend_switching() {
    let mut client = ScraperClient::new().unwrap();

    // Start with Reqwest
    assert_eq!(client.backend(), HttpBackend::Reqwest);

    // Switch to CloudScraper
    client.set_backend(HttpBackend::CloudScraper);
    assert_eq!(client.backend(), HttpBackend::CloudScraper);

    // Switch back to Reqwest
    client.set_backend(HttpBackend::Reqwest);
    assert_eq!(client.backend(), HttpBackend::Reqwest);
}

#[tokio::test]
async fn test_auto_fallback_disabled() {
    let mut client = ScraperClient::new().unwrap();
    client.set_auto_fallback(false);

    // Fallback should be disabled
    // (We can't easily test this without making actual requests)
}

#[tokio::test]
async fn test_cloudscraper_availability_check() {
    let client = ScraperClient::new().unwrap();
    let available = client.is_cloudscraper_available().await;

    // This test just verifies the availability check works
    // It doesn't require cloudscraper to be installed
    println!("CloudScraper availability: {}", available);
}

// Note: These tests don't make real HTTP requests to avoid network dependencies
// Real integration tests with actual websites should be run separately
