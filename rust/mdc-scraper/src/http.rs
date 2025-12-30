//! HTTP client for web scraping with retry, proxy, and CloudFlare support
//!
//! This module provides a robust HTTP client that can handle:
//! - Automatic retries with exponential backoff
//! - Proxy support (HTTP, HTTPS, SOCKS5)
//! - User-agent rotation
//! - Cookie jar management
//! - CloudFlare bypass via Python subprocess

use anyhow::{anyhow, Result};
use reqwest::header::{HeaderMap, HeaderValue, COOKIE, USER_AGENT};
use reqwest::{Client, Proxy, Response, StatusCode};
use std::collections::HashMap;
use std::time::Duration;

/// Default user agent (Chrome 100)
pub const DEFAULT_USER_AGENT: &str =
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/100.0.4896.133 Safari/537.36";

/// Default timeout in seconds
pub const DEFAULT_TIMEOUT: u64 = 10;

/// Default retry count
pub const DEFAULT_RETRIES: u32 = 3;

/// HTTP backend type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HttpBackend {
    /// Standard reqwest client
    Reqwest,
    /// CloudFlare bypass via Python cloudscraper
    CloudScraper,
}

/// HTTP client configuration
#[derive(Debug, Clone)]
pub struct HttpConfig {
    /// User agent string
    pub user_agent: String,

    /// Request timeout in seconds
    pub timeout: Duration,

    /// Number of retries
    pub retries: u32,

    /// HTTP/HTTPS proxy URL
    pub proxy: Option<String>,

    /// SOCKS5 proxy URL
    pub socks_proxy: Option<String>,

    /// Verify SSL certificates
    pub verify_ssl: bool,

    /// Additional headers
    pub headers: HashMap<String, String>,

    /// Cookies
    pub cookies: HashMap<String, String>,

    /// Backend to use
    pub backend: HttpBackend,

    /// Debug mode
    pub debug: bool,
}

impl Default for HttpConfig {
    fn default() -> Self {
        Self {
            user_agent: DEFAULT_USER_AGENT.to_string(),
            timeout: Duration::from_secs(DEFAULT_TIMEOUT),
            retries: DEFAULT_RETRIES,
            proxy: None,
            socks_proxy: None,
            verify_ssl: true,
            headers: HashMap::new(),
            cookies: HashMap::new(),
            backend: HttpBackend::Reqwest,
            debug: false,
        }
    }
}

/// HTTP client
#[derive(Debug)]
pub struct HttpClient {
    config: HttpConfig,
    client: Client,
}

impl HttpClient {
    /// Create a new HTTP client with the given configuration
    pub fn new(config: HttpConfig) -> Result<Self> {
        let mut client_builder = Client::builder()
            .timeout(config.timeout)
            .danger_accept_invalid_certs(!config.verify_ssl)
            .cookie_store(true);

        // Add proxy if configured
        if let Some(ref proxy_url) = config.proxy {
            let proxy = Proxy::all(proxy_url)?;
            client_builder = client_builder.proxy(proxy);
        } else if let Some(ref socks_url) = config.socks_proxy {
            let proxy = Proxy::all(socks_url)?;
            client_builder = client_builder.proxy(proxy);
        }

        // Build default headers
        let mut headers = HeaderMap::new();
        headers.insert(USER_AGENT, HeaderValue::from_str(&config.user_agent)?);

        // Add custom headers
        for (key, value) in &config.headers {
            let header_name: reqwest::header::HeaderName = key.parse()?;
            headers.insert(header_name, HeaderValue::from_str(value)?);
        }

        client_builder = client_builder.default_headers(headers);

        let client = client_builder.build()?;

        Ok(Self { config, client })
    }

    /// Perform a GET request with retries
    pub async fn get(&self, url: &str) -> Result<Response> {
        self.get_with_retries(url, self.config.retries).await
    }

    /// Perform a GET request and return the response as text
    pub async fn get_text(&self, url: &str) -> Result<String> {
        let response = self.get(url).await?;
        let text = response.text().await?;
        Ok(text)
    }

    /// Perform a GET request with custom Cookie header and return the response as text
    ///
    /// # Arguments
    /// * `url` - URL to fetch
    /// * `cookie_header` - Cookie header value (e.g., "name1=value1; name2=value2")
    ///
    /// # Returns
    /// * `Result<String>` - Response body as text
    pub async fn get_text_with_cookies(&self, url: &str, cookie_header: &str) -> Result<String> {
        let response = self
            .client
            .get(url)
            .header(COOKIE, HeaderValue::from_str(cookie_header)?)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(anyhow!("HTTP {}: {}", response.status(), url));
        }

