//! CloudFlare bypass using Python cloudscraper subprocess
//!
//! This module provides CloudFlare protection bypass by calling a Python script
//! that uses the cloudscraper library.

use anyhow::{anyhow, Result};
use std::collections::HashMap;
use std::process::{Command, Stdio};
use std::time::Duration;

/// CloudScraper client using Python subprocess
#[derive(Debug, Clone)]
pub struct CloudScraperClient {
    /// Path to Python executable
    python_path: String,

    /// Path to cloudflare_bridge.py
    bridge_script: String,

    /// User agent
    user_agent: String,

    /// Timeout in seconds
    timeout: u64,

    /// Proxy URL
    proxy: Option<String>,

    /// Verify SSL
    verify_ssl: bool,

    /// Debug mode
    debug: bool,
}

impl CloudScraperClient {
    /// Create a new CloudScraper client
    pub fn new() -> Self {
        Self {
            python_path: "python3".to_string(),
            bridge_script: Self::find_bridge_script(),
            user_agent: crate::http::DEFAULT_USER_AGENT.to_string(),
            timeout: crate::http::DEFAULT_TIMEOUT,
            proxy: None,
            verify_ssl: false,
            debug: false,
        }
    }

    /// Find the cloudflare_bridge.py script
    fn find_bridge_script() -> String {
        // Try several possible locations
        let candidates = vec![
            "./cloudflare_bridge.py",
            "./mdc-scraper/cloudflare_bridge.py",
            "../mdc-scraper/cloudflare_bridge.py",
            "/usr/local/share/mdc/cloudflare_bridge.py",
        ];

        for path in candidates {
            if std::path::Path::new(path).exists() {
                return path.to_string();
            }
        }

        // Default to relative path
        "./cloudflare_bridge.py".to_string()
    }

    /// Set custom Python path
    pub fn python_path(mut self, path: &str) -> Self {
        self.python_path = path.to_string();
        self
    }

    /// Set custom bridge script path
    pub fn bridge_script(mut self, path: &str) -> Self {
        self.bridge_script = path.to_string();
        self
    }

    /// Set user agent
    pub fn user_agent(mut self, ua: &str) -> Self {
        self.user_agent = ua.to_string();
        self
    }

    /// Set timeout
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout.as_secs();
        self
    }

    /// Set proxy
    pub fn proxy(mut self, proxy: &str) -> Self {
        self.proxy = Some(proxy.to_string());
        self
    }

    /// Set SSL verification
    pub fn verify_ssl(mut self, verify: bool) -> Self {
        self.verify_ssl = verify;
        self
    }

    /// Enable debug mode
    pub fn debug(mut self, debug: bool) -> Self {
        self.debug = debug;
        self
    }

    /// Perform a GET request
    pub async fn get(&self, url: &str) -> Result<String> {
        self.request("GET", url, &HashMap::new(), &HashMap::new(), &HashMap::new())
            .await
    }

    /// Perform a POST request
    pub async fn post(
        &self,
        url: &str,
        data: &HashMap<String, String>,
    ) -> Result<String> {
        self.request("POST", url, data, &HashMap::new(), &HashMap::new())
            .await
    }

    /// Perform a request with full options
    pub async fn request(
        &self,
        method: &str,
        url: &str,
        data: &HashMap<String, String>,
        cookies: &HashMap<String, String>,
        headers: &HashMap<String, String>,
    ) -> Result<String> {
        // Build command arguments
        let mut args = vec![
            self.bridge_script.clone(),
            url.to_string(),
            "--method".to_string(),
            method.to_string(),
            "--timeout".to_string(),
            self.timeout.to_string(),
            "--user-agent".to_string(),
            self.user_agent.clone(),
        ];

        // Add proxy
        if let Some(ref proxy) = self.proxy {
            args.push("--proxy".to_string());
            args.push(proxy.clone());
        }

        // Add SSL verification
        if self.verify_ssl {
            args.push("--verify-ssl".to_string());
        }

        // Add cookies
        for (key, value) in cookies {
            args.push("--cookie".to_string());
            args.push(format!("{}={}", key, value));
        }

        // Add headers
        for (key, value) in headers {
            args.push("--header".to_string());
            args.push(format!("{}:{}", key, value));
        }

        // Add POST data
        if method == "POST" {
            for (key, value) in data {
                args.push("--data".to_string());
                args.push(format!("{}={}", key, value));
            }
        }

        if self.debug {
            tracing::debug!("CloudScraper command: {} {}", self.python_path, args.join(" "));
        }

        // Execute Python script
        let output = Command::new(&self.python_path)
            .args(&args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()?;

        if output.status.success() {
            let text = String::from_utf8(output.stdout)?;
            Ok(text)
        } else {
            let error = String::from_utf8_lossy(&output.stderr);
            Err(anyhow!("CloudScraper failed: {}", error))
        }
    }

    /// Check if CloudScraper is available
    pub async fn is_available(&self) -> bool {
        // Check if Python is available
        let python_check = Command::new(&self.python_path)
            .arg("--version")
            .output();

        if python_check.is_err() {
            return false;
        }

        // Check if bridge script exists
        if !std::path::Path::new(&self.bridge_script).exists() {
            return false;
        }

        // Check if cloudscraper module is installed
        let module_check = Command::new(&self.python_path)
            .args(&["-c", "import cloudscraper"])
            .output();

        module_check.is_ok() && module_check.unwrap().status.success()
    }
}

impl Default for CloudScraperClient {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_cloudscraper_availability() {
        let client = CloudScraperClient::new();
        let available = client.is_available().await;

        // This test doesn't require cloudscraper to be installed
        // It just checks the availability check works
        println!("CloudScraper available: {}", available);
    }

    #[test]
    fn test_builder() {
        let client = CloudScraperClient::new()
            .python_path("python3")
            .user_agent("TestBot/1.0")
            .timeout(Duration::from_secs(30))
            .proxy("http://localhost:8080")
            .verify_ssl(true)
            .debug(true);

        assert_eq!(client.python_path, "python3");
        assert_eq!(client.user_agent, "TestBot/1.0");
        assert_eq!(client.timeout, 30);
        assert_eq!(client.proxy, Some("http://localhost:8080".to_string()));
        assert_eq!(client.verify_ssl, true);
        assert_eq!(client.debug, true);
    }
}
