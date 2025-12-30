//! Tokyo-Hot scraper
//!
//! Tokyo-Hot (tokyo-hot.com) is a premium JAV studio with unique numbering formats
//! (n0001, k0001, etc.) and official metadata on their site.

use anyhow::{anyhow, Result};
use async_trait::async_trait;
use scraper::Html;

use crate::metadata::MovieMetadata;
use crate::scraper::Scraper;

/// Tokyo-Hot scraper implementation
pub struct TokyohotScraper {
    /// Base URL
    pub base_url: String,
}

impl TokyohotScraper {
    /// Create a new Tokyo-Hot scraper with default URL
    pub fn new() -> Self {
        Self {
            base_url: "https://www.tokyo-hot.com".to_string(),
        }
    }

    /// Create scraper with custom base URL
    pub fn with_base_url(base_url: String) -> Self {
        Self { base_url }
    }

    /// Normalize Tokyo-Hot ID format
    fn normalize_id(&self, number: &str) -> String {
        // Handle formats:
        // TOKYO-HOT-N0001 -> n0001
        // N0001 -> n0001
        // n0001 -> n0001 (already correct)
        let clean = number
            .to_lowercase()
            .replace("tokyo-hot-", "")
            .replace("tokyohot-", "")
            .replace("tokyo-hot", "")
            .replace("tokyohot", "");

        clean.trim().to_string()
    }

    /// Extract info from dt/dd pairs
    fn extract_info_from_dl(&self, html: &Html) -> std::collections::HashMap<String, String> {
        let mut info = std::collections::HashMap::new();

        if let Ok(dl_selector) = scraper::Selector::parse("dl.info, dl.detail") {
            for dl in html.select(&dl_selector) {
                if let (Ok(dt_sel), Ok(dd_sel)) = (
                    scraper::Selector::parse("dt"),
                    scraper::Selector::parse("dd"),
                ) {
                    let dts: Vec<_> = dl.select(&dt_sel).collect();
                    let dds: Vec<_> = dl.select(&dd_sel).collect();

                    for (dt, dd) in dts.iter().zip(dds.iter()) {
                        let key = dt.text().collect::<String>().trim().to_string();
                        let value = dd.text().collect::<String>().trim().to_string();
                        if !key.is_empty() && !value.is_empty() {
                            info.insert(key, value);
                        }
                    }
                }
            }
        }

        info
    }

    /// Clean text
    fn clean_text(&self, text: &str) -> String {
        text.split_whitespace()
            .collect::<Vec<_>>()
            .join(" ")
            .trim()
            .to_string()
    }
}

impl Default for TokyohotScraper {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Scraper for TokyohotScraper {
    fn source(&self) -> &str {
        "tokyohot"
    }

    fn imagecut(&self) -> i32 {
        1 // Smart crop with face detection
    }

    async fn query_number_url(&self, number: &str) -> Result<String> {
        // Tokyo-Hot URL pattern: /product/?q=n0001
        let th_id = self.normalize_id(number);
        Ok(format!("{}/product/?q={}", self.base_url, th_id))
    }

