//! DMM/FANZA scraper
//!
//! DMM (dmm.co.jp / FANZA) is the official and most authoritative source for JAV metadata.
//! It provides high-quality cover images, comprehensive metadata, and serves as the
//! primary reference for most other JAV aggregators.

use anyhow::{anyhow, Result};
use async_trait::async_trait;
use scraper::{Html, Selector};
use std::collections::HashMap;

use crate::metadata::MovieMetadata;
use crate::scraper::{IdFormat, Scraper, ScraperConfig};

/// DMM/FANZA scraper implementation
pub struct DmmScraper {
    /// Base URL (DMM.co.jp domain)
    pub base_url: String,
}

impl DmmScraper {
    /// Create a new DMM scraper with default URL
    pub fn new() -> Self {
        Self {
            base_url: "https://www.dmm.co.jp".to_string(),
        }
    }

    /// Create scraper with custom base URL
    pub fn with_base_url(base_url: String) -> Self {
        Self { base_url }
    }

    /// Extract content ID (cid) from URL
    /// Example: /mono/dvd/-/detail/=/cid=ssis00123/ → ssis00123
    fn extract_cid_from_url(&self, url: &str) -> Option<String> {
        if let Some(cid_start) = url.find("cid=") {
            let cid_part = &url[cid_start + 4..];
            let cid = cid_part
                .split('/')
                .next()
                .unwrap_or("")
                .split('&')
                .next()
                .unwrap_or("")
                .to_string();

            if !cid.is_empty() {
                return Some(cid);
            }
        }
        None
    }

    /// Parse info table to extract metadata fields
    fn parse_info_table(&self, html: &Html) -> HashMap<String, String> {
        let mut info = HashMap::new();

        // DMM uses a table structure with <td> pairs
        // Structure: <td class="nw">Label:</td><td>Value</td>
        if let Ok(row_selector) = Selector::parse("table.mg-b20 tr") {
            for row in html.select(&row_selector) {
                if let Ok(label_sel) = Selector::parse("td.nw") {
                    if let Ok(value_sel) = Selector::parse("td:not(.nw)") {
                        if let (Some(label), Some(value)) =
                            (row.select(&label_sel).next(), row.select(&value_sel).next())
                        {
                            let key = label.text().collect::<String>().trim().to_string();
                            let val = value.text().collect::<String>().trim().to_string();
                            if !key.is_empty() && !val.is_empty() {
                                info.insert(key, val);
                            }
                        }
                    }
                }
            }
        }

        info
    }

    /// Clean DMM text (removes extra whitespace and formatting)
    fn clean_text(&self, text: &str) -> String {
        text.split_whitespace()
            .collect::<Vec<_>>()
            .join(" ")
            .trim()
            .to_string()
    }

    /// Search for both DVD and Digital versions
    async fn search_dual_format(&self, content_id: &str, config: &ScraperConfig) -> Result<String> {
        let search_url = format!("{}/search/=/searchstr={}/", self.base_url, content_id);

        if config.debug {
            tracing::debug!("DMM search URL: {}", search_url);
        }

        // Fetch search results
        let html_text = config.client.get(&search_url).await?;
        let html = Html::parse_document(&html_text);

        // Look for DVD detail page link
        if let Ok(selector) = Selector::parse("a[href*='/mono/dvd/-/detail/']") {
            if let Some(link) = html.select(&selector).next() {
                if let Some(href) = link.value().attr("href") {
                    let full_url = if href.starts_with("http") {
                        href.to_string()
                    } else {
                        format!("{}{}", self.base_url, href)
                    };
                    return Ok(full_url);
                }
            }
        }

        // Fallback: Look for Digital video link
        if let Ok(selector) = Selector::parse("a[href*='/digital/videoa/-/detail/']") {
            if let Some(link) = html.select(&selector).next() {
                if let Some(href) = link.value().attr("href") {
                    let full_url = if href.starts_with("http") {
                        href.to_string()
                    } else {
                        format!("{}{}", self.base_url, href)
                    };
                    return Ok(full_url);
                }
            }
        }

        Err(anyhow!("No DMM product page found for: {}", content_id))
    }
}

