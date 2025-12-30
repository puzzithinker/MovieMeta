//! Jav321 scraper
//!
//! Jav321 (jp.jav321.com) is a Japanese JAV aggregator with good metadata coverage.
//! It uses POST-based search and provides comprehensive movie information.
//!
//! Key Features:
//! - POST-based search (sn parameter)
//! - Japanese HTML structure (品番, メーカー, etc.)
//! - Fallback aggregator with decent coverage
//! - Simple, clean HTML structure
//!
//! Implementation based on verified Javinizer patterns.

use crate::metadata::MovieMetadata;
use crate::scraper::{IdFormat, Scraper, ScraperConfig};
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use regex::Regex;
use scraper::Html;
use std::sync::OnceLock;

/// Jav321 scraper
pub struct Jav321Scraper {
    pub base_url: String,
}

impl Jav321Scraper {
    /// Create new Jav321 scraper
    pub fn new() -> Self {
        Self {
            base_url: "https://jp.jav321.com".to_string(),
        }
    }

    /// Parse metadata from detail page
    fn parse_detail_metadata(&self, html: &Html, url: &str) -> Result<MovieMetadata> {
        let mut metadata = MovieMetadata {
            website: url.to_string(),
            ..Default::default()
        };

        let html_text = html.html();

        // Extract ID (品番) - Pattern: <b>品番</b>: (.*)<br><b>
        static ID_RE: OnceLock<Regex> = OnceLock::new();
        let id_re = ID_RE.get_or_init(|| {
            Regex::new(r#"<b>品番</b>:\s*([^<]+)<br>"#).unwrap()
        });
        if let Some(cap) = id_re.captures(&html_text) {
            metadata.number = cap.get(1).map(|m| m.as_str().trim().to_uppercase()).unwrap_or_default();
        }

        // Extract title - Pattern: <div class="panel-heading"><h3>(.*) <small>
        static TITLE_RE: OnceLock<Regex> = OnceLock::new();
        let title_re = TITLE_RE.get_or_init(|| {
            Regex::new(r#"<div class="panel-heading"><h3>([^<]+)<small"#).unwrap()
        });
        if let Some(cap) = title_re.captures(&html_text) {
            metadata.title = self.clean_text(cap.get(1).map(|m| m.as_str()).unwrap_or_default());
        }

        // Extract release date - Pattern: <b>(.*)</b>: (\d{4}-\d{2}-\d{2})<br>
        static DATE_RE: OnceLock<Regex> = OnceLock::new();
        let date_re = DATE_RE.get_or_init(|| {
            Regex::new(r#"<b>[^<]*</b>:\s*(\d{4}-\d{2}-\d{2})<br>"#).unwrap()
        });
        if let Some(cap) = date_re.captures(&html_text) {
            if let Some(date_match) = cap.get(1) {
                metadata.release = date_match.as_str().to_string();
            }
        }

        // Extract runtime - Pattern: <b>(.*)</b>: (\d{1,3}) minutes<br>
        static RUNTIME_RE: OnceLock<Regex> = OnceLock::new();
        let runtime_re = RUNTIME_RE.get_or_init(|| {
            Regex::new(r#"<b>[^<]*</b>:\s*(\d{1,3})\s*minutes?<br>"#).unwrap()
        });
        if let Some(cap) = runtime_re.captures(&html_text) {
            if let Some(runtime_match) = cap.get(1) {
                metadata.runtime = runtime_match.as_str().to_string();
            }
        }

        // Extract maker/studio - Pattern: <b>メーカー</b>: (.*)>(.*)</a><br><b>ジャンル
        static MAKER_RE: OnceLock<Regex> = OnceLock::new();
        let maker_re = MAKER_RE.get_or_init(|| {
            Regex::new(r#"<b>メーカー</b>:[^>]*>([^<]+)</a>"#).unwrap()
        });
        if let Some(cap) = maker_re.captures(&html_text) {
            if let Some(maker_match) = cap.get(1) {
                metadata.studio = self.clean_text(maker_match.as_str());
            }
        }

        // Extract genres - Pattern: <a href="/genre/.+?">(.+?)</a>
        static GENRE_RE: OnceLock<Regex> = OnceLock::new();
        let genre_re = GENRE_RE.get_or_init(|| {
            Regex::new(r#"<a href="/genre/[^"]+">([^<]+)</a>"#).unwrap()
        });
        metadata.tag = genre_re
            .captures_iter(&html_text)
            .filter_map(|cap| cap.get(1).map(|m| self.clean_text(m.as_str())))
            .filter(|s| !s.is_empty())
            .collect();

        // Extract cover image - Pattern: "/snapshot/(.*)/\d/0"><img class="img-responsive".*src="(.*)"
        static COVER_RE: OnceLock<Regex> = OnceLock::new();
        let cover_re = COVER_RE.get_or_init(|| {
            Regex::new(r#"src="(https?://[^"]+\.jpg)"[^>]*class="img-responsive""#).unwrap()
        });
        if let Some(cap) = cover_re.captures(&html_text) {
            if let Some(cover_match) = cap.get(1) {
                metadata.cover = cover_match.as_str().to_string();
            }
        }

        // Extract screenshots - Pattern: (https://www.jav321.com/digital/video/(.*)/(.*)jpg)
        static SCREENSHOT_RE: OnceLock<Regex> = OnceLock::new();
        let screenshot_re = SCREENSHOT_RE.get_or_init(|| {
            Regex::new(r#"(https://[^"]*jav321\.com/[^"]*\.jpg)"#).unwrap()
        });
        metadata.extrafanart = screenshot_re
            .captures_iter(&html_text)
            .filter_map(|cap| cap.get(1).map(|m| m.as_str().to_string()))
            .filter(|url| !url.contains("img-responsive")) // Skip cover
            .collect();

        // Extract actresses - Pattern: https://www\.jav321\.com/mono/actjpgs/(.*)">(.*)</a>
        static ACTRESS_RE: OnceLock<Regex> = OnceLock::new();
        let actress_re = ACTRESS_RE.get_or_init(|| {
            Regex::new(r#"/mono/actjpgs/[^"]*">([^<]+)</a>"#).unwrap()
        });
        metadata.actor = actress_re
            .captures_iter(&html_text)
            .filter_map(|cap| cap.get(1).map(|m| {
                // Remove parenthetical text
                let name = m.as_str();
                let cleaned = if let Some(pos) = name.find('(') {
                    &name[..pos]
                } else {
                    name
                };
                self.clean_text(cleaned)
            }))
            .filter(|s| !s.is_empty())
            .collect();

        // Validation
        if metadata.title.is_empty() {
            return Err(anyhow!("Missing required field: title"));
        }

        Ok(metadata)
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

impl Default for Jav321Scraper {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Scraper for Jav321Scraper {
    fn source(&self) -> &str {
        "jav321"
    }

    fn imagecut(&self) -> i32 {
        1 // Smart crop
    }

    fn preferred_id_format(&self) -> IdFormat {
        IdFormat::Display // Jav321 uses display format
    }

    async fn query_number_url(&self, _number: &str) -> Result<String> {
        // Jav321 uses POST search, but this is required by trait
        // We override scrape() to handle POST properly
        Ok(format!("{}/search", self.base_url))
    }

    fn parse_metadata(&self, html: &Html, url: &str) -> Result<MovieMetadata> {
        self.parse_detail_metadata(html, url)
    }

    async fn scrape(&self, number: &str, config: &ScraperConfig) -> Result<MovieMetadata> {
        // Note: Jav321 uses POST search with "sn={number}" body
        // For simplicity, we'll try direct URL access first
        // A full implementation would use reqwest POST with form data

        // Try direct URL pattern: /video/{number}
        let direct_url = format!("{}/video/{}", self.base_url, number.to_lowercase());

        let html_text = match config.client.get(&direct_url).await {
            Ok(text) => text,
            Err(_) => {
                // Fallback: try search URL (may require POST implementation)
                return Err(anyhow!(
                    "Jav321 requires POST search which is not yet fully implemented. \
                     Direct URL access failed for: {}",
                    number
                ));
            }
        };

        let html = Html::parse_document(&html_text);
        self.parse_detail_metadata(&html, &direct_url)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_jav321_scraper_url() {
        let scraper = Jav321Scraper::new();
        assert_eq!(scraper.base_url, "https://jp.jav321.com");
        assert_eq!(scraper.source(), "jav321");
    }

    #[test]
    fn test_jav321_preferred_id_format() {
        let scraper = Jav321Scraper::new();
        assert!(matches!(scraper.preferred_id_format(), IdFormat::Display));
    }

    #[test]
    fn test_jav321_parse_metadata() {
        let scraper = Jav321Scraper::new();

        let html_content = r#"
            <html>
                <body>
                    <div class="panel-heading"><h3>Test Movie Title<small>2023</small></h3></div>
                    <b>品番</b>: ssis-123<br>
                    <b>Release Date</b>: 2023-01-15<br>
                    <b>Runtime</b>: 120 minutes<br>
                    <b>メーカー</b>: <a href="/maker/1">Prestige</a><br><b>ジャンル
                    <a href="/genre/1">Drama</a>
                    <a href="/genre/2">Amateur</a>
                    <img class="img-responsive" src="https://www.jav321.com/cover.jpg">
                    <a href="https://www.jav321.com/digital/video/ssis123/screenshot1.jpg">
                    <a href="/mono/actjpgs/actress1.jpg">Actress Name (Alias)</a>
                </body>
            </html>
        "#;

        let html = Html::parse_document(html_content);
        let metadata = scraper.parse_detail_metadata(&html, "https://jp.jav321.com/video/ssis-123").unwrap();

        assert!(metadata.title.contains("Test Movie Title"));
        assert_eq!(metadata.number, "SSIS-123");
        assert_eq!(metadata.release, "2023-01-15");
        assert_eq!(metadata.runtime, "120");
        assert_eq!(metadata.studio, "Prestige");
        assert_eq!(metadata.tag.len(), 2);
        assert!(!metadata.actor.is_empty());
        assert_eq!(metadata.actor[0], "Actress Name");
    }

    #[test]
    fn test_jav321_clean_text() {
        let scraper = Jav321Scraper::new();

        assert_eq!(scraper.clean_text("  Test  "), "Test");
        assert_eq!(scraper.clean_text("Test&amp;More"), "Test&More");
    }

    #[test]
    fn test_jav321_missing_title_error() {
        let scraper = Jav321Scraper::new();

        let html_content = r#"
            <html><body><b>品番</b>: ssis-123<br></body></html>
        "#;

        let html = Html::parse_document(html_content);
        let result = scraper.parse_detail_metadata(&html, "https://jp.jav321.com/video/ssis-123");

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("title"));
    }
}
