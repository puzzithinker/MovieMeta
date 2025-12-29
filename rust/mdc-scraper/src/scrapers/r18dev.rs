//! R18Dev scraper
//!
//! R18Dev (r18.dev) is a modern JAV metadata API that provides excellent English translations
//! and comprehensive metadata via JSON endpoints. It's a wrapper around R18.com data.

use anyhow::{anyhow, Result};
use async_trait::async_trait;
use scraper::Html;
use serde_json::Value;

use crate::metadata::MovieMetadata;
use crate::scraper::{IdFormat, Scraper, ScraperConfig};

/// R18Dev scraper implementation
pub struct R18DevScraper {
    /// Base URL
    pub base_url: String,
}

impl R18DevScraper {
    /// Create a new R18Dev scraper with default URL
    pub fn new() -> Self {
        Self {
            base_url: "https://r18.dev/videos/vod/movies/detail/-".to_string(),
        }
    }

    /// Create scraper with custom base URL
    pub fn with_base_url(base_url: String) -> Self {
        Self { base_url }
    }

    /// Try to fetch metadata using dvd_id endpoint
    async fn try_dvd_id_endpoint(&self, number: &str, config: &ScraperConfig) -> Result<Value> {
        let url = format!("{}/dvd_id={}/json", self.base_url, number);

        if config.debug {
            tracing::debug!("R18Dev dvd_id URL: {}", url);
        }

        let response = config.client.get(&url).await?;
        let json: Value = serde_json::from_str(&response)?;

        Ok(json)
    }

    /// Try to fetch metadata using combined (content_id) endpoint
    async fn try_combined_endpoint(&self, content_id: &str, config: &ScraperConfig) -> Result<Value> {
        let url = format!("{}/combined={}/json", self.base_url, content_id);

        if config.debug {
            tracing::debug!("R18Dev combined URL: {}", url);
        }

        let response = config.client.get(&url).await?;
        let json: Value = serde_json::from_str(&response)?;

        Ok(json)
    }

    /// Parse JSON response to MovieMetadata
    fn parse_json(&self, json: &Value) -> Result<MovieMetadata> {
        let mut metadata = MovieMetadata::default();

        // Extract main movie object
        let movie = json.get("movie").ok_or_else(|| anyhow!("No movie object in JSON"))?;

        // DVD ID (product_id)
        if let Some(dvd_id) = movie.get("dvd_id").and_then(|v| v.as_str()) {
            metadata.number = dvd_id.to_uppercase();
        } else if let Some(content_id) = movie.get("content_id").and_then(|v| v.as_str()) {
            // Fallback: convert content_id to display format
            metadata.number = content_id.to_uppercase();
            // Try to insert hyphen if needed (ssis00123 → SSIS-123)
            if let Some(idx) = metadata.number.find(char::is_numeric) {
                if idx > 0 {
                    let (prefix, suffix) = metadata.number.split_at(idx);
                    metadata.number = format!("{}-{}", prefix, suffix.trim_start_matches('0'));
                }
            }
        }

        // Title (English)
        if let Some(title) = movie.get("title").and_then(|v| v.as_str()) {
            metadata.title = title.to_string();
        }

        // Cover image
        if let Some(images) = movie.get("images") {
            // Try to get the largest cover image
            if let Some(jacket_full) = images.get("jacket_image").and_then(|v| v.get("large")).and_then(|v| v.as_str()) {
                metadata.cover = jacket_full.to_string();
            } else if let Some(jacket) = images.get("jacket_image").and_then(|v| v.get("medium")).and_then(|v| v.as_str()) {
                metadata.cover = jacket.to_string();
            }
        }

        // Release date
        if let Some(date) = movie.get("release_date").and_then(|v| v.as_str()) {
            metadata.release = date.to_string();
        }

        // Runtime (in minutes)
        if let Some(runtime) = movie.get("runtime_minutes").and_then(|v| v.as_i64()) {
            metadata.runtime = format!("{}分", runtime);
        }

        // Director
        if let Some(directors) = movie.get("directors").and_then(|v| v.as_array()) {
            if let Some(director) = directors.first() {
                if let Some(name) = director.get("name").and_then(|v| v.as_str()) {
                    metadata.director = name.to_string();
                }
            }
        }

        // Studio (maker)
        if let Some(maker) = movie.get("maker") {
            if let Some(name) = maker.get("name").and_then(|v| v.as_str()) {
                metadata.studio = name.to_string();
            }
        }

        // Label
        if let Some(label) = movie.get("label") {
            if let Some(name) = label.get("name").and_then(|v| v.as_str()) {
                metadata.label = name.to_string();
            }
        }

        // Series
        if let Some(series) = movie.get("series") {
            if let Some(name) = series.get("name").and_then(|v| v.as_str()) {
                metadata.series = name.to_string();
            }
        }

        // Actors/Actresses
        if let Some(actresses) = movie.get("actresses").and_then(|v| v.as_array()) {
            for actress in actresses {
                if let Some(name) = actress.get("name").and_then(|v| v.as_str()) {
                    metadata.actor.push(name.to_string());
                }
            }
        }

        // Genres/Tags
        if let Some(categories) = movie.get("categories").and_then(|v| v.as_array()) {
            for category in categories {
                if let Some(name) = category.get("name").and_then(|v| v.as_str()) {
                    metadata.tag.push(name.to_string());
                }
            }
        }

        // Description/Outline
        if let Some(description) = movie.get("description").and_then(|v| v.as_str()) {
            metadata.outline = description.to_string();
        }

        // Rating (if available)
        if let Some(rating) = movie.get("rating").and_then(|v| v.get("average")).and_then(|v| v.as_f64()) {
            metadata.userrating = rating as f32;
        }

        // Validate essential fields
        if metadata.title.is_empty() {
            return Err(anyhow!("Failed to extract title from R18Dev JSON"));
        }

        Ok(metadata)
    }
}

