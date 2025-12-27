//! AVMOO scraper
//!
//! AVMOO (avmoo.com) is a popular JAV aggregator similar to JAVBus,
//! providing comprehensive metadata, covers, and actor information.

use anyhow::{anyhow, Result};
use async_trait::async_trait;
use scraper::{Html, Selector};

use crate::metadata::MovieMetadata;
use crate::scraper::Scraper;

/// AVMOO scraper implementation
pub struct AvmooScraper {
    /// Base URL (can be different mirrors)
    pub base_url: String,
}

impl AvmooScraper {
    /// Create a new AVMOO scraper with default URL
    pub fn new() -> Self {
        Self {
            base_url: "https://avmoo.com".to_string(),
        }
    }

    /// Create scraper with custom base URL (for mirrors)
    pub fn with_base_url(base_url: String) -> Self {
        Self { base_url }
    }

    /// Extract text after a label in info panel
    fn extract_info_value(&self, html: &Html, label: &str) -> String {
        // AVMOO uses structure like: <p><span class="header">Label:</span> Value</p>
        if let Ok(selector) = Selector::parse("div.info p, div.row p") {
            for p in html.select(&selector) {
                let text = p.text().collect::<String>();
                if text.contains(label) {
                    // Extract text after the label
                    if let Some(value) = text.split(label).nth(1) {
                        return value.trim().to_string();
                    }
                }
            }
        }
        String::new()
    }

    /// Clean and normalize text
    fn clean_text(&self, text: &str) -> String {
        text.split_whitespace()
            .collect::<Vec<_>>()
            .join(" ")
            .trim()
            .to_string()
    }
}

impl Default for AvmooScraper {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Scraper for AvmooScraper {
    fn source(&self) -> &str {
        "avmoo"
    }

    fn imagecut(&self) -> i32 {
        1 // Smart crop with face detection
    }

    async fn query_number_url(&self, number: &str) -> Result<String> {
        // AVMOO URL pattern: /movie/ABC-001
        let clean_number = number.to_uppercase();
        Ok(format!("{}/movie/{}", self.base_url, clean_number))
    }

