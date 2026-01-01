//! Main processing workflow orchestration

use anyhow::{Context, Result};
use std::path::{Path, PathBuf};

use crate::file_ops::{
    execute_file_operation, get_video_extension, move_subtitles, sanitize_filename,
};
use crate::nfo::{generate_nfo, write_nfo};
use crate::processor::{FileAttributes, LinkMode, ProcessingMode, Template};

/// Configuration for the processor
#[derive(Debug, Clone)]
pub struct ProcessorConfig {
    /// Processing mode
    pub mode: ProcessingMode,

    /// Link mode for file operations
    pub link_mode: LinkMode,

    /// Success output folder
    pub success_folder: PathBuf,

    /// Location rule template (e.g., "actor/number")
    pub location_rule: String,

    /// Naming rule template (e.g., "number")
    pub naming_rule: String,

    /// Maximum title length
    pub max_title_len: usize,

    /// Whether to skip if file already exists
    pub skip_existing: bool,

    /// Whether to download images
    pub download_images: bool,

    /// Whether to create NFO files
    pub create_nfo: bool,

    /// Whether to move subtitles
    pub move_subtitles: bool,
}

impl Default for ProcessorConfig {
    fn default() -> Self {
        Self {
            mode: ProcessingMode::Scraping,
            link_mode: LinkMode::Move,
            success_folder: PathBuf::from("./output"),
            location_rule: "actor/year/number-title".to_string(),
            naming_rule: "number-title".to_string(),
            max_title_len: 50,
            skip_existing: false,
            download_images: true,
            create_nfo: true,
            move_subtitles: true,
        }
    }
}

/// Download an image from a URL and save it to a file
async fn download_image(url: &str, dest_path: &Path, cookie_header: Option<&str>) -> Result<()> {
    // Create HTTP client with proper headers to bypass hotlink protection
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
        .build()?;

    // Extract domain for Referer header
    let referer = url::Url::parse(url)
        .ok()
        .and_then(|u| {
            u.host_str().map(|host| {
                format!("{}://{}/", u.scheme(), host)
            })
        })
        .unwrap_or_else(|| "https://www.javbus.com/".to_string());

    // Build request with proper headers
    let mut request = client
        .get(url)
        .header("Referer", &referer)
        .header("Accept", "image/avif,image/webp,image/apng,image/svg+xml,image/*,*/*;q=0.8");

    // Add cookies if provided
    if let Some(cookies) = cookie_header {
        request = request.header("Cookie", cookies);
    }

    // Download image
    let response = request
        .send()
        .await
        .with_context(|| format!("Failed to download image from {}", url))?;

    if !response.status().is_success() {
        return Err(anyhow::anyhow!(
            "HTTP error {}: {}",
            response.status(),
            url
        ));
    }

    let bytes = response.bytes().await
        .with_context(|| format!("Failed to read image bytes from {}", url))?;

    // Save to file
    std::fs::write(dest_path, &bytes)
        .with_context(|| format!("Failed to write image to {:?}", dest_path))?;

    Ok(())
}

/// Download images (poster only) for a movie
async fn download_movie_images(
    metadata: &serde_json::Value,
    dest_folder: &Path,
    base_name: &str,
) -> Result<()> {
    // Extract image URLs from metadata
    // MovieMetadata struct has: cover (main image), cover_small (poster variant)
    // We prioritize cover_small for poster, fallback to cover if not available
    let cover_url = metadata["cover"].as_str();
    let cover_small_url = metadata["cover_small"].as_str();

    tracing::debug!("Image URLs - cover: {:?}, cover_small: {:?}", cover_url, cover_small_url);

    // Determine which URL to use for poster (prefer cover_small, fallback to cover)
    let poster_url = cover_small_url
        .filter(|url| !url.is_empty())
        .or_else(|| cover_url.filter(|url| !url.is_empty()));

    if let Some(url) = poster_url {
        // Try to load cookies from config for authenticated downloads
        let cookie_header = load_cookies_for_domain(extract_domain(url));

        let poster_path = dest_folder.join(format!("{}-poster.jpg", base_name));
        tracing::info!("Downloading poster from: {}", url);

        if let Err(e) = download_image(url, &poster_path, cookie_header.as_deref()).await {
            tracing::warn!("Failed to download poster from {}: {}", url, e);
        } else {
            tracing::info!("Downloaded poster: {:?}", poster_path);
        }
    } else {
        tracing::debug!("No poster URL available for {}", base_name);
    }

    Ok(())
}

