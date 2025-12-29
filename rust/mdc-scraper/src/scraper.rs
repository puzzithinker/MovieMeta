//! Scraper trait and configuration

use anyhow::Result;
use async_trait::async_trait;
use scraper::{Html, Selector};

use crate::client::ScraperClient;
use crate::metadata::MovieMetadata;

/// ID format preference for scrapers
///
/// Different scrapers may require different ID formats:
/// - Display: Human-readable format like "SSIS-123" (most scrapers)
/// - Content: API format like "ssis00123" (DMM, JAVLibrary)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IdFormat {
    /// Display format: "SSIS-123" (human-readable, with hyphen)
    Display,

    /// Content format: "ssis00123" (lowercase, zero-padded, for APIs)
    Content,
}

impl Default for IdFormat {
    fn default() -> Self {
        IdFormat::Display
    }
}

/// Scraper configuration passed to each scraper
#[derive(Debug, Clone)]
pub struct ScraperConfig {
    /// HTTP client for making requests
    pub client: ScraperClient,

    /// Enable debug output
    pub debug: bool,

    /// Request more detailed storyline/outline
    pub morestoryline: bool,

    /// Specific URL to scrape (overrides search)
    pub specified_url: Option<String>,
}

impl ScraperConfig {
    /// Create a new scraper configuration
    pub fn new(client: ScraperClient) -> Self {
        Self {
            client,
            debug: false,
            morestoryline: false,
            specified_url: None,
        }
    }

    /// Enable debug mode
    pub fn debug(mut self, debug: bool) -> Self {
        self.debug = debug;
        self
    }

    /// Request more detailed storyline
    pub fn morestoryline(mut self, enabled: bool) -> Self {
        self.morestoryline = enabled;
        self
    }

    /// Set specific URL to scrape
    pub fn specified_url(mut self, url: Option<String>) -> Self {
        self.specified_url = url;
        self
    }
}

/// Base scraper trait for metadata sources
#[async_trait]
pub trait Scraper: Send + Sync {
    /// Source identifier (e.g., "tmdb", "imdb")
    fn source(&self) -> &str;

    /// Default image cut mode
    /// - 0: Copy cover without cropping
    /// - 1: Smart crop with face detection (default)
    /// - 3: Download small cover
    fn imagecut(&self) -> i32 {
        1
    }

    /// Preferred ID format for this scraper
    ///
    /// Most scrapers use Display format (SSIS-123), but some like
    /// DMM and JAVLibrary require Content format (ssis00123).
    ///
    /// # Returns
    /// - `IdFormat::Display` - Most scrapers (default)
    /// - `IdFormat::Content` - DMM, JAVLibrary (lowercase, zero-padded)
    fn preferred_id_format(&self) -> IdFormat {
        IdFormat::Display
    }

    /// Main scraping entry point
    ///
    /// # Arguments
    /// * `number` - Movie number/ID to scrape
    /// * `config` - Scraper configuration
    async fn scrape(&self, number: &str, config: &ScraperConfig) -> Result<MovieMetadata> {
        // Get URL (either specified or via query)
        let url = if let Some(ref specified_url) = config.specified_url {
            specified_url.clone()
        } else {
            self.query_number_url(number).await?
        };

        if config.debug {
            tracing::debug!("Scraping URL: {}", url);
        }

        // Fetch HTML
        let html = self.fetch_html(&url, config).await?;

        // Parse metadata
        let mut metadata = self.parse_metadata(&html, &url)?;

        // Post-processing
        metadata.source = self.source().to_string();
        metadata.website = url;
        metadata.imagecut = self.imagecut();
        metadata.extract_year();
        metadata.normalize_runtime();
        metadata.detect_uncensored();

        Ok(metadata)
    }

    /// Query the detail page URL for a given number
    async fn query_number_url(&self, number: &str) -> Result<String>;

    /// Fetch and parse HTML from URL
    async fn fetch_html(&self, url: &str, config: &ScraperConfig) -> Result<Html> {
        let html_text = config.client.get(url).await?;

        // Check for 404 patterns
        if html_text.contains("<title>404 Page Not Found")
            || html_text.contains("<title>未找到页面")
            || html_text.contains("404 Not Found")
            || html_text.contains("<title>404")
            || html_text.contains("<title>お探しの商品が見つかりません")
        {
            return Err(anyhow::anyhow!("Page not found (404)"));
        }

        Ok(Html::parse_document(&html_text))
    }

    /// Parse metadata from HTML document
    fn parse_metadata(&self, html: &Html, url: &str) -> Result<MovieMetadata>;

    /// Extract text from element using CSS selector
    fn select_text(&self, html: &Html, selector_str: &str) -> String {
        if selector_str.is_empty() {
            return String::new();
        }

        match Selector::parse(selector_str) {
            Ok(selector) => html
                .select(&selector)
                .next()
                .map(|el| el.text().collect::<String>().trim().to_string())
                .unwrap_or_default(),
            Err(_) => String::new(),
        }
    }