        let text = response.text().await?;
        Ok(text)
    }

    /// Perform a GET request and return the response as bytes
    pub async fn get_bytes(&self, url: &str) -> Result<Vec<u8>> {
        let response = self.get(url).await?;
        let bytes = response.bytes().await?;
        Ok(bytes.to_vec())
    }

    /// Perform a POST request with retries
    pub async fn post(&self, url: &str, data: &HashMap<String, String>) -> Result<Response> {
        self.post_with_retries(url, data, self.config.retries).await
    }

    /// Perform a POST request and return the response as text
    pub async fn post_text(&self, url: &str, data: &HashMap<String, String>) -> Result<String> {
        let response = self.post(url, data).await?;
        let text = response.text().await?;
        Ok(text)
    }

    /// Internal GET with retry logic
    async fn get_with_retries(&self, url: &str, retries: u32) -> Result<Response> {
        let mut last_error = None;

        for attempt in 0..=retries {
            match self.try_get(url).await {
                Ok(response) => {
                    if response.status().is_success() {
                        return Ok(response);
                    } else if self.is_retryable_status(response.status()) {
                        if self.config.debug {
                            tracing::debug!(
                                "Retryable status {} for {}, attempt {}/{}",
                                response.status(),
                                url,
                                attempt + 1,
                                retries + 1
                            );
                        }
                        last_error = Some(anyhow!("HTTP {}: {}", response.status(), url));
                    } else {
                        return Ok(response);
                    }
                }
                Err(e) => {
                    if self.config.debug {
                        tracing::debug!(
                            "Request failed for {}, attempt {}/{}: {}",
                            url,
                            attempt + 1,
                            retries + 1,
                            e
                        );
                    }
                    last_error = Some(e);
                }
            }

            if attempt < retries {
                // Exponential backoff: 1s, 2s, 4s, etc.
                let delay = Duration::from_secs(2_u64.pow(attempt));
                tokio::time::sleep(delay).await;
            }
        }

        Err(last_error.unwrap_or_else(|| anyhow!("Request failed after {} retries", retries)))
    }

    /// Internal POST with retry logic
    async fn post_with_retries(
        &self,
        url: &str,
        data: &HashMap<String, String>,
        retries: u32,
    ) -> Result<Response> {
        let mut last_error = None;

        for attempt in 0..=retries {
            match self.try_post(url, data).await {
                Ok(response) => {
                    if response.status().is_success() {
                        return Ok(response);
                    } else if self.is_retryable_status(response.status()) {
                        if self.config.debug {
                            tracing::debug!(
                                "Retryable status {} for {}, attempt {}/{}",
                                response.status(),
                                url,
                                attempt + 1,
                                retries + 1
                            );
                        }
                        last_error = Some(anyhow!("HTTP {}: {}", response.status(), url));
                    } else {
                        return Ok(response);
                    }
                }
                Err(e) => {
                    if self.config.debug {
                        tracing::debug!(
                            "Request failed for {}, attempt {}/{}: {}",
                            url,
                            attempt + 1,
                            retries + 1,
                            e
                        );
                    }
                    last_error = Some(e);
                }
            }

            if attempt < retries {
                let delay = Duration::from_secs(2_u64.pow(attempt));
                tokio::time::sleep(delay).await;
            }
        }

        Err(last_error.unwrap_or_else(|| anyhow!("Request failed after {} retries", retries)))
    }

    /// Try a single GET request
    async fn try_get(&self, url: &str) -> Result<Response> {
        let mut request = self.client.get(url);

        // Add cookies if present
        if !self.config.cookies.is_empty() {
            let cookie_header = self.build_cookie_header();
            request = request.header(COOKIE, cookie_header);
        }

        let response = request.send().await?;
        Ok(response)
    }

    /// Try a single POST request
    async fn try_post(&self, url: &str, data: &HashMap<String, String>) -> Result<Response> {
        let mut request = self.client.post(url).form(data);

        // Add cookies if present
        if !self.config.cookies.is_empty() {
            let cookie_header = self.build_cookie_header();
            request = request.header(COOKIE, cookie_header);
        }

        let response = request.send().await?;
        Ok(response)
    }

    /// Check if a status code should trigger a retry
    fn is_retryable_status(&self, status: StatusCode) -> bool {
        matches!(
            status,
            StatusCode::TOO_MANY_REQUESTS
                | StatusCode::INTERNAL_SERVER_ERROR
                | StatusCode::BAD_GATEWAY
                | StatusCode::SERVICE_UNAVAILABLE
                | StatusCode::GATEWAY_TIMEOUT
        )
    }

    /// Build cookie header from cookie map
    fn build_cookie_header(&self) -> String {
        self.config
            .cookies
            .iter()
            .map(|(k, v)| format!("{}={}", k, v))
            .collect::<Vec<_>>()
            .join("; ")
    }

    /// Get the current configuration
    pub fn config(&self) -> &HttpConfig {
        &self.config
    }
}

