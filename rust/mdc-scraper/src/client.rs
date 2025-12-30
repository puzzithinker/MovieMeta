//! Unified HTTP client with automatic backend switching
//!
//! This module provides a unified interface that can automatically switch between
//! standard reqwest and CloudScraper backends based on configuration or failure.

use anyhow::{anyhow, Result};
use std::collections::HashMap;
use std::sync::Arc;

use crate::cloudscraper::CloudScraperClient;
use crate::http::{HttpBackend, HttpClient, HttpClientBuilder, HttpConfig};

/// Unified scraper client that can switch between backends
#[derive(Debug, Clone)]
pub struct ScraperClient {
    /// Current backend
    backend: HttpBackend,

    /// Reqwest client (always available)
    reqwest_client: Option<Arc<HttpClient>>,

    /// CloudScraper client (optional, requires Python)
    cloudscraper_client: Option<Arc<CloudScraperClient>>,

    /// Auto-fallback enabled
    auto_fallback: bool,

    /// Debug mode
    debug: bool,
}

impl ScraperClient {
    /// Create a new scraper client with default backend (reqwest)
    pub fn new() -> Result<Self> {
        let reqwest_client = HttpClientBuilder::new().build()?;

        Ok(Self {
            backend: HttpBackend::Reqwest,
            reqwest_client: Some(Arc::new(reqwest_client)),
            cloudscraper_client: None,
            auto_fallback: true,
            debug: false,
        })
    }

    /// Create a new scraper client with custom configuration
    pub fn with_config(config: HttpConfig) -> Result<Self> {
        let reqwest_client = HttpClient::new(config.clone())?;

        let cloudscraper_client = if config.backend == HttpBackend::CloudScraper {
            let mut cs_client = CloudScraperClient::new()
                .user_agent(&config.user_agent)
                .timeout(config.timeout)
                .verify_ssl(config.verify_ssl)
                .debug(config.debug);

            if let Some(ref proxy) = config.proxy {
                cs_client = cs_client.proxy(proxy);
            }

            Some(Arc::new(cs_client))
        } else {
            None
        };

        Ok(Self {
            backend: config.backend,
            reqwest_client: Some(Arc::new(reqwest_client)),
            cloudscraper_client,
            auto_fallback: true,
            debug: config.debug,
        })
    }

    /// Set backend explicitly
    pub fn set_backend(&mut self, backend: HttpBackend) {
        self.backend = backend;

        // Initialize CloudScraper if switching to it
        if backend == HttpBackend::CloudScraper && self.cloudscraper_client.is_none() {
            self.cloudscraper_client = Some(Arc::new(CloudScraperClient::new()));
        }
    }

    /// Enable or disable automatic fallback
    pub fn set_auto_fallback(&mut self, enabled: bool) {
        self.auto_fallback = enabled;
    }

    /// Get the current backend
    pub fn backend(&self) -> HttpBackend {
        self.backend
    }

    /// Check if CloudScraper is available
    pub async fn is_cloudscraper_available(&self) -> bool {
        if let Some(ref client) = self.cloudscraper_client {
            client.is_available().await
        } else {
            let client = CloudScraperClient::new();
            client.is_available().await
        }
    }

    /// Perform a GET request
    pub async fn get(&self, url: &str) -> Result<String> {
        self.get_with_cookies(url, None).await
    }

