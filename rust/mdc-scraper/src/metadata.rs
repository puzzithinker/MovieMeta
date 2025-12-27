//! Movie metadata structure returned by scrapers

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Movie metadata scraped from various sources
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MovieMetadata {
    /// Movie number/ID (e.g., "MOVIE-001", "tt1234567")
    pub number: String,

    /// Movie title
    pub title: String,

    /// Production studio
    #[serde(default)]
    pub studio: String,

    /// Release date (YYYY-MM-DD format)
    #[serde(default)]
    pub release: String,

    /// Release year
    #[serde(default)]
    pub year: String,

    /// Plot outline/description
    #[serde(default)]
    pub outline: String,

    /// Runtime in minutes
    #[serde(default)]
    pub runtime: String,

    /// Director name
    #[serde(default)]
    pub director: String,

    /// List of actor names
    #[serde(default)]
    pub actor: Vec<String>,

    /// Actor name to photo URL mapping
    #[serde(default)]
    pub actor_photo: HashMap<String, String>,

    /// Cover image URL (fanart)
    #[serde(default)]
    pub cover: String,

    /// Small cover image URL (poster)
    #[serde(default)]
    pub cover_small: String,

    /// Extra fanart URLs
    #[serde(default)]
    pub extrafanart: Vec<String>,

    /// Trailer URL
    #[serde(default)]
    pub trailer: String,

    /// Tags/genres
    #[serde(default)]
    pub tag: Vec<String>,

    /// Label/publisher
    #[serde(default)]
    pub label: String,

    /// Series name
    #[serde(default)]
    pub series: String,

    /// User rating (0.0 - 10.0)
    #[serde(default)]
    pub userrating: f32,

    /// Number of user votes
    #[serde(default)]
    pub uservotes: u32,

    /// Whether content is uncensored
    #[serde(default)]
    pub uncensored: bool,

    /// Source website URL
    pub website: String,

    /// Source scraper name
    pub source: String,

    /// Image cut mode:
    /// - 0: Copy cover without cropping
    /// - 1: Smart crop with face detection
    /// - 3: Download small cover
    #[serde(default = "default_imagecut")]
    pub imagecut: i32,
}

fn default_imagecut() -> i32 {
    1
}

impl Default for MovieMetadata {
    fn default() -> Self {
        Self {
            number: String::new(),
            title: String::new(),
            studio: String::new(),
            release: String::new(),
            year: String::new(),
            outline: String::new(),
            runtime: String::new(),
            director: String::new(),
            actor: Vec::new(),
            actor_photo: HashMap::new(),
            cover: String::new(),
            cover_small: String::new(),
            extrafanart: Vec::new(),
            trailer: String::new(),
            tag: Vec::new(),
            label: String::new(),
            series: String::new(),
            userrating: 0.0,
            uservotes: 0,
            uncensored: false,
            website: String::new(),
            source: String::new(),
            imagecut: 1,
        }
    }
}

impl MovieMetadata {
    /// Check if metadata is valid (has required fields)
    pub fn is_valid(&self) -> bool {
        !self.title.is_empty()
            && !self.number.is_empty()
            && (!self.cover.is_empty() || !self.cover_small.is_empty())
    }

    /// Extract year from release date
    pub fn extract_year(&mut self) {
        if self.year.is_empty() && !self.release.is_empty() {
            if let Some(year) = self.release.split('-').next() {
                if year.len() == 4 {
                    self.year = year.to_string();
                }
            }
        }
    }

    /// Normalize runtime (remove "min" suffix, extract numbers)
    pub fn normalize_runtime(&mut self) {
        if !self.runtime.is_empty() {
            self.runtime = self
                .runtime
                .trim()
                .trim_end_matches("min")
                .trim_end_matches("mi")
                .trim()
                .to_string();
        }
    }

    /// Detect if movie is uncensored based on tags and title
    pub fn detect_uncensored(&mut self) {
        if self.uncensored {
            return;
        }

        let uncensored_keywords = ["無码", "無修正", "uncensored", "无码"];

        // Check tags
        for tag in &self.tag {
            let tag_lower = tag.to_lowercase();
            for keyword in &uncensored_keywords {
                if tag_lower.contains(&keyword.to_lowercase()) {
                    self.uncensored = true;
                    return;
                }
            }
        }

        // Check title
        let title_lower = self.title.to_lowercase();
        for keyword in &uncensored_keywords {
            if title_lower.contains(&keyword.to_lowercase()) {
                self.uncensored = true;
                return;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metadata_validity() {
        let mut meta = MovieMetadata::default();
        assert!(!meta.is_valid());

        meta.number = "TEST-001".to_string();
        meta.title = "Test Movie".to_string();
        meta.cover = "http://example.com/cover.jpg".to_string();
        assert!(meta.is_valid());
    }

    #[test]
    fn test_extract_year() {
        let mut meta = MovieMetadata::default();
        meta.release = "2024-03-15".to_string();
        meta.extract_year();
        assert_eq!(meta.year, "2024");
    }

    #[test]
    fn test_normalize_runtime() {
        let mut meta = MovieMetadata::default();
        meta.runtime = "120min".to_string();
        meta.normalize_runtime();
        assert_eq!(meta.runtime, "120");

        meta.runtime = "90 mi".to_string();
        meta.normalize_runtime();
        assert_eq!(meta.runtime, "90");
    }

    #[test]
    fn test_detect_uncensored() {
        let mut meta = MovieMetadata::default();
        meta.tag = vec!["無码".to_string(), "HD".to_string()];
        meta.detect_uncensored();
        assert!(meta.uncensored);

        let mut meta2 = MovieMetadata::default();
        meta2.title = "Test Movie UNCENSORED".to_string();
        meta2.detect_uncensored();
        assert!(meta2.uncensored);
    }
}
