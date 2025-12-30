//! Scraper registry and source management

use anyhow::Result;
use std::collections::HashMap;
use std::sync::Arc;

use crate::metadata::MovieMetadata;
use crate::scraper::{IdFormat, Scraper, ScraperConfig};

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
    /// * `number` - Movie number/ID to search (display or content format)
    /// * `sources` - Optional list of sources (uses defaults if None)
    /// * `config` - Scraper configuration
    ///
    /// Returns the first successful metadata result, or None if all sources fail
    ///
    /// # Note
    /// This is the simple API that uses a single ID. For dual ID support (display + content),
    /// use `search_with_ids()` instead.
    pub async fn search(
        &self,
        number: &str,
        sources: Option<Vec<String>>,
        config: &ScraperConfig,
    ) -> Result<Option<MovieMetadata>> {
        // Use the same ID for both display and content (backward compatibility)
        self.search_with_ids(number, number, sources, config).await
    }

    /// Search for metadata with dual ID support
    ///
    /// # Arguments
    /// * `display_id` - Human-readable ID (e.g., "SSIS-123")
    /// * `content_id` - API format ID (e.g., "ssis00123")
    /// * `sources` - Optional list of sources (uses defaults if None)
    /// * `config` - Scraper configuration
    ///
    /// Each scraper will receive the ID format it prefers (display or content).
    pub async fn search_with_ids(
        &self,
        display_id: &str,
        content_id: &str,
        sources: Option<Vec<String>>,
        config: &ScraperConfig,
    ) -> Result<Option<MovieMetadata>> {
        // Determine sources to try
        let sources_to_try = if let Some(ref url) = config.specified_url {
            // If specific URL provided, try to infer source or use all
            vec![self
                .infer_source_from_url(url)
                .unwrap_or_else(|| "unknown".to_string())]
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

            // Select the appropriate ID format based on scraper preference
            let scraper_id = match scraper.preferred_id_format() {
                IdFormat::Display => display_id,
                IdFormat::Content => content_id,
            };

            if config.debug {
                tracing::debug!(
                    "Using ID format {:?} for scraper '{}': {}",
                    scraper.preferred_id_format(),
                    source,
                    scraper_id
                );
            }

            match scraper.scrape(scraper_id, config).await {
                Ok(metadata) => {
                    if metadata.is_valid() {
                        if config.debug {
                            tracing::info!(
                                "Found metadata for [{}] from source '{}'",
                                scraper_id,
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
            tracing::warn!(
                "No metadata found for [{}]/[{}] in any source",
                display_id,
                content_id
            );
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
    use crate::ScraperClient;
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

    // ===== Phase 5: Dual ID Integration Tests =====

    /// Mock scraper that records which ID it received
    struct IdCapturingScraper {
        source: String,
        format: IdFormat,
        captured_id: Arc<std::sync::Mutex<Option<String>>>,
    }

    #[async_trait]
    impl Scraper for IdCapturingScraper {
        fn source(&self) -> &str {
            &self.source
        }

        fn preferred_id_format(&self) -> IdFormat {
            self.format
        }

        async fn query_number_url(&self, number: &str) -> Result<String> {
            // Capture the ID we received
            *self.captured_id.lock().unwrap() = Some(number.to_string());
            Ok(format!("http://example.com/{}", number))
        }

        fn parse_metadata(&self, _html: &Html, _url: &str) -> Result<MovieMetadata> {
            let mut meta = MovieMetadata::default();
            meta.number = "TEST-001".to_string();
            meta.title = "Test Movie".to_string();
            meta.cover = "http://example.com/cover.jpg".to_string();
            Ok(meta)
        }
    }

    #[tokio::test]
    async fn test_search_with_ids_display_format() {
        // Test that Display-preferring scrapers receive the display ID
        let captured_id = Arc::new(std::sync::Mutex::new(None));
        let scraper = Arc::new(IdCapturingScraper {
            source: "test_display".to_string(),
            format: IdFormat::Display,
            captured_id: captured_id.clone(),
        });

        let mut registry = ScraperRegistry::new();
        registry.register(scraper);

        let client = ScraperClient::new().unwrap();
        let config = ScraperConfig::new(client);

        // Search with dual IDs
        let _ = registry
            .search_with_ids("SSIS-123", "ssis00123", None, &config)
            .await;

        // Verify the scraper received the display ID
        let received = captured_id.lock().unwrap();
        assert_eq!(received.as_ref().unwrap(), "SSIS-123");
    }

    #[tokio::test]
    async fn test_search_with_ids_content_format() {
        // Test that Content-preferring scrapers receive the content ID
        let captured_id = Arc::new(std::sync::Mutex::new(None));
        let scraper = Arc::new(IdCapturingScraper {
            source: "test_content".to_string(),
            format: IdFormat::Content,
            captured_id: captured_id.clone(),
        });

        let mut registry = ScraperRegistry::new();
        registry.register(scraper);

        let client = ScraperClient::new().unwrap();
        let config = ScraperConfig::new(client);

        // Search with dual IDs
        let _ = registry
            .search_with_ids("SSIS-123", "ssis00123", None, &config)
            .await;

        // Verify the scraper received the content ID
        let received = captured_id.lock().unwrap();
        assert_eq!(received.as_ref().unwrap(), "ssis00123");
    }

    #[tokio::test]
    async fn test_search_backward_compatibility() {
        // Test that old search() method still works (uses same ID for both)
        let captured_id = Arc::new(std::sync::Mutex::new(None));
        let scraper = Arc::new(IdCapturingScraper {
            source: "test_compat".to_string(),
            format: IdFormat::Display,
            captured_id: captured_id.clone(),
        });

        let mut registry = ScraperRegistry::new();
        registry.register(scraper);

        let client = ScraperClient::new().unwrap();
        let config = ScraperConfig::new(client);

        // Use old search() method
        let _ = registry.search("SSIS-123", None, &config).await;

        // Verify the scraper received the ID
        let received = captured_id.lock().unwrap();
        assert_eq!(received.as_ref().unwrap(), "SSIS-123");
    }

    #[tokio::test]
    async fn test_multiple_scrapers_different_formats() {
        // Test that different scrapers receive their preferred formats
        // We test each scraper separately since registry returns on first success
        let captured_display = Arc::new(std::sync::Mutex::new(None));
        let captured_content = Arc::new(std::sync::Mutex::new(None));

        let display_scraper = Arc::new(IdCapturingScraper {
            source: "display_scraper".to_string(),
            format: IdFormat::Display,
            captured_id: captured_display.clone(),
        });

        let content_scraper = Arc::new(IdCapturingScraper {
            source: "content_scraper".to_string(),
            format: IdFormat::Content,
            captured_id: captured_content.clone(),
        });

        // Test display scraper
        {
            let mut registry = ScraperRegistry::new();
            registry.register(display_scraper.clone());

            let client = ScraperClient::new().unwrap();
            let config = ScraperConfig::new(client);

            let _ = registry
                .search_with_ids("SSIS-123", "ssis00123", None, &config)
                .await;

            let display_received = captured_display.lock().unwrap();
            assert_eq!(display_received.as_ref().unwrap(), "SSIS-123");
        }

        // Test content scraper
        {
            let mut registry = ScraperRegistry::new();
            registry.register(content_scraper.clone());

            let client = ScraperClient::new().unwrap();
            let config = ScraperConfig::new(client);

            let _ = registry
                .search_with_ids("SSIS-123", "ssis00123", None, &config)
                .await;

            let content_received = captured_content.lock().unwrap();
            assert_eq!(content_received.as_ref().unwrap(), "ssis00123");
        }
    }

    #[tokio::test]
    async fn test_search_with_ids_fc2_format() {
        // Test FC2 content ID format (fc2-ppv-1234567)
        let captured_id = Arc::new(std::sync::Mutex::new(None));
        let scraper = Arc::new(IdCapturingScraper {
            source: "test_fc2".to_string(),
            format: IdFormat::Content,
            captured_id: captured_id.clone(),
        });

        let mut registry = ScraperRegistry::new();
        registry.register(scraper);

        let client = ScraperClient::new().unwrap();
        let config = ScraperConfig::new(client);

        // Search with FC2 IDs
        let _ = registry
            .search_with_ids("FC2-PPV-1234567", "fc2-ppv-1234567", None, &config)
            .await;

        // Verify the scraper received the content ID (lowercase)
        let received = captured_id.lock().unwrap();
        assert_eq!(received.as_ref().unwrap(), "fc2-ppv-1234567");
    }
}