/// Extract domain from URL
fn extract_domain(url: &str) -> Option<String> {
    url::Url::parse(url)
        .ok()
        .and_then(|u| u.host_str().map(|h| h.to_string()))
}

/// Load cookies for a domain from config.ini
fn load_cookies_for_domain(domain: Option<String>) -> Option<String> {
    let domain = domain?;

    // Try to load config.ini
    use std::path::PathBuf;
    let config_paths = vec![
        PathBuf::from("./config.ini"),
        PathBuf::from("../config.ini"),
        dirs::home_dir()?.join(".mdc/config.ini"),
        dirs::home_dir()?.join(".config/mdc/config.ini"),
    ];

    for path in config_paths {
        if path.exists() {
            if let Ok(content) = std::fs::read_to_string(&path) {
                // Simple INI parsing for cookies section
                if let Some(cookies_section) = content.split("[cookies]").nth(1) {
                    for line in cookies_section.lines() {
                        let line = line.trim();
                        if line.is_empty() || line.starts_with('[') {
                            break;
                        }
                        if let Some((key, value)) = line.split_once('=') {
                            let key = key.trim();
                            if key == &domain || key == domain.trim_start_matches("www.") {
                                return Some(value.trim().to_string());
                            }
                        }
                    }
                }
            }
        }
    }

    None
}

/// Context for processing a single file
pub struct ProcessingContext {
    /// Original movie file path
    pub movie_path: PathBuf,

    /// Extracted movie number
    pub number: String,

    /// Scraped metadata (JSON)
    pub metadata: serde_json::Value,

    /// File attributes detected from path
    pub attributes: FileAttributes,

    /// Configuration
    pub config: ProcessorConfig,
}

impl ProcessingContext {
    /// Create a new processing context
    pub fn new(
        movie_path: PathBuf,
        number: String,
        metadata: serde_json::Value,
        config: ProcessorConfig,
    ) -> Self {
        let attributes = FileAttributes::from_path(&movie_path);

        Self {
            movie_path,
            number,
            metadata,
            attributes,
            config,
        }
    }

    /// Determine the destination folder based on location rule
    pub fn get_destination_folder(&self) -> Result<PathBuf> {
        let template = Template::new(&self.config.location_rule);
        let location = template.render(&self.metadata);

        // Split by path separators and sanitize each component
        let components: Vec<String> = location.split('/').map(|s| sanitize_filename(s)).collect();

        let mut dest_folder = self.config.success_folder.clone();
        for component in components {
            if !component.is_empty() {
                dest_folder = dest_folder.join(component);
            }
        }

        Ok(dest_folder)
    }

    /// Determine the base filename (without extension) based on naming rule
    pub fn get_destination_filename_base(&self) -> Result<String> {
        let template = Template::new(&self.config.naming_rule);
        let mut base_name = template.render(&self.metadata);

        // Add suffix for attributes
        let suffix = self.attributes.get_suffix();
        if !suffix.is_empty() {
            base_name.push_str(&suffix);
        }

        // Add multi-part suffix
        if self.attributes.multi_part && !self.attributes.part.is_empty() {
            base_name.push_str(&self.attributes.part);
        }

        // Sanitize filename
        let sanitized = sanitize_filename(&base_name);

        Ok(sanitized)
    }

    /// Get the full destination path for the movie file
    pub fn get_destination_path(&self) -> Result<PathBuf> {
        let folder = self.get_destination_folder()?;
        let base_name = self.get_destination_filename_base()?;
        let ext = get_video_extension(&self.movie_path);

        Ok(folder.join(format!("{}{}", base_name, ext)))
    }

    /// Execute the workflow based on processing mode
    pub fn execute(&self) -> Result<()> {
        match self.config.mode {
            ProcessingMode::Scraping => self.execute_scraping_mode(),
            ProcessingMode::Organizing => self.execute_organizing_mode(),
            ProcessingMode::Analysis => self.execute_analysis_mode(),
        }
    }