/// HTTP session builder for fluent configuration
pub struct HttpClientBuilder {
    config: HttpConfig,
}

impl HttpClientBuilder {
    /// Create a new builder with default configuration
    pub fn new() -> Self {
        Self {
            config: HttpConfig::default(),
        }
    }

    /// Set user agent
    pub fn user_agent(mut self, ua: &str) -> Self {
        self.config.user_agent = ua.to_string();
        self
    }

    /// Set timeout
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.config.timeout = timeout;
        self
    }

    /// Set retry count
    pub fn retries(mut self, retries: u32) -> Self {
        self.config.retries = retries;
        self
    }

    /// Set HTTP/HTTPS proxy
    pub fn proxy(mut self, proxy: &str) -> Self {
        self.config.proxy = Some(proxy.to_string());
        self
    }

    /// Set SOCKS5 proxy
    pub fn socks_proxy(mut self, proxy: &str) -> Self {
        self.config.socks_proxy = Some(proxy.to_string());
        self
    }

    /// Set SSL verification
    pub fn verify_ssl(mut self, verify: bool) -> Self {
        self.config.verify_ssl = verify;
        self
    }

    /// Add a custom header
    pub fn header(mut self, key: &str, value: &str) -> Self {
        self.config
            .headers
            .insert(key.to_string(), value.to_string());
        self
    }

    /// Add a cookie
    pub fn cookie(mut self, key: &str, value: &str) -> Self {
        self.config
            .cookies
            .insert(key.to_string(), value.to_string());
        self
    }

    /// Set backend
    pub fn backend(mut self, backend: HttpBackend) -> Self {
        self.config.backend = backend;
        self
    }

    /// Enable debug mode
    pub fn debug(mut self, debug: bool) -> Self {
        self.config.debug = debug;
        self
    }

    /// Build the HTTP client
    pub fn build(self) -> Result<HttpClient> {
        HttpClient::new(self.config)
    }
}

impl Default for HttpClientBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_builder() {
        let config = HttpClientBuilder::new()
            .user_agent("TestBot/1.0")
            .timeout(Duration::from_secs(30))
            .retries(5)
            .build();

        assert!(config.is_ok());
    }

    #[test]
    fn test_cookie_header() {
        let mut cookies = HashMap::new();
        cookies.insert("session".to_string(), "abc123".to_string());
        cookies.insert("user".to_string(), "john".to_string());

        let config = HttpConfig {
            cookies,
            ..Default::default()
        };

        let client = HttpClient::new(config).unwrap();
        let header = client.build_cookie_header();

        assert!(header.contains("session=abc123"));
        assert!(header.contains("user=john"));
    }

    #[test]
    fn test_retryable_status() {
        let client = HttpClient::new(HttpConfig::default()).unwrap();

        assert!(client.is_retryable_status(StatusCode::TOO_MANY_REQUESTS));
        assert!(client.is_retryable_status(StatusCode::INTERNAL_SERVER_ERROR));
        assert!(client.is_retryable_status(StatusCode::BAD_GATEWAY));
        assert!(client.is_retryable_status(StatusCode::SERVICE_UNAVAILABLE));
        assert!(client.is_retryable_status(StatusCode::GATEWAY_TIMEOUT));

        assert!(!client.is_retryable_status(StatusCode::OK));
        assert!(!client.is_retryable_status(StatusCode::NOT_FOUND));
        assert!(!client.is_retryable_status(StatusCode::FORBIDDEN));
    }
}