    fn parse_metadata(&self, html: &Html, url: &str) -> Result<MovieMetadata> {
        let mut metadata = MovieMetadata::default();

        // Title: <h3>Movie Title</h3> in container
        metadata.title = self.select_text(html, "div.container h3, h3.title, div.movie h3");
        metadata.title = self.clean_text(&metadata.title);

        // Number: Extract from URL or title
        if let Some(number_part) = url.split('/').last() {
            metadata.number = number_part.to_uppercase();
        }

        // If title contains the number, remove it for cleaner title
        if !metadata.number.is_empty() && metadata.title.starts_with(&metadata.number) {
            metadata.title = metadata
                .title
                .replacen(&metadata.number, "", 1)
                .trim()
                .to_string();
        }

        // Cover image: <a class="bigImage"> <img src="...">
        let cover = self.select_attr(
            html,
            "a.bigImage img, img.bigImage, div.screencap img, img#video_jacket_img",
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

        // Info panel parsing
        // Release Date
        metadata.release = self.extract_info_value(html, "發行日期:");
        if metadata.release.is_empty() {
            metadata.release = self.extract_info_value(html, "Release Date:");
        }
        if metadata.release.is_empty() {
            metadata.release = self.extract_info_value(html, "日期:");
        }

        // Runtime
        metadata.runtime = self.extract_info_value(html, "長度:");
        if metadata.runtime.is_empty() {
            metadata.runtime = self.extract_info_value(html, "Runtime:");
        }
        if metadata.runtime.is_empty() {
            metadata.runtime = self.extract_info_value(html, "時長:");
        }

        // Director
        metadata.director = self.extract_info_value(html, "導演:");
        if metadata.director.is_empty() {
            metadata.director = self.extract_info_value(html, "Director:");
        }

        // Studio/Maker
        metadata.studio = self.extract_info_value(html, "製作商:");
        if metadata.studio.is_empty() {
            metadata.studio = self.extract_info_value(html, "Studio:");
            if metadata.studio.is_empty() {
                metadata.studio = self.extract_info_value(html, "Maker:");
            }
        }

        // Label
        metadata.label = self.extract_info_value(html, "發行商:");
        if metadata.label.is_empty() {
            metadata.label = self.extract_info_value(html, "Label:");
        }
        if metadata.label.is_empty() {
            metadata.label = self.extract_info_value(html, "系列:");
        }

        // Series
        metadata.series = self.extract_info_value(html, "系列:");
        if metadata.series.is_empty() {
            metadata.series = self.extract_info_value(html, "Series:");
        }

        // Actors: <div class="star-name"> or <a> with actor links
        metadata.actor = self.select_all_text(
            html,
            "div.star-name a, div#avatar-waterfall a .photo-info span, .avatar-box span, span.star a",
        );
        metadata.actor.retain(|a| !a.is_empty() && a.len() > 1);

        // Genres/Tags: <span class="genre"> <a> or <label>
        metadata.tag = self.select_all_text(html, "span.genre a, p.header a.genre, div.genre a");
        metadata.tag.retain(|t| !t.is_empty());

        // Rating: Look for star ratings or numeric scores
        let rating_container = self.select_text(html, "span.score, div.rating, span.userrating");
        if !rating_container.is_empty() {
            if let Ok(rating) = rating_container.trim().parse::<f32>() {
                metadata.userrating = rating;
            }
        }

        // Outline: Usually constructed from genres
        if metadata.outline.is_empty() && !metadata.tag.is_empty() {
            metadata.outline = metadata.tag.join(", ");
        }

        // Validate essential fields
        if metadata.title.is_empty() {
            return Err(anyhow!("Failed to extract title from AVMOO"));
        }
        if metadata.cover.is_empty() {
            return Err(anyhow!("Failed to extract cover image from AVMOO"));
        }

        Ok(metadata)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_avmoo_scraper_url() {
        let scraper = AvmooScraper::new();
        let url = scraper.query_number_url("ABP-001").await.unwrap();
        assert!(url.contains("avmoo.com"));
        assert!(url.contains("ABP-001"));
    }

    #[test]
    fn test_avmoo_scraper_parse() {
        let scraper = AvmooScraper::new();

        let html_content = r#"
        <html>
        <head>
            <title>ABP-001 - AVMOO</title>
        </head>
        <body>
            <div class="container">
                <h3>ABP-001 Test Movie Title</h3>
                <a class="bigImage">
                    <img src="https://example.com/covers/abp001.jpg" />
                </a>
                <div class="info">
                    <p><span class="header">發行日期:</span> 2024-01-15</p>
                    <p><span class="header">長度:</span> 120分鐘</p>
                    <p><span class="header">導演:</span> Test Director</p>
                    <p><span class="header">製作商:</span> Test Studio</p>
                    <p><span class="header">發行商:</span> Test Label</p>
                    <p><span class="header">系列:</span> Test Series</p>
                </div>
                <div id="avatar-waterfall">
                    <a><div class="photo-info"><span>Actress One</span></div></a>
                    <a><div class="photo-info"><span>Actress Two</span></div></a>
                </div>
                <div class="genre">
                    <a>Drama</a>
                    <a>Romance</a>
                </div>
            </div>
        </body>
        </html>
        "#;

        let html = Html::parse_document(html_content);
        let metadata = scraper
            .parse_metadata(&html, "https://avmoo.com/movie/ABP-001")
            .unwrap();

        assert_eq!(metadata.number, "ABP-001");
        assert!(metadata.title.contains("Test Movie Title"));
        assert_eq!(metadata.release, "2024-01-15");
        assert_eq!(metadata.runtime, "120分鐘");
        assert_eq!(metadata.director, "Test Director");
        assert_eq!(metadata.studio, "Test Studio");
        assert!(metadata.cover.contains("abp001.jpg"));
        assert_eq!(metadata.actor.len(), 2);
        assert_eq!(metadata.tag.len(), 2);
    }

    #[test]
    fn test_clean_text() {
        let scraper = AvmooScraper::new();
        assert_eq!(scraper.clean_text("  Test   Movie  "), "Test Movie");
    }
}
