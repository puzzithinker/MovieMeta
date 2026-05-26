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
        let title = self.select_attr(html, r#"meta[property="og:title"]"#, "content");
        let release = self.select_text(html, "div span.release");
        let cover_path = self.select_attr(html, r#"meta[property="og:image"]"#, "content");
        let cover = if !cover_path.is_empty() {
            if cover_path.starts_with("http") {
                cover_path
            } else {
                format!("https://www.themoviedb.org{}", cover_path)
            }
        } else {
            String::new()
        };
        let outline = self.select_attr(html, r#"meta[property="og:description"]"#, "content");

        let number = url
            .split('/')
            .nth_back(0)
            .and_then(|id| id.split('?').next())
            .map(|s| s.to_string())
            .unwrap_or_default();

        let runtime = self.select_text(html, "span.runtime");
        let director = self.select_text(html, "li.profile a[href*='/person/']");
        let actor = self.select_all_text(html, "ol.people li p a");
        let tag = self.select_all_text(html, "span.genres a");

        let rating_str = self.select_attr(html, r#"div.user_score_chart"#, "data-percent");
        let userrating = if !rating_str.is_empty() {
            rating_str
                .parse::<f32>()
                .ok()
                .map(|p| p / 10.0)
                .unwrap_or(0.0)
        } else {
            0.0
        };

        let metadata = MovieMetadata {
            title,
            release,
            cover,
            outline,
            number,
            runtime,
            director,
            actor,
            tag,
            userrating,
            ..Default::default()
        };

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
