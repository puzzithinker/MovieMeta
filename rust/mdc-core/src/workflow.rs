//! Main processing workflow orchestration

use anyhow::{Context, Result};
use std::path::PathBuf;

use crate::file_ops::{execute_file_operation, get_video_extension, move_subtitles, sanitize_filename};
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
            location_rule: "number".to_string(),
            naming_rule: "number".to_string(),
            max_title_len: 50,
            skip_existing: false,
            download_images: true,
            create_nfo: true,
            move_subtitles: true,
        }
    }
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
        let components: Vec<String> = location
            .split('/')
            .map(|s| sanitize_filename(s))
            .collect();

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

        // TODO: Download images (requires image processing module integration)
        // This would call mdc-image crate to download cover, fanart, etc.

        // Move/link the movie file
        execute_file_operation(&self.movie_path, &dest_path, self.config.link_mode)?;

        // Move subtitles if enabled
        if self.config.move_subtitles {
            let _ = move_subtitles(&self.movie_path, &dest_folder, &base_name, self.config.link_mode);
        }

        tracing::info!("Processing complete: {} -> {}", self.movie_path.display(), dest_path.display());
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
            let _ = move_subtitles(&self.movie_path, &dest_folder, &base_name, self.config.link_mode);
        }

        tracing::info!("Organizing complete: {} -> {}", self.movie_path.display(), dest_path.display());
        Ok(())
    }

    /// Mode 3: Analysis mode (scrape in-place, no file moving)
    fn execute_analysis_mode(&self) -> Result<()> {
        let source_folder = self.movie_path.parent()
            .ok_or_else(|| anyhow::anyhow!("Cannot determine parent folder"))?;
        let base_name = self.get_destination_filename_base()?;

        // Generate and write NFO in the same folder
        if self.config.create_nfo {
            let nfo_path = source_folder.join(format!("{}.nfo", base_name));
            let nfo_content = generate_nfo(&self.metadata, &self.number)?;
            write_nfo(&nfo_path, &nfo_content)?;
        }

        // TODO: Download images to source folder
        // This would call mdc-image crate to download cover, fanart, etc.

        tracing::info!("Analysis complete (in-place): {}", self.movie_path.display());
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
            skip_existing: true,
            link_mode: LinkMode::Move,
            create_nfo: false,
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