    /// Mode 1: Full scraping workflow
    fn execute_scraping_mode(&self) -> Result<()> {
        let dest_folder = self.get_destination_folder()?;
        let base_name = self.get_destination_filename_base()?;
        let dest_path = self.get_destination_path()?;

        // Create destination folder
        std::fs::create_dir_all(&dest_folder)
            .with_context(|| format!("Failed to create folder: {:?}", dest_folder))?;

        // Skip if already exists
        if self.config.skip_existing && dest_path.exists() {
            tracing::info!("Skipping existing file: {}", dest_path.display());
            return Ok(());
        }

        // Generate and write NFO
        if self.config.create_nfo {
            let nfo_path = dest_folder.join(format!("{}.nfo", base_name));
            let nfo_content = generate_nfo(&self.metadata, &self.number)?;
            write_nfo(&nfo_path, &nfo_content)?;
        }

        // Download images if enabled
        if self.config.download_images {
            // Use tokio::task::block_in_place to run async code from sync context
            tokio::task::block_in_place(|| {
                tokio::runtime::Handle::current().block_on(async {
                    download_movie_images(
                        &self.metadata,
                        &dest_folder,
                        &base_name,
                    ).await
                })
            })?;
        }

        // Move/link the movie file
        execute_file_operation(&self.movie_path, &dest_path, self.config.link_mode)?;

        // Move subtitles if enabled
        if self.config.move_subtitles {
            let _ = move_subtitles(
                &self.movie_path,
                &dest_folder,
                &base_name,
                self.config.link_mode,
            );
        }

        tracing::info!(
            "Processing complete: {} -> {}",
            self.movie_path.display(),
            dest_path.display()
        );
        Ok(())
    }

    /// Mode 2: Organizing only (no metadata scraping)
    fn execute_organizing_mode(&self) -> Result<()> {
        let dest_folder = self.get_destination_folder()?;
        let base_name = self.get_destination_filename_base()?;
        let dest_path = self.get_destination_path()?;

        // Create destination folder
        std::fs::create_dir_all(&dest_folder)
            .with_context(|| format!("Failed to create folder: {:?}", dest_folder))?;

        // Skip if already exists
        if self.config.skip_existing && dest_path.exists() {
            tracing::info!("Skipping existing file: {}", dest_path.display());
            return Ok(());
        }

        // Move/link the movie file
        execute_file_operation(&self.movie_path, &dest_path, self.config.link_mode)?;

        // Move subtitles if enabled
        if self.config.move_subtitles {
            let _ = move_subtitles(
                &self.movie_path,
                &dest_folder,
                &base_name,
                self.config.link_mode,
            );
        }

        tracing::info!(
            "Organizing complete: {} -> {}",
            self.movie_path.display(),
            dest_path.display()
        );
        Ok(())
    }