impl Default for DmmScraper {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Scraper for DmmScraper {
    fn source(&self) -> &str {
        "dmm"
    }

    fn imagecut(&self) -> i32 {
        1 // Smart crop with face detection
    }

    fn preferred_id_format(&self) -> IdFormat {
        // DMM requires content ID format (lowercase, zero-padded)
        // Example: "SSIS-123" should be passed as "ssis00123"
        IdFormat::Content
    }

    async fn query_number_url(&self, number: &str) -> Result<String> {
        // DMM expects content ID format (already provided by registry)
        // Build search URL and find the product page
        let search_url = format!("{}/search/=/searchstr={}/", self.base_url, number);
        Ok(search_url)
    }

    async fn scrape(&self, number: &str, config: &ScraperConfig) -> Result<MovieMetadata> {
        // Use dual format search to find the product page
        let url = self.search_dual_format(number, config).await?;

        if config.debug {
            tracing::debug!("DMM detail URL: {}", url);
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

    fn parse_metadata(&self, html: &Html, url: &str) -> Result<MovieMetadata> {
        let mut metadata = MovieMetadata::default();

        // Parse info table
        let info = self.parse_info_table(html);

        // Extract content ID from URL
        if let Some(cid) = self.extract_cid_from_url(url) {
            metadata.number = cid.to_uppercase().replace("00", "-");
            // Simple heuristic: if it starts with letters, insert hyphen
            // Example: ssis00123 → SSIS-123
            if let Some(idx) = metadata.number.find(char::is_numeric) {
                if idx > 0 {
                    let (prefix, suffix) = metadata.number.split_at(idx);
                    metadata.number = format!("{}-{}", prefix, suffix.trim_start_matches('0'));
                }
            }
        }

        // Title: <h1 id="title">
        metadata.title = self.select_text(html, "h1#title");
        metadata.title = self.clean_text(&metadata.title);

        // Cover image: <img id="sample-video"> or <div class="img">
        let cover = self.select_attr_by_exprs(
            html,
            &["div.img img", "img#sample-video", "div.preview-images img"],
            "src",
        );
        if !cover.is_empty() {
            metadata.cover = if cover.starts_with("http") {
                cover
            } else if cover.starts_with("//") {
                format!("https:{}", cover)
            } else {
                format!("{}{}", self.base_url, cover)
            };
        }

        // Release Date: from info table
        if let Some(date) = info.get("発売日：").or_else(|| info.get("配信開始日：")) {
            metadata.release = date.clone();
        }

        // Runtime: from info table (e.g., "120分")
        if let Some(runtime) = info.get("収録時間：") {
            metadata.runtime = runtime.clone();
        }

        // Director: from info table
        if let Some(director) = info.get("監督：") {
            metadata.director = director.clone();
        }

        // Studio: from info table
        if let Some(studio) = info.get("メーカー：") {
            metadata.studio = studio.clone();
        }

        // Label: from info table
        if let Some(label) = info.get("レーベル：") {
            metadata.label = label.clone();
        }

        // Series: from info table
        if let Some(series) = info.get("シリーズ：") {
            metadata.series = series.clone();
        }

        // Actors: <span id="performer"> <a>
        metadata.actor = self.select_all_text(html, "span#performer a");
        metadata.actor.retain(|a| !a.is_empty());

        // Genres/Tags: <table class="mg-b20"> genre links
        metadata.tag = self.select_all_text(html, "table.mg-b20 a[href*='/genre/']");
        metadata.tag.retain(|t| !t.is_empty());

        // Outline/Description: Multiple possible locations
        let outline = self.select_text_by_exprs(
            html,
            &["div.mg-b20.lh4 p", "p.mg-b20", "div.txt.txt-product"],
        );
        if !outline.is_empty() {
            metadata.outline = self.clean_text(&outline);
        }

        // Validate essential fields
        if metadata.title.is_empty() {
            return Err(anyhow!("Failed to extract title from DMM"));
        }
        if metadata.cover.is_empty() {
            return Err(anyhow!("Failed to extract cover image from DMM"));
        }

        Ok(metadata)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_dmm_scraper_url() {
        let scraper = DmmScraper::new();
        let url = scraper.query_number_url("ssis00123").await.unwrap();
        assert!(url.contains("dmm.co.jp"));
        assert!(url.contains("ssis00123"));
    }

    #[test]
    fn test_dmm_extract_cid() {
        let scraper = DmmScraper::new();

        let url1 = "https://www.dmm.co.jp/mono/dvd/-/detail/=/cid=ssis00123/";
        assert_eq!(
            scraper.extract_cid_from_url(url1),
            Some("ssis00123".to_string())
        );

        let url2 = "https://www.dmm.co.jp/digital/videoa/-/detail/=/cid=abp00001/";
        assert_eq!(
            scraper.extract_cid_from_url(url2),
            Some("abp00001".to_string())
        );

        let url3 = "https://www.dmm.co.jp/search/";
        assert_eq!(scraper.extract_cid_from_url(url3), None);
    }

    #[test]
    fn test_dmm_scraper_parse() {
        let scraper = DmmScraper::new();

        let html_content = r#"
        <html>
        <head>
            <title>SSIS-123 Test Movie - DMM</title>
        </head>
        <body>
            <h1 id="title">テストJAVムービー</h1>
            <div class="img">
                <img src="//pics.dmm.co.jp/digital/video/ssis00123/ssis00123pl.jpg" />
            </div>
            <table class="mg-b20">
                <tr><td class="nw">発売日：</td><td>2024-01-15</td></tr>
                <tr><td class="nw">収録時間：</td><td>120分</td></tr>
                <tr><td class="nw">監督：</td><td>Test Director</td></tr>
                <tr><td class="nw">メーカー：</td><td>Test Studio</td></tr>
                <tr><td class="nw">レーベル：</td><td>Test Label</td></tr>
                <tr><td class="nw">シリーズ：</td><td>Test Series</td></tr>
            </table>
            <span id="performer">
                <a>Actress One</a>
                <a>Actress Two</a>
            </span>
            <table class="mg-b20">
                <tr><td><a href="/genre/1/">Drama</a></td></tr>
                <tr><td><a href="/genre/2/">Romance</a></td></tr>
            </table>
            <div class="mg-b20 lh4">
                <p>This is a test movie description for integration testing.</p>
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

        assert_eq!(metadata.title, "テストJAVムービー");
        assert_eq!(metadata.release, "2024-01-15");
        assert_eq!(metadata.runtime, "120分");
        assert_eq!(metadata.director, "Test Director");
        assert_eq!(metadata.studio, "Test Studio");
        assert_eq!(metadata.label, "Test Label");
        assert_eq!(metadata.series, "Test Series");
        assert!(metadata.cover.contains("ssis00123pl.jpg"));
        assert_eq!(metadata.actor.len(), 2);
        assert!(metadata.actor.contains(&"Actress One".to_string()));
        assert_eq!(metadata.tag.len(), 2);
        assert!(metadata.tag.contains(&"Drama".to_string()));
        assert!(metadata.outline.contains("test movie description"));
    }

    #[test]
    fn test_clean_text() {
        let scraper = DmmScraper::new();
        assert_eq!(scraper.clean_text("  Test   Movie  "), "Test Movie");
        assert_eq!(scraper.clean_text("Test\n\nMovie\n"), "Test Movie");
    }

    #[test]
    fn test_dmm_preferred_id_format() {
        let scraper = DmmScraper::new();
        assert_eq!(scraper.preferred_id_format(), IdFormat::Content);
    }
}
