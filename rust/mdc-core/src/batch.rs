//! Batch processing with concurrency control

use anyhow::Result;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Semaphore;

use crate::number_parser;
use crate::processor::ProcessingStats;
use crate::workflow::{ProcessingContext, ProcessorConfig};

/// Result of processing a single file
#[derive(Debug)]
pub struct ProcessingResult {
    /// File path that was processed
    pub file_path: PathBuf,

    /// Whether processing succeeded
    pub success: bool,

    /// Extracted movie number (if successful)
    pub number: Option<String>,

    /// Error message (if failed)
    pub error: Option<String>,
}

/// Batch processor for concurrent movie processing
pub struct BatchProcessor {
    /// Configuration for processing
    config: ProcessorConfig,

    /// Maximum concurrent tasks
    max_concurrent: usize,
}

impl BatchProcessor {
    /// Create a new batch processor
    pub fn new(config: ProcessorConfig, max_concurrent: usize) -> Self {
        Self {
            config,
            max_concurrent,
        }
    }

    /// Process a single file
    async fn process_file<F, Fut>(
        &self,
        file_path: PathBuf,
        metadata_provider: Arc<F>,
    ) -> ProcessingResult
    where
        F: Fn(String) -> Fut + Send + Sync,
        Fut: std::future::Future<Output = Result<serde_json::Value>> + Send,
    {
        // Extract movie number from filename
        let file_path_str = file_path.to_string_lossy().to_string();
        let number = match number_parser::get_number(&file_path_str, None) {
            Ok(num) => num,
            Err(e) => {
                return ProcessingResult {
                    file_path,
                    success: false,
                    number: None,
                    error: Some(format!("Number parsing error: {}", e)),
                };
            }
        };

        // Fetch metadata
        let metadata = match metadata_provider(number.clone()).await {
            Ok(meta) => meta,
            Err(e) => {
                return ProcessingResult {
                    file_path,
                    success: false,
                    number: Some(number),
                    error: Some(format!("Metadata fetch error: {}", e)),
                };
            }
        };

        // Create processing context
        let context = ProcessingContext::new(
            file_path.clone(),
            number.clone(),
            metadata,
            self.config.clone(),
        );

        // Execute workflow
        match context.execute() {
            Ok(_) => ProcessingResult {
                file_path,
                success: true,
                number: Some(number),
                error: None,
            },
            Err(e) => ProcessingResult {
                file_path,
                success: false,
                number: Some(number),
                error: Some(format!("Processing error: {}", e)),
            },
        }
    }

    /// Process multiple files concurrently
    pub async fn process_batch<F, Fut>(
        &self,
        files: Vec<PathBuf>,
        metadata_provider: Arc<F>,
        progress_callback: Option<Arc<dyn Fn(usize, usize) + Send + Sync>>,
    ) -> Result<(Vec<ProcessingResult>, ProcessingStats)>
    where
        F: Fn(String) -> Fut + Send + Sync + 'static,
        Fut: std::future::Future<Output = Result<serde_json::Value>> + Send + 'static,
    {
        let semaphore = Arc::new(Semaphore::new(self.max_concurrent));
        let total = files.len();
        let mut tasks = Vec::new();

        for (index, file_path) in files.into_iter().enumerate() {
            let sem = semaphore.clone();
            let meta_provider = metadata_provider.clone();
            let processor = self.clone();
            let progress = progress_callback.clone();

            let task = tokio::spawn(async move {
                let _permit = sem.acquire().await.unwrap();

                let result = processor.process_file(file_path, meta_provider).await;

                // Report progress
                if let Some(callback) = progress {
                    callback(index + 1, total);
                }

                result
            });

            tasks.push(task);
        }

        // Wait for all tasks to complete
        let mut results = Vec::new();
        for task in tasks {
            if let Ok(result) = task.await {
                results.push(result);
            }
        }

        // Calculate statistics
        let stats = ProcessingStats {
            total_processed: results.len(),
            succeeded: results.iter().filter(|r| r.success).count(),
            failed: results.iter().filter(|r| !r.success).count(),
            skipped: 0,
        };

        Ok((results, stats))
    }
}