    fn parse_metadata(&self, html: &Html, url: &str) -> Result<MovieMetadata> {
        let mut metadata = MovieMetadata::default();

        // Extract ID from URL
        if let Some(query_part) = url.split("?q=").nth(1) {
            let id = query_part.split('&').next().unwrap_or("").to_lowercase();
            metadata.number = format!("Tokyo-Hot {}", id.to_uppercase());
        }

        // Title: <h2 class="title"> or <div class="title">
        metadata.title = self.select_text(html, "h2.title, div.title h2, div.detail h2");
        metadata.title = self.clean_text(&metadata.title);

        // Cover image: <img class="package"> or main product image
        let cover = self.select_attr(
            html,
            "img.package, div.package img, img.product-image, meta[property='og:image']",
            "src",
        );
        if cover.is_empty() {
            let og_image = self.select_attr(html, "meta[property='og:image']", "content");
            if !og_image.is_empty() {
                metadata.cover = og_image;
            }
        } else {
            metadata.cover = if cover.starts_with("http") {
                cover
            } else if cover.starts_with("//") {
                format!("https:{}", cover)
            } else {
                format!("{}{}", self.base_url, cover)
            };
        }

        // Parse info from dl/dt/dd structure
        let info = self.extract_info_from_dl(html);

        // Release Date
        if let Some(date) = info
            .get("配信開始日:")
            .or_else(|| info.get("Release Date:"))
            .or_else(|| info.get("配信日:"))
        {
            metadata.release = date.clone();
        }

        // Runtime
        if let Some(runtime) = info
            .get("収録時間:")
            .or_else(|| info.get("Runtime:"))
            .or_else(|| info.get("再生時間:"))
        {
            metadata.runtime = runtime.clone();
        }

        // Studio is always Tokyo-Hot
        metadata.studio = "Tokyo-Hot".to_string();
        metadata.label = "Tokyo-Hot".to_string();

        // Director
        if let Some(director) = info.get("監督:").or_else(|| info.get("Director:")) {
            metadata.director = director.clone();
        }

        // Series
        if let Some(series) = info.get("シリーズ:").or_else(|| info.get("Series:")) {
            metadata.series = series.clone();
        }

        // Actresses: <a class="actress"> or in cast list
        metadata.actor =
            self.select_all_text(html, "a.actress, div.cast a, dl.cast dd a, span.actress a");
        metadata.actor.retain(|a| !a.is_empty() && a.len() > 1);

        // Genres/Tags: <a class="genre"> or category links
        metadata.tag = self.select_all_text(html, "a.genre, div.genre a, a.category");
        metadata.tag.retain(|t| !t.is_empty());

        // Description/Outline
        metadata.outline = self.select_text(
            html,
            "div.description, p.description, div.summary, div.comment",
        );
        metadata.outline = self.clean_text(&metadata.outline);

        // Tokyo-Hot is always uncensored
        metadata.tag.push("Uncensored".to_string());

        // Validate essential fields
        if metadata.title.is_empty() {
            return Err(anyhow!("Failed to extract title from Tokyo-Hot"));
        }
        if metadata.cover.is_empty() {
            tracing::warn!("No cover image found for Tokyo-Hot content");
        }

        Ok(metadata)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_tokyohot_scraper_url() {
        let scraper = TokyohotScraper::new();
        let url = scraper.query_number_url("n0001").await.unwrap();
        assert!(url.contains("tokyo-hot.com"));
        assert!(url.contains("n0001"));
    }

    #[test]
    fn test_normalize_id() {
        let scraper = TokyohotScraper::new();
        assert_eq!(scraper.normalize_id("TOKYO-HOT-N0001"), "n0001");
        assert_eq!(scraper.normalize_id("N0001"), "n0001");
        assert_eq!(scraper.normalize_id("n0001"), "n0001");
        assert_eq!(scraper.normalize_id("k1234"), "k1234");
    }

    #[test]
    fn test_tokyohot_scraper_parse() {
        let scraper = TokyohotScraper::new();

        let html_content = r#"
        <html>
        <head>
            <title>Tokyo-Hot n0001</title>
            <meta property="og:image" content="https://example.com/th-cover.jpg" />
        </head>
        <body>
            <h2 class="title">Test Tokyo-Hot Movie Title</h2>
            <img class="package" src="https://example.com/package.jpg" />
            <dl class="info">
                <dt>配信開始日:</dt><dd>2024-01-15</dd>
                <dt>収録時間:</dt><dd>120分</dd>
                <dt>監督:</dt><dd>Test Director</dd>
                <dt>シリーズ:</dt><dd>Test Series</dd>
            </dl>
            <div class="cast">
                <a class="actress">Actress One</a>
                <a class="actress">Actress Two</a>
            </div>
            <div class="genre">
                <a class="genre">Hardcore</a>
                <a class="genre">Creampie</a>
            </div>
            <div class="description">
                This is a test Tokyo-Hot movie description with uncensored content.
            </div>
        </body>
        </html>
        "#;

        let html = Html::parse_document(html_content);
        let metadata = scraper
            .parse_metadata(&html, "https://www.tokyo-hot.com/product/?q=n0001")
            .unwrap();

        assert_eq!(metadata.number, "Tokyo-Hot N0001");
        assert!(metadata.title.contains("Test Tokyo-Hot Movie Title"));
        assert_eq!(metadata.release, "2024-01-15");
        assert_eq!(metadata.runtime, "120分");
        assert_eq!(metadata.studio, "Tokyo-Hot");
        assert_eq!(metadata.director, "Test Director");
        assert_eq!(metadata.series, "Test Series");
        // Cover can be either package or og:image
        assert!(!metadata.cover.is_empty());
        assert!(metadata.cover.contains("package.jpg") || metadata.cover.contains("th-cover.jpg"));
        assert_eq!(metadata.actor.len(), 2);
        assert!(metadata.tag.len() >= 3); // 2 genres + "Uncensored"
        assert!(metadata.tag.contains(&"Uncensored".to_string()));
        assert!(!metadata.outline.is_empty());
    }

    #[test]
    fn test_clean_text() {
        let scraper = TokyohotScraper::new();
        assert_eq!(scraper.clean_text("  Test   Movie  "), "Test Movie");
    }
}
