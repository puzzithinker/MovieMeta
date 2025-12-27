//! FC2 Club scraper
//!
//! FC2 Club is specialized for FC2-PPV content, which has a unique numbering format
//! (FC2-PPV-1234567) and different metadata structure compared to traditional JAV.

use anyhow::{anyhow, Result};
use async_trait::async_trait;
use scraper::Html;

use crate::metadata::MovieMetadata;
use crate::scraper::Scraper;

/// FC2 Club scraper implementation
pub struct Fc2Scraper {
    /// Base URL
    pub base_url: String,
}

impl Fc2Scraper {
    /// Create a new FC2 scraper with default URL
    pub fn new() -> Self {
        Self {
            base_url: "https://adult.contents.fc2.com".to_string(),
        }
    }

    /// Create scraper with custom base URL
    pub fn with_base_url(base_url: String) -> Self {
        Self { base_url }
    }

    /// Extract FC2 ID from various formats
    fn extract_fc2_id(&self, number: &str) -> String {
        // Handle formats:
        // FC2-PPV-1234567 -> 1234567
        // FC2-1234567 -> 1234567
        // FC2PPV1234567 -> 1234567
        // 1234567 -> 1234567
        let clean = number
            .to_uppercase()
            .replace("FC2-PPV-", "")
            .replace("FC2-", "")
            .replace("FC2PPV", "")
            .replace("FC2", "");

        clean.trim().to_string()
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

impl Default for Fc2Scraper {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Scraper for Fc2Scraper {
    fn source(&self) -> &str {
        "fc2"
    }

    fn imagecut(&self) -> i32 {
        1 // Smart crop with face detection
    }

    async fn query_number_url(&self, number: &str) -> Result<String> {
        // FC2 URL pattern: /article/1234567/
        let fc2_id = self.extract_fc2_id(number);
        Ok(format!("{}/article/{}/", self.base_url, fc2_id))
    }

    fn parse_metadata(&self, html: &Html, url: &str) -> Result<MovieMetadata> {
        let mut metadata = MovieMetadata::default();

        // Extract FC2 ID from URL
        if let Some(id_part) = url.split("/article/").nth(1) {
            if let Some(id) = id_part.split('/').next() {
                metadata.number = format!("FC2-PPV-{}", id);
            }
        }

        // Title: <h2 class="items_article_headerInfo"> or <h3>
        metadata.title = self.select_text(
            html,
            "h2.items_article_headerInfo, h3.items_article_headerInfo, div.items_article_MainitemsTitle h3",
        );
        metadata.title = self.clean_text(&metadata.title);

        // FC2 titles often don't include the number, so it's already clean

        // Cover image: <div class="items_article_MainitemThumb"> <img>
        let cover = self.select_attr(
            html,
            "div.items_article_MainitemThumb img, img.items_article_MainitemThumb, meta[property='og:image']",
            "src",
        );
        if cover.is_empty() {
            // Try og:image as fallback
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

        // Release Date: <p class="items_article_info">
        let info_text = self.select_text(html, "p.items_article_info, div.items_article_info");
        if info_text.contains("販売日:") || info_text.contains("Sales Date:") {
            if let Some(date_part) = info_text.split("販売日:").nth(1) {
                metadata.release = date_part
                    .split_whitespace()
                    .next()
                    .unwrap_or("")
                    .to_string();
            } else if let Some(date_part) = info_text.split("Sales Date:").nth(1) {
                metadata.release = date_part
                    .split_whitespace()
                    .next()
                    .unwrap_or("")
                    .to_string();
            }
        }

        // Runtime: Often in minutes, look for pattern like "60分"
        if info_text.contains("再生時間:") || info_text.contains("Runtime:") {
            if let Some(runtime_part) = info_text.split("再生時間:").nth(1) {
                let runtime_str = runtime_part
                    .split_whitespace()
                    .next()
                    .unwrap_or("")
                    .to_string();
                metadata.runtime = runtime_str;
            } else if let Some(runtime_part) = info_text.split("Runtime:").nth(1) {
                let runtime_str = runtime_part
                    .split_whitespace()
                    .next()
                    .unwrap_or("")
                    .to_string();
                metadata.runtime = runtime_str;
            }
        }

        // FC2 usually doesn't have formal studios, use "FC2" as studio
        metadata.studio = "FC2".to_string();
        metadata.label = "FC2-PPV".to_string();

        // Seller/Producer as director (FC2 specific)
        let seller = self.select_text(html, "a.items_article_SellerName, div.seller a");
        if !seller.is_empty() {
            metadata.director = seller;
        }

        // Tags: FC2 uses different tag structure
        metadata.tag = self.select_all_text(html, "a.tag, a.items_article_tag, div.tag a");
        metadata.tag.retain(|t| !t.is_empty());

        // Description/Outline
        metadata.outline = self.select_text(
            html,
            "div.items_article_MainitemComment, div.description, p.items_comment",
        );
        metadata.outline = self.clean_text(&metadata.outline);

        // FC2 often has amateur content, so actors might not be listed
        // Use tags or description for context
        if metadata.actor.is_empty() {
            metadata.actor = vec!["Amateur".to_string()];
        }

        // Validate essential fields
        if metadata.title.is_empty() {
            return Err(anyhow!("Failed to extract title from FC2"));
        }
        // FC2 might not always have cover images available
        if metadata.cover.is_empty() {
            tracing::warn!("No cover image found for FC2 content");
        }

        Ok(metadata)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_fc2_scraper_url() {
        let scraper = Fc2Scraper::new();
        let url = scraper.query_number_url("FC2-PPV-1234567").await.unwrap();
        assert!(url.contains("fc2.com"));
        assert!(url.contains("1234567"));
    }

    #[test]
    fn test_extract_fc2_id() {
        let scraper = Fc2Scraper::new();
        assert_eq!(scraper.extract_fc2_id("FC2-PPV-1234567"), "1234567");
        assert_eq!(scraper.extract_fc2_id("FC2-1234567"), "1234567");
        assert_eq!(scraper.extract_fc2_id("FC2PPV1234567"), "1234567");
        assert_eq!(scraper.extract_fc2_id("1234567"), "1234567");
    }

    #[test]
    fn test_fc2_scraper_parse() {
        let scraper = Fc2Scraper::new();

        let html_content = r#"
        <html>
        <head>
            <title>FC2-PPV-1234567</title>
            <meta property="og:image" content="https://example.com/fc2-cover.jpg" />
        </head>
        <body>
            <h2 class="items_article_headerInfo">Test FC2 Movie Title</h2>
            <div class="items_article_MainitemThumb">
                <img src="https://example.com/thumbnail.jpg" />
            </div>
            <p class="items_article_info">
                販売日: 2024-01-15
                再生時間: 60分
            </p>
            <a class="items_article_SellerName">Test Seller</a>
            <div class="tag">
                <a>Amateur</a>
                <a>POV</a>
            </div>
            <div class="items_article_MainitemComment">
                This is a test FC2 movie description with details about the content.
            </div>
        </body>
        </html>
        "#;

        let html = Html::parse_document(html_content);
        let metadata = scraper
            .parse_metadata(&html, "https://adult.contents.fc2.com/article/1234567/")
            .unwrap();

        assert_eq!(metadata.number, "FC2-PPV-1234567");
        assert!(metadata.title.contains("Test FC2 Movie Title"));
        assert_eq!(metadata.release, "2024-01-15");
        assert_eq!(metadata.runtime, "60分");
        assert_eq!(metadata.studio, "FC2");
        assert_eq!(metadata.label, "FC2-PPV");
        assert_eq!(metadata.director, "Test Seller");
        // Cover can be either thumbnail or og:image
        assert!(!metadata.cover.is_empty());
        assert!(
            metadata.cover.contains("thumbnail.jpg") || metadata.cover.contains("fc2-cover.jpg")
        );
        assert_eq!(metadata.tag.len(), 2);
        assert!(!metadata.outline.is_empty());
    }

    #[test]
    fn test_clean_text() {
        let scraper = Fc2Scraper::new();
        assert_eq!(scraper.clean_text("  Test   Movie  "), "Test Movie");
    }
}
