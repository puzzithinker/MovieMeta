//! Mgstage scraper
//!
//! Mgstage (mgstage.com) is the official site for MGS/Prestige studio.
//! It provides high-quality metadata for MGS-produced JAV content.
//!
//! Key Features:
//! - Official studio metadata (authoritative)
//! - Search-based movie discovery
//! - Session cookie support (adc=1)
//! - Trailing slash URL normalization
//! - Japanese HTML structure (品番, 配信開始日, etc.)
//!
//! Implementation based on verified Javinizer patterns.

use crate::metadata::MovieMetadata;
use crate::scraper::{IdFormat, Scraper, ScraperConfig};
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use regex::Regex;
use scraper::{Html, Selector};
use std::sync::OnceLock;

/// Mgstage scraper
pub struct MgstageScraper {
    pub base_url: String,
}

impl MgstageScraper {
    /// Create new Mgstage scraper
    pub fn new() -> Self {
        Self {
            base_url: "https://www.mgstage.com".to_string(),
        }
    }

    /// Search for movie and return detail page URL
    async fn search_movie(&self, number: &str, config: &ScraperConfig) -> Result<String> {
        let search_url = format!("{}/search/cSearch.php?search_word={}", self.base_url, number);

        // Fetch search results with cookie
        let html_text = if let Some(cookie_header) = config.get_cookie_header("mgstage.com") {
            config.client.get_with_cookies(&search_url, Some(&cookie_header)).await?
        } else {
            // Add default adc=1 cookie if not in config
            config.client.get_with_cookies(&search_url, Some("adc=1")).await?
        };

        let html = Html::parse_document(&html_text);

        // Extract detail page URL from search results
        self.parse_search_results(&html, number)
    }

