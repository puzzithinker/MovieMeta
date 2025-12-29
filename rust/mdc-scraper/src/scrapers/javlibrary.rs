//! JAVLibrary scraper
//!
//! JAVLibrary (javlibrary.com) is one of the most comprehensive JAV metadata databases,
//! providing detailed information including actors, genres, studios, release dates, and covers.

use anyhow::{anyhow, Result};
use async_trait::async_trait;
use scraper::{Html, Selector};
use std::collections::HashMap;

use crate::metadata::MovieMetadata;
use crate::scraper::{IdFormat, Scraper};

/// JAVLibrary scraper implementation
pub struct JavlibraryScraper {
    /// Base URL (can be different mirrors)
    pub base_url: String,
}

impl JavlibraryScraper {
    /// Create a new JAVLibrary scraper with default URL
    pub fn new() -> Self {
        Self {
            base_url: "https://www.javlibrary.com".to_string(),
        }
    }

    /// Create scraper with custom base URL (for mirrors)
    pub fn with_base_url(base_url: String) -> Self {
        Self { base_url }
    }

    /// Parse info table to extract metadata fields
    fn parse_info_table(&self, html: &Html) -> HashMap<String, String> {
        let mut info = HashMap::new();

        // JAVLibrary uses a table with id="video_info" or similar
        // Structure: <td class="header">Label:</td><td class="text">Value</td>
        if let Ok(row_selector) = Selector::parse("div#video_info tr, div.item tr") {
            for row in html.select(&row_selector) {
                if let Ok(header_sel) = Selector::parse("td.header") {
                    if let Ok(text_sel) = Selector::parse("td.text") {
                        if let (Some(header), Some(text)) = (
                            row.select(&header_sel).next(),
                            row.select(&text_sel).next(),
                        ) {
                            let key = header.text().collect::<String>().trim().to_string();
                            let value = text.text().collect::<String>().trim().to_string();
                            if !key.is_empty() && !value.is_empty() {
                                info.insert(key, value);
                            }
                        }
                    }
                }
            }
        }

        info
    }

    /// Clean JAVLibrary text (removes extra whitespace and Japanese formatting)
    fn clean_text(&self, text: &str) -> String {
        text.split_whitespace()
            .collect::<Vec<_>>()
            .join(" ")
            .trim()
            .to_string()
    }
}

impl Default for JavlibraryScraper {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Scraper for JavlibraryScraper {
    fn source(&self) -> &str {
        "javlibrary"
    }

    fn imagecut(&self) -> i32 {
        1 // Smart crop with face detection (JAV covers usually have faces)
    }

    fn preferred_id_format(&self) -> IdFormat {
        // JAVLibrary requires content ID format (lowercase, zero-padded)
        // Example: "SSIS-123" should be passed as "ssis00123" in URLs
        IdFormat::Content
    }

    async fn query_number_url(&self, number: &str) -> Result<String> {
        // JAVLibrary search pattern: /ja/?v=javXXXXXX
        // We need to convert the number format
        let clean_number = number.to_uppercase().replace("-", "");

        // Try direct URL first (most common pattern)
        let search_url = format!("{}/en/?v=jav{}", self.base_url, clean_number);

        // TODO: Implement actual search if direct URL fails
        // For now, return direct URL
        Ok(search_url)
    }