impl Default for R18DevScraper {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Scraper for R18DevScraper {
    fn source(&self) -> &str {
        "r18dev"
    }

    fn imagecut(&self) -> i32 {
        1 // Smart crop with face detection
    }

    fn preferred_id_format(&self) -> IdFormat {
        // R18Dev works best with display format for dvd_id endpoint
        // But can also use content format for combined endpoint
        IdFormat::Display
    }

    async fn query_number_url(&self, number: &str) -> Result<String> {
        // Return the dvd_id endpoint URL
        Ok(format!("{}/dvd_id={}/json", self.base_url, number))
    }

    async fn scrape(&self, number: &str, config: &ScraperConfig) -> Result<MovieMetadata> {
        // Try dvd_id endpoint first (display format)
        let json = match self.try_dvd_id_endpoint(number, config).await {
            Ok(j) => j,
            Err(e1) => {
                if config.debug {
                    tracing::debug!("dvd_id endpoint failed: {}, trying combined...", e1);
                }

                // Fallback: Try combined endpoint with content_id format
                // Convert display to content if needed (SSIS-123 → ssis00123)
                let content_id = number.to_lowercase().replace("-", "");

                match self.try_combined_endpoint(&content_id, config).await {
                    Ok(j) => j,
                    Err(e2) => {
                        if config.debug {
                            tracing::debug!("combined endpoint also failed: {}", e2);
                        }
                        return Err(anyhow!("R18Dev: Both endpoints failed for {}", number));
                    }
                }
            }
        };

        // Parse JSON to metadata
        let mut metadata = self.parse_json(&json)?;

        // Post-processing
        metadata.source = self.source().to_string();
        metadata.website = format!("{}/dvd_id={}/json", self.base_url, number);
        metadata.imagecut = self.imagecut();
        metadata.extract_year();
        metadata.normalize_runtime();
        metadata.detect_uncensored();

        Ok(metadata)
    }

