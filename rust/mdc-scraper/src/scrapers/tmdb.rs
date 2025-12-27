//! TMDB (The Movie Database) scraper

use anyhow::Result;
use async_trait::async_trait;
use scraper::Html;

use crate::metadata::MovieMetadata;
use crate::scraper::Scraper;

/// TMDB scraper implementation
pub struct TmdbScraper {
    /// Optional API key (not currently used)
    pub api_key: Option<String>,
}

impl TmdbScraper {
    /// Create a new TMDB scraper
    pub fn new() -> Self {
        Self { api_key: None }
    }

    /// Create scraper with API key
    pub fn with_api_key(api_key: String) -> Self {
        Self {
            api_key: Some(api_key),
        }
    }
}

impl Default for TmdbScraper {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Scraper for TmdbScraper {
    fn source(&self) -> &str {
        "tmdb"
    }

    fn imagecut(&self) -> i32 {
        0 // Don't crop TMDB covers
    }

    async fn query_number_url(&self, number: &str) -> Result<String> {
        // number can be either movie ID or name
        // For now, assume it's an ID
        // TODO: Implement search by name
        Ok(format!(
            "https://www.themoviedb.org/movie/{}?language=zh-CN",
            number
        ))
    }

    fn parse_metadata(&self, html: &Html, url: &str) -> Result<MovieMetadata> {
        let mut metadata = MovieMetadata::default();

        // Extract metadata using CSS selectors (converted from XPath)

        // Title: //head/meta[@property="og:title"]/@content
        metadata.title = self.select_attr(html, r#"meta[property="og:title"]"#, "content");

        // Release: //div/span[@class="release"]/text()
        metadata.release = self.select_text(html, "div span.release");

        // Cover: //head/meta[@property="og:image"]/@content
        let cover_path = self.select_attr(html, r#"meta[property="og:image"]"#, "content");
        if !cover_path.is_empty() {
            // Prepend TMDB base URL if needed
            if cover_path.starts_with("http") {
                metadata.cover = cover_path;
            } else {
                metadata.cover = format!("https://www.themoviedb.org{}", cover_path);
            }
        }

        // Outline: //head/meta[@property="og:description"]/@content
        metadata.outline = self.select_attr(html, r#"meta[property="og:description"]"#, "content");

        // Extract movie ID from URL
        if let Some(id) = url.split('/').nth_back(0) {
            if let Some(clean_id) = id.split('?').next() {
                metadata.number = clean_id.to_string();
            }
        }

        // Additional fields from the page (if available)

        // Runtime (e.g., "120 min")
        metadata.runtime = self.select_text(html, "span.runtime");

        // Director
        metadata.director = self.select_text(html, "li.profile a[href*='/person/']");

        // Actors
        metadata.actor = self.select_all_text(html, "ol.people li p a");

        // Genres/Tags
        metadata.tag = self.select_all_text(html, "span.genres a");

        // Rating
        let rating_str = self.select_attr(html, r#"div.user_score_chart"#, "data-percent");
        if !rating_str.is_empty() {
            if let Ok(percent) = rating_str.parse::<f32>() {
                // Convert percentage (0-100) to rating (0-10)
                metadata.userrating = percent / 10.0;
            }
        }

        Ok(metadata)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_tmdb_scraper_url() {
        let scraper = TmdbScraper::new();
        let url = scraper.query_number_url("123456").await.unwrap();
        assert!(url.contains("themoviedb.org"));
        assert!(url.contains("123456"));
    }

    #[test]
    fn test_tmdb_scraper_parse() {
        let scraper = TmdbScraper::new();

        let html_content = r#"
        <html>
        <head>
            <meta property="og:title" content="Test Movie (2024)" />
            <meta property="og:description" content="A test movie description" />
            <meta property="og:image" content="/path/to/poster.jpg" />
        </head>
        <body>
            <div><span class="release">2024-03-15</span></div>
            <span class="runtime">120 min</span>
            <span class="genres"><a>Action</a><a>Drama</a></span>
        </body>
        </html>
        "#;

        let html = Html::parse_document(html_content);
        let metadata = scraper
            .parse_metadata(&html, "https://www.themoviedb.org/movie/123456")
            .unwrap();

        assert_eq!(metadata.title, "Test Movie (2024)");
        assert_eq!(metadata.outline, "A test movie description");
        assert!(metadata.cover.contains("poster.jpg"));
        assert_eq!(metadata.release, "2024-03-15");
        assert_eq!(metadata.runtime, "120 min");
        assert_eq!(metadata.tag, vec!["Action", "Drama"]);
        assert_eq!(metadata.source, ""); // Not set by parse_metadata, set by scrape()
    }
}