    /// Mode 3: Analysis mode (scrape in-place, no file moving)
    fn execute_analysis_mode(&self) -> Result<()> {
        let source_folder = self
            .movie_path
            .parent()
            .ok_or_else(|| anyhow::anyhow!("Cannot determine parent folder"))?;
        let base_name = self.get_destination_filename_base()?;

        // Generate and write NFO in the same folder
        if self.config.create_nfo {
            let nfo_path = source_folder.join(format!("{}.nfo", base_name));
            let nfo_content = generate_nfo(&self.metadata, &self.number)?;
            write_nfo(&nfo_path, &nfo_content)?;
        }

        // Download images to source folder if enabled
        if self.config.download_images {
            // Use tokio::task::block_in_place to run async code from sync context
            tokio::task::block_in_place(|| {
                tokio::runtime::Handle::current().block_on(async {
                    download_movie_images(
                        &self.metadata,
                        source_folder,
                        &base_name,
                    ).await
                })
            })?;
        }

        tracing::info!(
            "Analysis complete (in-place): {}",
            self.movie_path.display()
        );
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::fs;
    use tempfile::TempDir;

    fn create_test_metadata() -> serde_json::Value {
        json!({
            "number": "TEST-001",
            "title": "Test Movie",
            "studio": "Test Studio",
            "actor": ["Test Actor"],
            "year": "2024"
        })
    }

    #[test]
    fn test_get_destination_folder() {
        let temp = TempDir::new().unwrap();
        let movie_path = temp.path().join("movie.mp4");

        let config = ProcessorConfig {
            success_folder: temp.path().join("output"),
            location_rule: "studio/number".to_string(),
            ..Default::default()
        };

        let ctx = ProcessingContext::new(
            movie_path,
            "TEST-001".to_string(),
            create_test_metadata(),
            config,
        );

        let dest = ctx.get_destination_folder().unwrap();
        let dest_str = dest.to_string_lossy();
        assert!(dest_str.contains("Test Studio"));
        assert!(dest_str.contains("TEST-001"));
    }

    #[test]
    fn test_get_destination_filename_base() {
        let temp = TempDir::new().unwrap();
        let movie_path = temp.path().join("movie-C.mp4");

        let config = ProcessorConfig {
            naming_rule: "number".to_string(),
            ..Default::default()
        };

        let ctx = ProcessingContext::new(
            movie_path,
            "TEST-001".to_string(),
            create_test_metadata(),
            config,
        );

        let base_name = ctx.get_destination_filename_base().unwrap();
        assert_eq!(base_name, "TEST-001-C"); // Should include -C suffix
    }

    #[test]
    fn test_execute_organizing_mode() {
        let temp = TempDir::new().unwrap();
        let movie_path = temp.path().join("source/movie.mp4");

        // Create source file
        fs::create_dir_all(movie_path.parent().unwrap()).unwrap();
        fs::write(&movie_path, "test content").unwrap();

        let config = ProcessorConfig {
            mode: ProcessingMode::Organizing,
            success_folder: temp.path().join("output"),
            location_rule: "number".to_string(),
            naming_rule: "number".to_string(),
            link_mode: LinkMode::Move,
            create_nfo: false,
            download_images: false, // Disable to avoid async runtime in tests
            ..Default::default()
        };

        let ctx = ProcessingContext::new(
            movie_path.clone(),
            "TEST-001".to_string(),
            create_test_metadata(),
            config,
        );

        ctx.execute().unwrap();

        // Verify file was moved
        assert!(!movie_path.exists());
        let dest_path = temp.path().join("output/TEST-001/TEST-001.mp4");
        assert!(dest_path.exists());
    }

    #[test]
    fn test_execute_analysis_mode() {
        let temp = TempDir::new().unwrap();
        let movie_path = temp.path().join("movie.mp4");

        // Create source file
        fs::write(&movie_path, "test content").unwrap();

        let config = ProcessorConfig {
            mode: ProcessingMode::Analysis,
            naming_rule: "number".to_string(),
            create_nfo: true,
            download_images: false, // Disable to avoid async runtime in tests
            ..Default::default()
        };

        let ctx = ProcessingContext::new(
            movie_path.clone(),
            "TEST-001".to_string(),
            create_test_metadata(),
            config,
        );

        ctx.execute().unwrap();

        // Verify NFO was created in same folder
        let nfo_path = temp.path().join("TEST-001.nfo");
        assert!(nfo_path.exists());

        // Verify original file still exists
        assert!(movie_path.exists());
    }

    #[test]
    fn test_skip_existing() {
        let temp = TempDir::new().unwrap();
        let movie_path = temp.path().join("source/movie.mp4");

        // Create source file
        fs::create_dir_all(movie_path.parent().unwrap()).unwrap();
        fs::write(&movie_path, "test content").unwrap();

        // Create existing destination
        let dest_path = temp.path().join("output/TEST-001/TEST-001.mp4");
        fs::create_dir_all(dest_path.parent().unwrap()).unwrap();
        fs::write(&dest_path, "existing content").unwrap();

        let config = ProcessorConfig {
            mode: ProcessingMode::Organizing,
            success_folder: temp.path().join("output"),
            location_rule: "number".to_string(), // Match test's expected destination
            naming_rule: "number".to_string(),   // Match test's expected destination
            skip_existing: true,
            link_mode: LinkMode::Move,
            create_nfo: false,
            download_images: false, // Disable to avoid async runtime in tests
            ..Default::default()
        };

        let ctx = ProcessingContext::new(
            movie_path.clone(),
            "TEST-001".to_string(),
            create_test_metadata(),
            config,
        );

        ctx.execute().unwrap();

        // Verify source still exists (was skipped)
        assert!(movie_path.exists());

        // Verify destination wasn't overwritten
        assert_eq!(fs::read_to_string(&dest_path).unwrap(), "existing content");
    }
}
