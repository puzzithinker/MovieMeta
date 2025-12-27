//! Scraper registry and source management

use anyhow::Result;
use std::collections::HashMap;
use std::sync::Arc;

use crate::metadata::MovieMetadata;
use crate::scraper::{Scraper, ScraperConfig};

/// Registry of available scrapers
pub struct ScraperRegistry {
    scrapers: HashMap<String, Arc<dyn Scraper>>,
    default_sources: Vec<String>,
}

impl ScraperRegistry {
    /// Create a new empty registry
    pub fn new() -> Self {
        Self {
            scrapers: HashMap::new(),
            default_sources: Vec::new(),
        }
    }

    /// Register a scraper
    pub fn register(&mut self, scraper: Arc<dyn Scraper>) {
        let source = scraper.source().to_string();
        self.scrapers.insert(source.clone(), scraper);
        if !self.default_sources.contains(&source) {
            self.default_sources.push(source);
        }
    }

    /// Get a scraper by source name
    pub fn get(&self, source: &str) -> Option<Arc<dyn Scraper>> {
        self.scrapers.get(source).cloned()
    }

    /// Get list of available sources
    pub fn available_sources(&self) -> Vec<String> {
        self.scrapers.keys().cloned().collect()
    }

    /// Search for metadata across multiple sources
    ///
    /// # Arguments
    /// * `number` - Movie number/ID to search
    /// * `sources` - Optional list of sources (uses defaults if None)
    /// * `config` - Scraper configuration
    ///
    /// Returns the first successful metadata result, or None if all sources fail
    pub async fn search(
        &self,
        number: &str,
        sources: Option<Vec<String>>,
        config: &ScraperConfig,
    ) -> Result<Option<MovieMetadata>> {
        // Determine sources to try
        let sources_to_try = if let Some(ref url) = config.specified_url {
            // If specific URL provided, try to infer source or use all
            vec![self.infer_source_from_url(url).unwrap_or_else(|| "unknown".to_string())]
        } else if let Some(sources_list) = sources {
            self.validate_sources(sources_list)
        } else {
            self.default_sources.clone()
        };

        if sources_to_try.is_empty() {
            tracing::warn!("No valid sources to search");
            return Ok(None);
        }

        // Try each source
        for source in sources_to_try {
            if config.debug {
                tracing::debug!("Trying source: {}", source);
            }

            let scraper = match self.scrapers.get(&source) {
                Some(s) => s,
                None => {
                    if config.debug {
                        tracing::warn!("Source not found: {}", source);
                    }
                    continue;
                }
            };

            match scraper.scrape(number, config).await {
                Ok(metadata) => {
                    if metadata.is_valid() {
                        if config.debug {
                            tracing::info!(
                                "Found metadata for [{}] from source '{}'",
                                number,
                                source
                            );
                        }
                        return Ok(Some(metadata));
                    } else {
                        if config.debug {
                            tracing::warn!("Invalid metadata from source '{}'", source);
                        }
                    }
                }
                Err(e) => {
                    if config.debug {
                        tracing::warn!("Error scraping from '{}': {}", source, e);
                    }
                }
            }
        }

        // No valid metadata found
        if config.debug {
            tracing::warn!("No metadata found for [{}] in any source", number);
        }
        Ok(None)
    }

    /// Validate and filter sources list
    fn validate_sources(&self, sources: Vec<String>) -> Vec<String> {
        sources
            .into_iter()
            .filter(|s| {
                let exists = self.scrapers.contains_key(s);
                if !exists {
                    tracing::warn!("Source not available: {}", s);
                }
                exists
            })
            .collect()
    }

    /// Try to infer source from URL
    fn infer_source_from_url(&self, url: &str) -> Option<String> {
        let url_lower = url.to_lowercase();

        for (source, _) in &self.scrapers {
            if url_lower.contains(source) {
                return Some(source.clone());
            }
        }

        // Check for known domains
        if url_lower.contains("themoviedb.org") {
            return Some("tmdb".to_string());
        } else if url_lower.contains("imdb.com") {
            return Some("imdb".to_string());
        } else if url_lower.contains("javlibrary.com") {
            return Some("javlibrary".to_string());
        } else if url_lower.contains("javbus.com") || url_lower.contains("javsee.com") {
            return Some("javbus".to_string());
        } else if url_lower.contains("avmoo.com") || url_lower.contains("avso.pw") {
            return Some("avmoo".to_string());
        } else if url_lower.contains("fc2.com") || url_lower.contains("fc2club") {
            return Some("fc2".to_string());
        } else if url_lower.contains("tokyo-hot.com") {
            return Some("tokyohot".to_string());
        }

        None
    }

    /// Set default sources order
    pub fn set_default_sources(&mut self, sources: Vec<String>) {
        self.default_sources = self.validate_sources(sources);
    }
}

impl Default for ScraperRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scraper::Scraper;
    use async_trait::async_trait;
    use scraper::Html;

    struct MockScraper {
        source: String,
        should_succeed: bool,
    }

    #[async_trait]
    impl Scraper for MockScraper {
        fn source(&self) -> &str {
            &self.source
        }

        async fn query_number_url(&self, number: &str) -> Result<String> {
            Ok(format!("http://{}.com/{}", self.source, number))
        }

        fn parse_metadata(&self, _html: &Html, _url: &str) -> Result<MovieMetadata> {
            if self.should_succeed {
                let mut meta = MovieMetadata::default();
                meta.number = "TEST-001".to_string();
                meta.title = format!("Test from {}", self.source);
                meta.cover = "http://example.com/cover.jpg".to_string();
                Ok(meta)
            } else {
                Err(anyhow::anyhow!("Mock failure"))
            }
        }
    }

    #[test]
    fn test_registry_registration() {
        let mut registry = ScraperRegistry::new();

        let scraper1 = Arc::new(MockScraper {
            source: "test1".to_string(),
            should_succeed: true,
        });
        let scraper2 = Arc::new(MockScraper {
            source: "test2".to_string(),
            should_succeed: true,
        });

        registry.register(scraper1);
        registry.register(scraper2);

        assert_eq!(registry.available_sources().len(), 2);
        assert!(registry.get("test1").is_some());
        assert!(registry.get("test2").is_some());
        assert!(registry.get("nonexistent").is_none());
    }

    #[test]
    fn test_validate_sources() {
        let mut registry = ScraperRegistry::new();

        registry.register(Arc::new(MockScraper {
            source: "test1".to_string(),
            should_succeed: true,
        }));

        let sources = vec!["test1".to_string(), "nonexistent".to_string()];
        let validated = registry.validate_sources(sources);

        assert_eq!(validated.len(), 1);
        assert_eq!(validated[0], "test1");
    }

    #[test]
    fn test_infer_source_from_url() {
        let mut registry = ScraperRegistry::new();

        registry.register(Arc::new(MockScraper {
            source: "tmdb".to_string(),
            should_succeed: true,
        }));

        let source = registry.infer_source_from_url("https://www.themoviedb.org/movie/123");
        assert_eq!(source, Some("tmdb".to_string()));

        let source = registry.infer_source_from_url("https://www.imdb.com/title/tt123");
        assert_eq!(source, Some("imdb".to_string()));
    }
}