    /// Parse search results and extract detail page URL
    fn parse_search_results(&self, html: &Html, number: &str) -> Result<String> {
        // Pattern: <a href="/product/product_detail/.../">
        static HREF_RE: OnceLock<Regex> = OnceLock::new();
        let href_re = HREF_RE.get_or_init(|| {
            Regex::new(r#"<a\s+href="(/product/product_detail/[^"]+)""#).unwrap()
        });

        let html_text = html.html();

        // Find all product links
        let matches: Vec<String> = href_re
            .captures_iter(&html_text)
            .filter_map(|cap| cap.get(1).map(|m| m.as_str().to_string()))
            .collect();

        if matches.is_empty() {
            return Err(anyhow!("No search results found for '{}'", number));
        }

        // Try to find exact match for the ID (case-insensitive)
        let number_lower = number.to_lowercase();
        for path in &matches {
            if path.to_lowercase().contains(&number_lower) {
                return Ok(self.normalize_url(path));
            }
        }

        // If no exact match, use first result
        tracing::warn!(
            "[Mgstage] No exact match for '{}', using first result: '{}'",
            number,
            matches[0]
        );
        Ok(self.normalize_url(&matches[0]))
    }

    /// Normalize URL with trailing slash
    fn normalize_url(&self, path: &str) -> String {
        let full_url = format!("{}{}", self.base_url, path);
        if full_url.ends_with('/') {
            full_url
        } else {
            format!("{}/", full_url)
        }
    }

    /// Parse metadata from detail page (custom implementation)
    fn parse_detail_metadata(&self, html: &Html, url: &str) -> Result<MovieMetadata> {
        let mut metadata = MovieMetadata {
            website: url.to_string(),
            ..Default::default()
        };

        // Extract title from <title> tag
        if let Ok(selector) = Selector::parse("title") {
            if let Some(element) = html.select(&selector).next() {
                let title = element.text().collect::<String>();
                metadata.title = self.clean_text(&title);
            }
        }

        // Extract metadata from info table (Japanese HTML structure)
        // Pattern: <th>Label:</th><td>Value</td>

        // ID (品番)
        metadata.number = self.extract_table_value(html, "品番：");

        // Release Date (配信開始日)
        if let Some(date_str) = self.extract_table_value_opt(html, "配信開始日：") {
            metadata.release = date_str;
        }

        // Runtime (収録時間)
        if let Some(runtime_str) = self.extract_table_value_opt(html, "収録時間：") {
            // Remove "min" suffix
            metadata.runtime = runtime_str.replace("min", "").trim().to_string();
        }

        // Maker/Studio (メーカー)
        if let Some(studio) = self.extract_table_link(html, "メーカー：") {
            metadata.studio = studio;
        }

        // Label (レーベル)
        if let Some(label) = self.extract_table_link(html, "レーベル：") {
            metadata.label = label;
        }

        // Series (シリーズ)
        if let Some(series) = self.extract_table_link(html, "シリーズ：") {
            metadata.series = series;
        }

        // Genres (ジャンル)
        metadata.tag = self.extract_genres(html);

        // Actresses (出演)
        metadata.actor = self.extract_actresses(html);

        // Cover Image
        if let Some(cover) = self.extract_cover_image(html) {
            metadata.cover = cover;
        }

        // Screenshots
        metadata.extrafanart = self.extract_screenshots(html);

        // Validation
        if metadata.title.is_empty() {
            return Err(anyhow!("Missing required field: title"));
        }

        Ok(metadata)
    }

    /// Extract value from table cell (Japanese label)
    fn extract_table_value(&self, html: &Html, label: &str) -> String {
        self.extract_table_value_opt(html, label).unwrap_or_default()
    }

    /// Extract optional value from table cell
    fn extract_table_value_opt(&self, html: &Html, label: &str) -> Option<String> {
        // Pattern: <th>Label:</th><td>Value</td>
        static TH_SELECTOR: OnceLock<Selector> = OnceLock::new();
        let th_sel = TH_SELECTOR.get_or_init(|| Selector::parse("th").unwrap());

        for th in html.select(th_sel) {
            let th_text = th.text().collect::<String>();
            if th_text.trim() == label {
                // Get next sibling <td>
                if let Some(parent) = th.parent() {
                    if let Some(td) = parent.children().find_map(|child| {
                        if let Some(elem) = child.value().as_element() {
                            if elem.name() == "td" {
                                return Some(child);
                            }
                        }
                        None
                    }) {
                        let value = td.children()
                            .filter_map(|c| c.value().as_text().map(|t| t.text.to_string()))
                            .collect::<String>();
                        let cleaned = self.clean_text(&value);
                        if !cleaned.is_empty() {
                            return Some(cleaned);
                        }
                    }
                }
            }
        }
        None
    }

    /// Extract link text from table cell
    fn extract_table_link(&self, html: &Html, label: &str) -> Option<String> {
        // Use regex to extract link text for the given label
        let pattern = format!(r#"<th>{}</th>\s*<td[^>]*>\s*<a[^>]*>([^<]+)</a>"#, regex::escape(label));

        let html_text = html.html();

        if let Ok(re) = Regex::new(&pattern) {
            if let Some(cap) = re.captures(&html_text) {
                if let Some(m) = cap.get(1) {
                    let text = self.clean_text(m.as_str());
                    if !text.is_empty() {
                        return Some(text);
                    }
                }
            }
        }
        None
    }

    /// Extract genres from table
    fn extract_genres(&self, html: &Html) -> Vec<String> {
        let html_text = html.html();

        // Pattern: <th>ジャンル：</th> followed by genre links
        static GENRE_RE: OnceLock<Regex> = OnceLock::new();
        let genre_re = GENRE_RE.get_or_init(|| {
            Regex::new(r#"/search/csearch\.php\?genre\[\]=\d+"[^>]*>([^<]+)<"#).unwrap()
        });

        genre_re
            .captures_iter(&html_text)
            .filter_map(|cap| cap.get(1).map(|m| self.clean_text(m.as_str())))
            .filter(|s| !s.is_empty())
            .collect()
    }

    /// Extract actresses
    fn extract_actresses(&self, html: &Html) -> Vec<String> {
        let html_text = html.html();

        // Pattern: <th>出演：</th> followed by actress links
        static ACTRESS_RE: OnceLock<Regex> = OnceLock::new();
        let actress_re = ACTRESS_RE.get_or_init(|| {
            Regex::new(r#"/search/csearch\.php\?actress\[\]=\d+"[^>]*>([^<]+)<"#).unwrap()
        });

        actress_re
            .captures_iter(&html_text)
            .filter_map(|cap| cap.get(1).map(|m| self.clean_text(m.as_str())))
            .filter(|s| !s.is_empty())
            .collect()
    }

    /// Extract cover image
    fn extract_cover_image(&self, html: &Html) -> Option<String> {
        let html_text = html.html();

        // Pattern: <a ... class="link_magnify" ... href="*.jpg">
        // Match attributes in any order
        static COVER_RE1: OnceLock<Regex> = OnceLock::new();
        static COVER_RE2: OnceLock<Regex> = OnceLock::new();

        let re1 = COVER_RE1.get_or_init(|| {
            Regex::new(r#"class="link_magnify"[^>]*?href="([^"]+\.jpg)""#).unwrap()
        });
        let re2 = COVER_RE2.get_or_init(|| {
            Regex::new(r#"href="([^"]+\.jpg)"[^>]*?class="link_magnify""#).unwrap()
        });

        // Try both patterns
        re1.captures(&html_text)
            .or_else(|| re2.captures(&html_text))
            .and_then(|cap| cap.get(1))
            .map(|m| m.as_str().to_string())
    }

    /// Extract screenshots
    fn extract_screenshots(&self, html: &Html) -> Vec<String> {
        let html_text = html.html();

        // Pattern: class="sample_image" href="(.*.jpg)"
        static SCREENSHOT_RE: OnceLock<Regex> = OnceLock::new();
        let screenshot_re = SCREENSHOT_RE.get_or_init(|| {
            Regex::new(r#"class="sample_image"\s+href="([^"]+\.jpg)""#).unwrap()
        });

        screenshot_re
            .captures_iter(&html_text)
            .filter_map(|cap| cap.get(1).map(|m| m.as_str().to_string()))
            .collect()
    }

    /// Clean text (whitespace, HTML entities)
    fn clean_text(&self, text: &str) -> String {
        text.trim()
            .replace("&amp;", "&")
            .replace("&lt;", "<")
            .replace("&gt;", ">")
            .replace("&quot;", "\"")
            .replace("&#39;", "'")
    }
}

impl Default for MgstageScraper {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Scraper for MgstageScraper {
    fn source(&self) -> &str {
        "mgstage"
    }

    fn imagecut(&self) -> i32 {
        1 // Smart crop
    }

    fn preferred_id_format(&self) -> IdFormat {
        IdFormat::Display // Mgstage uses display format (SSIS-123)
    }

    async fn query_number_url(&self, number: &str) -> Result<String> {
        // This is overridden by scrape(), but required by trait
        Ok(format!("{}/search/cSearch.php?search_word={}", self.base_url, number))
    }

    fn parse_metadata(&self, html: &Html, url: &str) -> Result<MovieMetadata> {
        // Required by trait - calls our custom implementation
        self.parse_detail_metadata(html, url)
    }

    async fn scrape(&self, number: &str, config: &ScraperConfig) -> Result<MovieMetadata> {
        // Search for movie
        let detail_url = self.search_movie(number, config).await?;

        // Fetch detail page
        let html_text = if let Some(cookie_header) = config.get_cookie_header("mgstage.com") {
            config.client.get_with_cookies(&detail_url, Some(&cookie_header)).await?
        } else {
            config.client.get_with_cookies(&detail_url, Some("adc=1")).await?
        };

        let html = Html::parse_document(&html_text);

        // Parse metadata
        self.parse_detail_metadata(&html, &detail_url)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mgstage_scraper_url() {
        let scraper = MgstageScraper::new();
        assert_eq!(scraper.base_url, "https://www.mgstage.com");
        assert_eq!(scraper.source(), "mgstage");
    }

    #[test]
    fn test_mgstage_preferred_id_format() {
        let scraper = MgstageScraper::new();
        assert!(matches!(scraper.preferred_id_format(), IdFormat::Display));
    }

    #[test]
    fn test_mgstage_normalize_url() {
        let scraper = MgstageScraper::new();

        // Without trailing slash
        let url1 = scraper.normalize_url("/product/product_detail/SIRO-123");
        assert_eq!(url1, "https://www.mgstage.com/product/product_detail/SIRO-123/");

        // With trailing slash
        let url2 = scraper.normalize_url("/product/product_detail/SIRO-123/");
        assert_eq!(url2, "https://www.mgstage.com/product/product_detail/SIRO-123/");
    }

    #[test]
    fn test_mgstage_parse_search_results() {
        let scraper = MgstageScraper::new();

        let html_content = r#"
            <html>
                <body>
                    <a href="/product/product_detail/SIRO-123/">SIRO-123</a>
                    <a href="/product/product_detail/SIRO-456/">SIRO-456</a>
                </body>
            </html>
        "#;

        let html = Html::parse_document(html_content);
        let result = scraper.parse_search_results(&html, "SIRO-123").unwrap();
        assert_eq!(result, "https://www.mgstage.com/product/product_detail/SIRO-123/");
    }

    #[test]
    fn test_mgstage_parse_detail_metadata() {
        let scraper = MgstageScraper::new();

        let html_content = r#"
            <html>
                <head><title>Test Movie Title - Mgstage</title></head>
                <body>
                    <table>
                        <tr><th>品番：</th><td>SIRO-123</td></tr>
                        <tr><th>配信開始日：</th><td>2023-01-15</td></tr>
                        <tr><th>収録時間：</th><td>120min</td></tr>
                        <tr><th>メーカー：</th><td><a href="/maker">Prestige</a></td></tr>
                        <tr><th>レーベル：</th><td><a href="/label">SIRO</a></td></tr>
                        <tr><th>シリーズ：</th><td><a href="/series">Amateur Series</a></td></tr>
                    </table>
                    <a class="link_magnify" href="https://image.mgstage.com/cover.jpg">Cover</a>
                    <a class="sample_image" href="https://image.mgstage.com/sample1.jpg">Sample 1</a>
                    <a class="sample_image" href="https://image.mgstage.com/sample2.jpg">Sample 2</a>
                </body>
            </html>
        "#;

        let html = Html::parse_document(html_content);
        let metadata = scraper.parse_detail_metadata(&html, "https://www.mgstage.com/product/product_detail/SIRO-123/").unwrap();

        assert!(metadata.title.contains("Test Movie Title"));
        assert_eq!(metadata.number, "SIRO-123");
        assert_eq!(metadata.release, "2023-01-15");
        assert_eq!(metadata.runtime, "120");
        assert_eq!(metadata.studio, "Prestige");
        assert_eq!(metadata.label, "SIRO");
        assert_eq!(metadata.series, "Amateur Series");
        assert_eq!(metadata.cover, "https://image.mgstage.com/cover.jpg");
        // Note: Screenshots extraction may vary based on HTML structure
        // In real HTML, screenshots are present but test HTML structure may differ
        assert_eq!(metadata.extrafanart.len(), 2);
    }
}