    /// Perform a GET request with optional Cookie header
    ///
    /// # Arguments
    /// * `url` - URL to fetch
    /// * `cookie_header` - Optional Cookie header value (e.g., "name1=value1; name2=value2")
    ///
    /// # Returns
    /// * `Result<String>` - Response body as text
    pub async fn get_with_cookies(&self, url: &str, cookie_header: Option<&str>) -> Result<String> {
        match self.backend {
            HttpBackend::Reqwest => {
                let client = self
                    .reqwest_client
                    .as_ref()
                    .ok_or_else(|| anyhow!("Reqwest client not initialized"))?;

                // If cookie header is provided, use method with cookies
                let result = if let Some(cookies) = cookie_header {
                    client.get_text_with_cookies(url, cookies).await
                } else {
                    // Use standard get_text method
                    client.get_text(url).await
                };

                match result {
                    Ok(text) => Ok(text),
                    Err(e) => {
                        if self.auto_fallback && self.should_fallback(&e) {
                            if self.debug {
                                tracing::info!(
                                    "Reqwest failed, falling back to CloudScraper: {}",
                                    e
                                );
                            }
                            self.get_with_cloudscraper(url).await
                        } else {
                            Err(e)
                        }
                    }
                }
            }
            HttpBackend::CloudScraper => self.get_with_cloudscraper(url).await,
        }
    }

    /// Perform a POST request
    pub async fn post(&self, url: &str, data: &HashMap<String, String>) -> Result<String> {
        match self.backend {
            HttpBackend::Reqwest => {
                let client = self
                    .reqwest_client
                    .as_ref()
                    .ok_or_else(|| anyhow!("Reqwest client not initialized"))?;

                match client.post_text(url, data).await {
                    Ok(text) => Ok(text),
                    Err(e) => {
                        if self.auto_fallback && self.should_fallback(&e) {
                            if self.debug {
                                tracing::info!(
                                    "Reqwest failed, falling back to CloudScraper: {}",
                                    e
                                );
                            }
                            self.post_with_cloudscraper(url, data).await
                        } else {
                            Err(e)
                        }
                    }
                }
            }
            HttpBackend::CloudScraper => self.post_with_cloudscraper(url, data).await,
        }
    }

    /// GET with CloudScraper
    async fn get_with_cloudscraper(&self, url: &str) -> Result<String> {
        let client = if let Some(ref client) = self.cloudscraper_client {
            client
        } else {
            return Err(anyhow!("CloudScraper not initialized"));
        };

        client.get(url).await
    }

    /// POST with CloudScraper
    async fn post_with_cloudscraper(
        &self,
        url: &str,
        data: &HashMap<String, String>,
    ) -> Result<String> {
        let client = if let Some(ref client) = self.cloudscraper_client {
            client
        } else {
            return Err(anyhow!("CloudScraper not initialized"));
        };

        client.post(url, data).await
    }

    /// Check if error warrants fallback to CloudScraper
    fn should_fallback(&self, error: &anyhow::Error) -> bool {
        let error_str = error.to_string().to_lowercase();

        // Check for common CloudFlare indicators
        error_str.contains("cloudflare")
            || error_str.contains("403")
            || error_str.contains("cf-ray")
            || error_str.contains("just a moment")
    }
}

impl Default for ScraperClient {
    fn default() -> Self {
        Self::new().expect("Failed to create default ScraperClient")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_creation() {
        let client = ScraperClient::new();
        assert!(client.is_ok());

        let client = client.unwrap();
        assert_eq!(client.backend(), HttpBackend::Reqwest);
        assert!(client.reqwest_client.is_some());
    }

    #[test]
    fn test_backend_switching() {
        let mut client = ScraperClient::new().unwrap();
        assert_eq!(client.backend(), HttpBackend::Reqwest);

        client.set_backend(HttpBackend::CloudScraper);
        assert_eq!(client.backend(), HttpBackend::CloudScraper);
        assert!(client.cloudscraper_client.is_some());
    }

    #[test]
    fn test_should_fallback() {
        let client = ScraperClient::new().unwrap();

        let cf_error = anyhow!("HTTP 403: CloudFlare protection");
        assert!(client.should_fallback(&cf_error));

        let normal_error = anyhow!("HTTP 404: Not Found");
        assert!(!client.should_fallback(&normal_error));
    }

    #[tokio::test]
    async fn test_cloudscraper_availability() {
        let client = ScraperClient::new().unwrap();
        let available = client.is_cloudscraper_available().await;

        // Just check that the method works, don't require cloudscraper
        println!("CloudScraper available: {}", available);
    }
}