    /// Extract attribute from element using CSS selector
    fn select_attr(&self, html: &Html, selector_str: &str, attr: &str) -> String {
        if selector_str.is_empty() {
            return String::new();
        }

        match Selector::parse(selector_str) {
            Ok(selector) => html
                .select(&selector)
                .next()
                .and_then(|el| el.value().attr(attr))
                .unwrap_or_default()
                .trim()
                .to_string(),
            Err(_) => String::new(),
        }
    }

    /// Extract all text matches for a selector
    fn select_all_text(&self, html: &Html, selector_str: &str) -> Vec<String> {
        if selector_str.is_empty() {
            return Vec::new();
        }

        match Selector::parse(selector_str) {
            Ok(selector) => html
                .select(&selector)
                .map(|el| el.text().collect::<String>().trim().to_string())
                .filter(|s| !s.is_empty())
                .collect(),
            Err(_) => Vec::new(),
        }
    }

    /// Extract all attribute values for a selector
    fn select_all_attr(&self, html: &Html, selector_str: &str, attr: &str) -> Vec<String> {
        if selector_str.is_empty() {
            return Vec::new();
        }

        match Selector::parse(selector_str) {
            Ok(selector) => html
                .select(&selector)
                .filter_map(|el| el.value().attr(attr))
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect(),
            Err(_) => Vec::new(),
        }
    }

    /// Try multiple selectors, return first match
    fn select_text_by_exprs(&self, html: &Html, selectors: &[&str]) -> String {
        for selector_str in selectors {
            let result = self.select_text(html, selector_str);
            if !result.is_empty() {
                return result;
            }
        }
        String::new()
    }

    /// Try multiple selectors for attributes, return first match
    fn select_attr_by_exprs(&self, html: &Html, selectors: &[&str], attr: &str) -> String {
        for selector_str in selectors {
            let result = self.select_attr(html, selector_str, attr);
            if !result.is_empty() {
                return result;
            }
        }
        String::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestScraper;

    #[async_trait]
    impl Scraper for TestScraper {
        fn source(&self) -> &str {
            "test"
        }

        async fn query_number_url(&self, number: &str) -> Result<String> {
            Ok(format!("http://example.com/{}", number))
        }

        fn parse_metadata(&self, _html: &Html, _url: &str) -> Result<MovieMetadata> {
            let mut meta = MovieMetadata::default();
            meta.number = "TEST-001".to_string();
            meta.title = "Test Movie".to_string();
            meta.cover = "http://example.com/cover.jpg".to_string();
            Ok(meta)
        }
    }

    #[test]
    fn test_scraper_trait_methods() {
        let scraper = TestScraper;
        assert_eq!(scraper.source(), "test");
        assert_eq!(scraper.imagecut(), 1);

        let html = Html::parse_document(
            r#"<html><head><title>Test</title></head><body><h1>Hello</h1></body></html>"#,
        );
        let title = scraper.select_text(&html, "title");
        assert_eq!(title, "Test");

        let h1 = scraper.select_text(&html, "h1");
        assert_eq!(h1, "Hello");
    }

    #[test]
    fn test_select_attr() {
        let scraper = TestScraper;
        let html = Html::parse_document(
            r#"<html><head><meta property="og:title" content="Test Title"/></head></html>"#,
        );
        let title = scraper.select_attr(&html, r#"meta[property="og:title"]"#, "content");
        assert_eq!(title, "Test Title");
    }

    #[test]
    fn test_select_all_text() {
        let scraper = TestScraper;
        let html = Html::parse_document(
            r#"<html><body><ul><li>One</li><li>Two</li><li>Three</li></ul></body></html>"#,
        );
        let items = scraper.select_all_text(&html, "li");
        assert_eq!(items, vec!["One", "Two", "Three"]);
    }

    // ===== Phase 5: ID Format Tests =====

    #[test]
    fn test_id_format_default() {
        // Test that IdFormat::default() returns Display
        assert_eq!(IdFormat::default(), IdFormat::Display);
    }

    #[test]
    fn test_id_format_equality() {
        // Test IdFormat enum equality
        assert_eq!(IdFormat::Display, IdFormat::Display);
        assert_eq!(IdFormat::Content, IdFormat::Content);
        assert_ne!(IdFormat::Display, IdFormat::Content);
    }

    #[test]
    fn test_scraper_default_id_format() {
        // Test that default scrapers prefer Display format
        let scraper = TestScraper;
        assert_eq!(scraper.preferred_id_format(), IdFormat::Display);
    }

    struct ContentIdScraper;

    #[async_trait]
    impl Scraper for ContentIdScraper {
        fn source(&self) -> &str {
            "content_test"
        }

        fn preferred_id_format(&self) -> IdFormat {
            IdFormat::Content
        }

        async fn query_number_url(&self, number: &str) -> Result<String> {
            Ok(format!("http://example.com/{}", number))
        }

        fn parse_metadata(&self, _html: &Html, _url: &str) -> Result<MovieMetadata> {
            Ok(MovieMetadata::default())
        }
    }

    #[test]
    fn test_scraper_content_id_format() {
        // Test scraper with Content format preference
        let scraper = ContentIdScraper;
        assert_eq!(scraper.preferred_id_format(), IdFormat::Content);
    }
}