    fn parse_metadata(&self, html: &Html, url: &str) -> Result<MovieMetadata> {
        let mut metadata = MovieMetadata::default();

        // Parse info table
        let info = self.parse_info_table(html);

        // Title: <h3 class="post-title text"> or <title>
        metadata.title = self.select_text(html, "h3.post-title.text");
        if metadata.title.is_empty() {
            // Fallback to page title, remove site name
            let title = self.select_text(html, "title");
            metadata.title = title
                .split('-')
                .next()
                .unwrap_or(&title)
                .trim()
                .to_string();
        }
        metadata.title = self.clean_text(&metadata.title);

        // Number: ID from info table or URL
        if let Some(id) = info.get("ID:").or_else(|| info.get("品番:")) {
            metadata.number = id.clone();
        } else {
            // Extract from URL: /ja/?v=javXXXXX
            if let Some(v_param) = url.split("v=").nth(1) {
                let id = v_param.split('&').next().unwrap_or("");
                metadata.number = id.replace("jav", "").to_uppercase();
            }
        }

        // Release Date: from info table
        if let Some(date) = info.get("Release Date:").or_else(|| info.get("発売日:")) {
            metadata.release = date.clone();
        }

        // Runtime: from info table (e.g., "120 min")
        if let Some(runtime) = info.get("Length:").or_else(|| info.get("収録時間:")) {
            metadata.runtime = runtime.clone();
        }

        // Director: from info table
        if let Some(director) = info.get("Director:").or_else(|| info.get("監督:")) {
            metadata.director = director.clone();
        }

        // Studio: from info table
        if let Some(studio) = info.get("Maker:").or_else(|| info.get("メーカー:")) {
            metadata.studio = studio.clone();
        }

        // Label: from info table
        if let Some(label) = info.get("Label:").or_else(|| info.get("レーベル:")) {
            metadata.label = label.clone();
        }

        // Series: from info table
        if let Some(series) = info.get("Series:").or_else(|| info.get("シリーズ:")) {
            metadata.series = series.clone();
        }

        // Cover image: <img id="video_jacket_img" src="...">
        let cover = self.select_attr(html, "img#video_jacket_img", "src");
        if !cover.is_empty() {
            metadata.cover = if cover.starts_with("http") {
                cover
            } else {
                format!("{}{}", self.base_url, cover)
            };
        }

        // Actors: <span class="cast"> <a>
        metadata.actor = self.select_all_text(html, "div#video_cast span.cast a, div.cast a");
        metadata.actor.retain(|a| !a.is_empty());

        // Genres/Tags: <span class="genre"> <a>
        metadata.tag = self.select_all_text(html, "div#video_genres span.genre a, div.genre a");
        metadata.tag.retain(|t| !t.is_empty());

        // Rating: JAVLibrary shows user ratings as stars
        let rating_str = self.select_attr(html, "span.score", "title");
        if !rating_str.is_empty() {
            // Parse formats like "7.5 (123 votes)"
            if let Some(rating_part) = rating_str.split('(').next() {
                if let Ok(rating) = rating_part.trim().parse::<f32>() {
                    metadata.userrating = rating;
                }
            }
        }

        // Outline/Description: May not be available on JAVLibrary
        // Usually just the title or tags serve as description
        if metadata.outline.is_empty() && !metadata.tag.is_empty() {
            metadata.outline = metadata.tag.join(", ");
        }

        // Validate essential fields
        if metadata.title.is_empty() {
            return Err(anyhow!("Failed to extract title from JAVLibrary"));
        }
        if metadata.cover.is_empty() {
            return Err(anyhow!("Failed to extract cover image from JAVLibrary"));
        }

        Ok(metadata)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_javlibrary_scraper_url() {
        let scraper = JavlibraryScraper::new();
        let url = scraper.query_number_url("ABP-001").await.unwrap();
        assert!(url.contains("javlibrary.com"));
        assert!(url.contains("ABP001") || url.contains("abp001"));
    }

    #[test]
    fn test_javlibrary_scraper_parse() {
        let scraper = JavlibraryScraper::new();

        let html_content = r#"
        <html>
        <head>
            <title>ABP-001 Test Movie - JAVLibrary</title>
        </head>
        <body>
            <h3 class="post-title text">Test JAV Movie タイトル</h3>
            <img id="video_jacket_img" src="/covers/abp001.jpg" />
            <div id="video_info">
                <table>
                    <tr><td class="header">ID:</td><td class="text">ABP-001</td></tr>
                    <tr><td class="header">Release Date:</td><td class="text">2024-01-15</td></tr>
                    <tr><td class="header">Length:</td><td class="text">120 min</td></tr>
                    <tr><td class="header">Director:</td><td class="text">Test Director</td></tr>
                    <tr><td class="header">Maker:</td><td class="text">Test Studio</td></tr>
                    <tr><td class="header">Label:</td><td class="text">Test Label</td></tr>
                    <tr><td class="header">Series:</td><td class="text">Test Series</td></tr>
                </table>
            </div>
            <div id="video_cast">
                <span class="cast"><a>Actress One</a></span>
                <span class="cast"><a>Actress Two</a></span>
            </div>
            <div id="video_genres">
                <span class="genre"><a>Drama</a></span>
                <span class="genre"><a>Romance</a></span>
            </div>
        </body>
        </html>
        "#;

        let html = Html::parse_document(html_content);
        let metadata = scraper
            .parse_metadata(&html, "https://www.javlibrary.com/en/?v=javabp001")
            .unwrap();

        assert_eq!(metadata.number, "ABP-001");
        assert!(metadata.title.contains("Test JAV Movie"));
        assert_eq!(metadata.release, "2024-01-15");
        assert_eq!(metadata.runtime, "120 min");
        assert_eq!(metadata.director, "Test Director");
        assert_eq!(metadata.studio, "Test Studio");
        assert_eq!(metadata.label, "Test Label");
        assert_eq!(metadata.series, "Test Series");
        assert!(metadata.cover.contains("abp001.jpg"));
        assert_eq!(metadata.actor.len(), 2);
        assert!(metadata.actor.contains(&"Actress One".to_string()));
        assert_eq!(metadata.tag.len(), 2);
        assert!(metadata.tag.contains(&"Drama".to_string()));
    }

    #[test]
    fn test_clean_text() {
        let scraper = JavlibraryScraper::new();
        assert_eq!(scraper.clean_text("  Test   Movie  "), "Test Movie");
        assert_eq!(
            scraper.clean_text("Test\n\nMovie\n"),
            "Test Movie"
        );
    }
}
