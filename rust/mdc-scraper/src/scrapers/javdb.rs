//! JavDB scraper
//!
//! JavDB (javdb.com) is a modern JAV metadata aggregator with excellent UI
//! and comprehensive coverage. It provides multi-language support and clean
//! HTML structure for reliable metadata extraction.
//!
//! Key Features:
//! - Multi-language support (English/Chinese)
//! - Modern, well-structured HTML
//! - Search-based movie discovery
//! - Comprehensive metadata coverage
//!
//! ## Locale Strategy
//! The scraper attempts English first for international compatibility,
//! then falls back to Chinese for comprehensive coverage.
//!
//! ## Implementation Notes
//! This implementation is based on verified patterns from the Javinizer project
//! (https://github.com/javinizer/Javinizer). Regex patterns are used instead
//! of CSS selectors as they provide more reliable extraction for JavDB's HTML structure.

use anyhow::{anyhow, Result};
use async_trait::async_trait;
use regex::Regex;
use scraper::Html;

use crate::metadata::MovieMetadata;
use crate::scraper::{IdFormat, Scraper, ScraperConfig};

/// JavDB scraper implementation
pub struct JavdbScraper {
    /// Base URL
    pub base_url: String,
    /// Locale preference (en/zh)
    pub locale: String,
}

impl JavdbScraper {
    /// Create a new JavDB scraper with English locale
    pub fn new() -> Self {
        Self {
            base_url: "https://javdb.com".to_string(),
            locale: "en".to_string(),
        }
    }

    /// Create scraper with custom locale
    pub fn with_locale(locale: String) -> Self {
        Self {
            base_url: "https://javdb.com".to_string(),
            locale,
        }
    }

    /// Search for movie and return detail page URL
    ///
    /// # Arguments
    /// * `number` - Movie number/ID to search for
    /// * `config` - Scraper configuration
    ///
    /// # Returns
    /// * `Ok(String)` - Detail page URL
    /// * `Err(_)` - If search fails or no results found
    async fn search_movie(&self, number: &str, config: &ScraperConfig) -> Result<String> {
        let search_url = format!("{}/search?q={}&f=all", self.base_url, number);

        if config.debug {
            tracing::debug!("[JavDB] Search URL: {}", search_url);
        }

        // Fetch search results (with cookies if configured)
        let html_text = if let Some(cookie_header) = config.get_cookie_header("javdb.com") {
            if config.debug {
                tracing::debug!("[JavDB] Using cookies for search request");
            }
            config
                .client
                .get_with_cookies(&search_url, Some(&cookie_header))
                .await?
        } else {
            config.client.get(&search_url).await?
        };

        let html = Html::parse_document(&html_text);

        // Parse search results to find matching detail URL
        self.parse_search_results(&html, number)
    }