impl Clone for BatchProcessor {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            max_concurrent: self.max_concurrent,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::fs;
    use tempfile::TempDir;
    use crate::processor::{ProcessingMode, LinkMode};

    #[tokio::test]
    async fn test_batch_processor() {
        let temp = TempDir::new().unwrap();

        // Create test files
        let file1 = temp.path().join("TEST-001.mp4");
        let file2 = temp.path().join("TEST-002.mp4");
        fs::write(&file1, "content1").unwrap();
        fs::write(&file2, "content2").unwrap();

        let config = ProcessorConfig {
            mode: ProcessingMode::Organizing,
            link_mode: LinkMode::Move,
            success_folder: temp.path().join("output"),
            location_rule: "number".to_string(),
            naming_rule: "number".to_string(),
            create_nfo: false,
            ..Default::default()
        };

        let processor = BatchProcessor::new(config, 2);

        // Mock metadata provider
        let metadata_provider = Arc::new(|number: String| async move {
            Ok(json!({
                "number": number,
                "title": format!("Movie {}", number),
                "studio": "Test Studio"
            }))
        });

        let files = vec![file1.clone(), file2.clone()];
        let (results, stats) = processor
            .process_batch(files, metadata_provider, None)
            .await
            .unwrap();

        assert_eq!(results.len(), 2);
        assert_eq!(stats.succeeded, 2);
        assert_eq!(stats.failed, 0);

        // Verify files were moved
        assert!(!file1.exists());
        assert!(!file2.exists());
        assert!(temp.path().join("output/TEST-001/TEST-001.mp4").exists());
        assert!(temp.path().join("output/TEST-002/TEST-002.mp4").exists());
    }

    #[tokio::test]
    async fn test_batch_processor_with_progress() {
        let temp = TempDir::new().unwrap();

        let file1 = temp.path().join("TEST-001.mp4");
        fs::write(&file1, "content").unwrap();

        let config = ProcessorConfig {
            mode: ProcessingMode::Organizing,
            success_folder: temp.path().join("output"),
            create_nfo: false,
            ..Default::default()
        };

        let processor = BatchProcessor::new(config, 1);

        let metadata_provider = Arc::new(|number: String| async move {
            Ok(json!({
                "number": number,
                "title": "Test"
            }))
        });

        // Track progress
        let progress_called = Arc::new(std::sync::Mutex::new(false));
        let progress_called_clone = progress_called.clone();

        let progress_callback = Arc::new(move |current: usize, total: usize| {
            *progress_called_clone.lock().unwrap() = true;
            assert_eq!(current, 1);
            assert_eq!(total, 1);
        });

        let files = vec![file1];
        processor
            .process_batch(files, metadata_provider, Some(progress_callback))
            .await
            .unwrap();

        assert!(*progress_called.lock().unwrap());
    }

    #[tokio::test]
    async fn test_batch_processor_error_handling() {
        let temp = TempDir::new().unwrap();

        // File with invalid number format
        let file1 = temp.path().join("invalid_movie.mp4");
        fs::write(&file1, "content").unwrap();

        let config = ProcessorConfig {
            mode: ProcessingMode::Organizing,
            success_folder: temp.path().join("output"),
            ..Default::default()
        };

        let processor = BatchProcessor::new(config, 1);

        let metadata_provider = Arc::new(|_number: String| async move {
            Err(anyhow::anyhow!("Metadata not found"))
        });

        let files = vec![file1];
        let (results, stats) = processor
            .process_batch(files, metadata_provider, None)
            .await
            .unwrap();

        assert_eq!(results.len(), 1);
        assert!(!results[0].success);
        assert_eq!(stats.succeeded, 0);
        assert_eq!(stats.failed, 1);
    }
}