    fn parse_metadata(&self, _html: &Html, _url: &str) -> Result<MovieMetadata> {
        // R18Dev uses JSON, not HTML
        Err(anyhow!("R18Dev uses JSON API, not HTML parsing"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[tokio::test]
    async fn test_r18dev_scraper_url() {
        let scraper = R18DevScraper::new();
        let url = scraper.query_number_url("SSIS-123").await.unwrap();
        assert!(url.contains("r18.dev"));
        assert!(url.contains("dvd_id=SSIS-123"));
        assert!(url.contains("/json"));
    }

    #[test]
    fn test_r18dev_scraper_parse_json() {
        let scraper = R18DevScraper::new();

        let json_data = json!({
            "movie": {
                "dvd_id": "SSIS-123",
                "content_id": "ssis00123",
                "title": "Test English Title",
                "description": "This is a test description in English.",
                "release_date": "2024-01-15",
                "runtime_minutes": 120,
                "images": {
                    "jacket_image": {
                        "large": "https://pics.r18.com/digital/video/ssis00123/ssis00123pl.jpg",
                        "medium": "https://pics.r18.com/digital/video/ssis00123/ssis00123pm.jpg"
                    }
                },
                "maker": {
                    "id": "12",
                    "name": "S1 NO.1 STYLE"
                },
                "label": {
                    "id": "1",
                    "name": "S1 NO.1 STYLE"
                },
                "series": {
                    "id": "10",
                    "name": "Test Series"
                },
                "directors": [
                    {
                        "id": "5",
                        "name": "Test Director"
                    }
                ],
                "actresses": [
                    {
                        "id": "100",
                        "name": "Test Actress One"
                    },
                    {
                        "id": "101",
                        "name": "Test Actress Two"
                    }
                ],
                "categories": [
                    {
                        "id": "1",
                        "name": "Drama"
                    },
                    {
                        "id": "2",
                        "name": "Featured Actress"
                    }
                ],
                "rating": {
                    "average": 4.5,
                    "count": 100
                }
            }
        });

        let metadata = scraper.parse_json(&json_data).unwrap();

        assert_eq!(metadata.number, "SSIS-123");
        assert_eq!(metadata.title, "Test English Title");
        assert_eq!(metadata.release, "2024-01-15");
        assert_eq!(metadata.runtime, "120分");
        assert_eq!(metadata.director, "Test Director");
        assert_eq!(metadata.studio, "S1 NO.1 STYLE");
        assert_eq!(metadata.label, "S1 NO.1 STYLE");
        assert_eq!(metadata.series, "Test Series");
        assert!(metadata.cover.contains("ssis00123pl.jpg"));
        assert_eq!(metadata.actor.len(), 2);
        assert!(metadata.actor.contains(&"Test Actress One".to_string()));
        assert_eq!(metadata.tag.len(), 2);
        assert!(metadata.tag.contains(&"Drama".to_string()));
        assert!(metadata.outline.contains("test description"));
        assert_eq!(metadata.userrating, 4.5);
    }

    #[test]
    fn test_r18dev_preferred_id_format() {
        let scraper = R18DevScraper::new();
        assert_eq!(scraper.preferred_id_format(), IdFormat::Display);
    }

    #[test]
    fn test_r18dev_parse_json_minimal() {
        let scraper = R18DevScraper::new();

        // Test with minimal JSON (only required fields)
        let json_data = json!({
            "movie": {
                "dvd_id": "TEST-001",
                "title": "Minimal Test Movie"
            }
        });

        let metadata = scraper.parse_json(&json_data).unwrap();
        assert_eq!(metadata.number, "TEST-001");
        assert_eq!(metadata.title, "Minimal Test Movie");
    }

    #[test]
    fn test_r18dev_parse_json_missing_movie() {
        let scraper = R18DevScraper::new();

        // Test with missing movie object
        let json_data = json!({
            "error": "not found"
        });

        let result = scraper.parse_json(&json_data);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("No movie object"));
    }

    #[test]
    fn test_r18dev_content_id_fallback() {
        let scraper = R18DevScraper::new();

        // Test fallback to content_id when dvd_id is missing
        let json_data = json!({
            "movie": {
                "content_id": "ssis00123",
                "title": "Test with Content ID Only"
            }
        });

        let metadata = scraper.parse_json(&json_data).unwrap();
        // Should convert ssis00123 → SSIS-123
        assert!(metadata.number.contains("SSIS"));
        assert_eq!(metadata.title, "Test with Content ID Only");
    }
}