    /// Parse search results to extract detail page path
    ///
    /// Uses regex patterns verified from Javinizer implementation:
    /// - Movie ID: `<div class="uid">(.*)</div>`
    /// - Movie Title: `<div class="video-title">(.*)</div>`
    /// - Detail URL: Extract href from matching links
    fn parse_search_results(&self, html: &Html, number: &str) -> Result<String> {
        let html_text = html.html();

        // Extract all movie IDs and their corresponding links
        let uid_re = Regex::new(r#"<div class="uid">([^<]+)</div>"#).unwrap();

        // Find all movie items (we'll look for the structure around uid/title)
        let mut matches = Vec::new();

        for uid_cap in uid_re.captures_iter(&html_text) {
            let id = uid_cap[1].trim().to_string();
            matches.push(id);
        }

        if matches.is_empty() {
            return Err(anyhow!("[JavDB] No search results found for: {}", number));
        }

        // Try to find exact match
        let number_upper = number.to_uppercase();
        for (idx, id) in matches.iter().enumerate() {
            if id.to_uppercase() == number_upper {
                // Found exact match, now extract the URL from the HTML around this position
                return self.extract_detail_url_from_html(&html_text, idx);
            }
        }

        // Try with cleaned ID (remove leading zeros from number part)
        if let Some(clean_id) = Self::clean_movie_id(number) {
            let clean_id_upper = clean_id.to_uppercase();
            for (idx, id) in matches.iter().enumerate() {
                if id.to_uppercase() == clean_id_upper {
                    return self.extract_detail_url_from_html(&html_text, idx);
                }
            }
        }

        // No exact match, take first result (log warning if needed)
        tracing::warn!(
            "[JavDB] No exact match for '{}', using first result: '{}'",
            number,
            matches[0]
        );
        self.extract_detail_url_from_html(&html_text, 0)
    }

    /// Extract detail URL from HTML by finding the nth movie item
    fn extract_detail_url_from_html(&self, html_text: &str, index: usize) -> Result<String> {
        // Look for all hrefs in the movie list
        let href_re = Regex::new(r#"<a[^>]+href="(/v/[^"]+)"[^>]*>"#).unwrap();

        for (count, cap) in href_re.captures_iter(html_text).enumerate() {
            if count == index {
                let path = &cap[1];
                return Ok(path.to_string());
            }
        }

        Err(anyhow!(
            "[JavDB] Failed to extract detail URL at index {}",
            index
        ))
    }

    /// Clean movie ID (remove leading zeros from number part)
    /// Example: "0123ABC-456" -> "123ABC-456"
    fn clean_movie_id(id: &str) -> Option<String> {
        let re = Regex::new(r"\d+(\D+-\d+)").unwrap();
        re.captures(id).map(|cap| cap[1].to_string())
    }

    /// Build detail URL with locale parameter
    fn build_detail_url(&self, path: &str, locale: &str) -> String {
        format!("{}{}?locale={}", self.base_url, path, locale)
    }

    /// Scrape with specific locale
    async fn scrape_with_locale(
        &self,
        number: &str,
        locale: &str,
        config: &ScraperConfig,
    ) -> Result<MovieMetadata> {
        // Search for movie
        let detail_path = self.search_movie(number, config).await?;

        // Build detail URL with locale
        let url = self.build_detail_url(&detail_path, locale);

        if config.debug {
            tracing::debug!("[JavDB] Detail URL ({}): {}", locale, url);
        }

        // Fetch detail page
        let html = self.fetch_html(&url, config).await?;

        // Parse metadata
        self.parse_metadata(&html, &url)
    }

    /// Clean text (remove extra whitespace)
    fn clean_text(&self, text: &str) -> String {
        text.split_whitespace()
            .collect::<Vec<_>>()
            .join(" ")
            .trim()
            .to_string()
    }

    /// Decode HTML entities
    fn decode_html_entities(&self, text: &str) -> String {
        // Basic HTML entity decoding
        text.replace("&amp;", "&")
            .replace("&lt;", "<")
            .replace("&gt;", ">")
            .replace("&quot;", "\"")
            .replace("&#39;", "'")
            .replace("&nbsp;", " ")
    }

    /// Extract text using regex pattern
    fn extract_with_regex(&self, html: &str, pattern: &str) -> Option<String> {
        let re = Regex::new(pattern).ok()?;
        re.captures(html).and_then(|cap| cap.get(1)).map(|m| {
            let decoded = self.decode_html_entities(m.as_str());
            self.clean_text(&decoded)
        })
    }

    /// Extract all matches using regex pattern
    fn extract_all_with_regex(&self, html: &str, pattern: &str) -> Vec<String> {
        let re = match Regex::new(pattern) {
            Ok(r) => r,
            Err(_) => return Vec::new(),
        };

        re.captures_iter(html)
            .filter_map(|cap| cap.get(1))
            .map(|m| {
                let decoded = self.decode_html_entities(m.as_str());
                self.clean_text(&decoded)
            })
            .filter(|s| !s.is_empty())
            .collect()
    }
}

impl Default for JavdbScraper {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Scraper for JavdbScraper {
    fn source(&self) -> &str {
        "javdb"
    }

    fn imagecut(&self) -> i32 {
        1 // Smart crop with face detection
    }

    fn preferred_id_format(&self) -> IdFormat {
        IdFormat::Display
    }

    async fn query_number_url(&self, number: &str) -> Result<String> {
        // Return search URL (actual detail URL is found via search)
        Ok(format!("{}/search?q={}&f=all", self.base_url, number))
    }

    /// Override fetch_html to support cookies
    async fn fetch_html(&self, url: &str, config: &ScraperConfig) -> Result<Html> {
        // Check if cookies are configured for javdb.com
        let html_text = if let Some(cookie_header) = config.get_cookie_header("javdb.com") {
            if config.debug {
                tracing::debug!("[JavDB] Using cookies for request");
            }
            config
                .client
                .get_with_cookies(url, Some(&cookie_header))
                .await?
        } else {
            config.client.get(url).await?
        };

        // Check for 404 patterns
        if html_text.contains("<title>404 Page Not Found")
            || html_text.contains("<title>未找到页面")
            || html_text.contains("404 Not Found")
            || html_text.contains("<title>404")
        {
            return Err(anyhow::anyhow!("Page not found (404)"));
        }

        Ok(Html::parse_document(&html_text))
    }

    async fn scrape(&self, number: &str, config: &ScraperConfig) -> Result<MovieMetadata> {
        // Try English first
        match self.scrape_with_locale(number, "en", config).await {
            Ok(mut metadata) if !metadata.title.is_empty() => {
                // Post-processing
                metadata.source = self.source().to_string();
                metadata.imagecut = self.imagecut();
                metadata.extract_year();
                metadata.normalize_runtime();
                metadata.detect_uncensored();
                return Ok(metadata);
            }
            Err(e) if config.debug => {
                tracing::debug!("[JavDB] English locale failed: {}, trying Chinese...", e);
            }
            _ => {}
        }

        // Fallback to Chinese
        let mut metadata = self.scrape_with_locale(number, "zh", config).await?;

        // Post-processing
        metadata.source = self.source().to_string();
        metadata.imagecut = self.imagecut();
        metadata.extract_year();
        metadata.normalize_runtime();
        metadata.detect_uncensored();

        Ok(metadata)
    }

    fn parse_metadata(&self, html: &Html, url: &str) -> Result<MovieMetadata> {
        let html_text = html.html();
        let mut metadata = MovieMetadata {
            website: url.to_string(),
            ..Default::default()
        };

        // Extract ID and Title from <title> tag (split by spaces)
        // Pattern from Javinizer: <title> contains "ID Title - JavDB"
        if let Some(title_match) = self.extract_with_regex(&html_text, r"<title>([^<]+)</title>") {
            let parts: Vec<&str> = title_match.split_whitespace().collect();
            if parts.len() >= 2 {
                metadata.number = parts[0].to_string();
                // Title is everything after the ID, but before " - JavDB"
                let title_parts = parts[1..].to_vec();
                let full_title = title_parts.join(" ");
                // Remove " - JavDB" suffix if present
                metadata.title = full_title
                    .split(" - JavDB")
                    .next()
                    .unwrap_or(&full_title)
                    .to_string();
            }
        }

        // Cover Image: <img src="(.*)" class="video-cover"
        if let Some(cover) = self.extract_with_regex(
            &html_text,
            r#"<img[^>]+class="video-cover"[^>]+src="([^"]+)""#,
        ) {
            metadata.cover = cover;
        } else if let Some(cover) = self.extract_with_regex(
            &html_text,
            r#"<img[^>]+src="([^"]+)"[^>]+class="video-cover""#,
        ) {
            metadata.cover = cover;
        }

        // Release Date: <span class="value">(\d{4}-\d{2}-\d{2})</span>
        if let Some(date) = self.extract_with_regex(
            &html_text,
            r#"<span class="value">(\d{4}-\d{2}-\d{2})</span>"#,
        ) {
            metadata.release = date;
        }

        // Runtime: <span class="value">(\d*) (分鍾|minute\(s\))
        // Handle both Chinese (分鍾) and English (minute(s))
        if let Some(runtime) =
            self.extract_with_regex(&html_text, r#"<span class="value">(\d+)\s*(?:分鍾|minute)"#)
        {
            metadata.runtime = runtime;
        }

        // Director: <a href="/directors/.*">(.*)</a>
        if let Some(director) =
            self.extract_with_regex(&html_text, r#"<a href="/directors/[^"]*">([^<]+)</a>"#)
        {
            metadata.director = director;
        }

        // Studio/Maker: <a href="/makers/.*">(.*)</a>
        if let Some(studio) =
            self.extract_with_regex(&html_text, r#"<a href="/makers/[^"]*">([^<]+)</a>"#)
        {
            metadata.studio = studio;
        }

        // Series: <a href=".*/series/.*">(.*)</a>
        if let Some(series) =
            self.extract_with_regex(&html_text, r#"<a href="[^"]*/ series/[^"]*">([^<]+)</a>"#)
        {
            metadata.series = series;
        }

        // Genres: <a href="/tags/.*">(.*)</a> (all matches)
        let genres =
            self.extract_all_with_regex(&html_text, r#"<a href="/tags/[^"]*">([^<]+)</a>"#);
        metadata.tag = genres;

        // Screenshots: <a class="tile-item" href="(.*)" data-fancybox="gallery"
        let screenshots = self.extract_all_with_regex(
            &html_text,
            r#"<a class="tile-item" href="([^"]+)" data-fancybox="gallery""#,
        );
        metadata.extrafanart = screenshots;

        // Trailer: src="(.*)" type="video/mp4"
        if let Some(trailer) =
            self.extract_with_regex(&html_text, r#"src="([^"]+)" type="video/mp4""#)
        {
            metadata.trailer = trailer;
        }

        // Actors: Extract actor names
        // Note: Skipping actor thumbnails to avoid N+1 HTTP requests
        // This can be added later as an optional feature
        let actor_names = self.extract_all_with_regex(
            &html_text,
            r#"<a href="/actors/[^"]*"><strong>([^<]+)</strong></a>"#,
        );
        metadata.actor = actor_names;

        // Validation: require title and cover
        if metadata.title.is_empty() {
            return Err(anyhow!("[JavDB] Failed to extract title from page"));
        }
        if metadata.cover.is_empty() {
            return Err(anyhow!("[JavDB] Failed to extract cover image from page"));
        }

        Ok(metadata)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_javdb_scraper_url() {
        let scraper = JavdbScraper::new();
        let url = scraper.query_number_url("SSIS-123").await.unwrap();
        assert!(url.contains("javdb.com"));
        assert!(url.contains("search?q=SSIS-123"));
        assert!(url.contains("&f=all"));
    }

    #[test]
    fn test_javdb_locale_configuration() {
        let scraper_en = JavdbScraper::new();
        assert_eq!(scraper_en.locale, "en");

        let scraper_zh = JavdbScraper::with_locale("zh".to_string());
        assert_eq!(scraper_zh.locale, "zh");
    }

    #[test]
    fn test_javdb_parse_metadata() {
        let scraper = JavdbScraper::new();

        let html_content = r#"
        <html>
            <head>
                <title>SSIS-123 Test JAV Movie Title - JavDB</title>
            </head>
            <body>
                <img src="https://javdb.com/covers/ssis123.jpg" class="video-cover" />
                <span class="value">2024-01-15</span>
                <span class="value">120 minute(s)</span>
                <a href="/directors/test-director">Test Director</a>
                <a href="/makers/test-studio">Test Studio</a>
                <a href="/actors/actress1"><strong>Actress One</strong></a>
                <a href="/actors/actress2"><strong>Actress Two</strong></a>
                <a href="/tags/drama">Drama</a>
                <a href="/tags/romance">Romance</a>
            </body>
        </html>
        "#;

        let html = Html::parse_document(html_content);
        let metadata = scraper
            .parse_metadata(&html, "https://javdb.com/v/abc123")
            .unwrap();

        assert_eq!(metadata.number, "SSIS-123");
        assert!(metadata.title.contains("Test JAV Movie Title"));
        assert!(metadata.cover.contains("ssis123.jpg"));
        assert_eq!(metadata.release, "2024-01-15");
        assert_eq!(metadata.runtime, "120");
        assert_eq!(metadata.director, "Test Director");
        assert_eq!(metadata.studio, "Test Studio");
        assert_eq!(metadata.actor.len(), 2);
        assert_eq!(metadata.actor[0], "Actress One");
        assert_eq!(metadata.tag.len(), 2);
        assert!(metadata.tag.contains(&"Drama".to_string()));
    }

    #[test]
    fn test_javdb_preferred_id_format() {
        let scraper = JavdbScraper::new();
        assert_eq!(scraper.preferred_id_format(), IdFormat::Display);
    }

    #[test]
    fn test_javdb_parse_search_results() {
        let scraper = JavdbScraper::new();

        let search_html = r#"
        <div class="movie-list">
            <div class="item">
                <a href="/v/abc123" class="box">
                    <div class="uid">SSIS-123</div>
                    <div class="video-title">Test Movie Title</div>
                </a>
            </div>
        </div>
        "#;

        let html = Html::parse_document(search_html);
        let detail_path = scraper.parse_search_results(&html, "SSIS-123").unwrap();
        assert_eq!(detail_path, "/v/abc123");
    }

    #[test]
    fn test_javdb_missing_title_error() {
        let scraper = JavdbScraper::new();

        let html_content = r#"
        <html>
            <body>
                <img src="https://javdb.com/covers/test.jpg" class="video-cover" />
            </body>
        </html>
        "#;

        let html = Html::parse_document(html_content);
        let result = scraper.parse_metadata(&html, "https://javdb.com/v/test");

        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Failed to extract title"));
    }
}
