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

/// Dual ID format for scrapers
#[derive(Debug, Clone)]
pub struct DualId {
    /// Display format (e.g., "SSIS-123")
    pub display: String,
    /// Content format (e.g., "ssis00123")
    pub content: String,
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
        F: Fn(DualId) -> Fut + Send + Sync,
        Fut: std::future::Future<Output = Result<serde_json::Value>> + Send,
    {
        // Extract movie number from filename using parse_number (dual ID support)
        let file_path_str = file_path.to_string_lossy().to_string();
        let parsed = match number_parser::parse_number(&file_path_str, None) {
            Ok(p) => p,
            Err(e) => {
                return ProcessingResult {
                    file_path,
                    success: false,
                    number: None,
                    error: Some(format!("Number parsing error: {}", e)),
                };
            }
        };

        // Extract dual IDs
        let dual_id = DualId {
            display: parsed.id.clone(),
            content: parsed.content_id.clone(),
        };
        let number_display = parsed.id.clone();

        // Fetch metadata using dual ID
        let metadata = match metadata_provider(dual_id).await {
            Ok(meta) => meta,
            Err(e) => {
                return ProcessingResult {
                    file_path,
                    success: false,
                    number: Some(number_display),
                    error: Some(format!("Metadata fetch error: {}", e)),
                };
            }
        };

        // Create processing context
        let context = ProcessingContext::new(
            file_path.clone(),
            number_display.clone(),
            metadata,
            self.config.clone(),
        );

        // Execute workflow
        match context.execute() {
            Ok(_) => ProcessingResult {
                file_path,
                success: true,
                number: Some(number_display),
                error: None,
            },
            Err(e) => ProcessingResult {
                file_path,
                success: false,
                number: Some(number_display),
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
        F: Fn(DualId) -> Fut + Send + Sync + 'static,
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
    use crate::processor::{LinkMode, ProcessingMode};
    use serde_json::json;
    use std::fs;
    use tempfile::TempDir;

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
        let metadata_provider = Arc::new(|dual_id: DualId| async move {
            Ok(json!({
                "number": dual_id.display,
                "title": format!("Movie {}", dual_id.display),
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

        let metadata_provider = Arc::new(|dual_id: DualId| async move {
            Ok(json!({
                "number": dual_id.display,
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

        let metadata_provider =
            Arc::new(|_dual_id: DualId| async move { Err(anyhow::anyhow!("Metadata not found")) });

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

    #[tokio::test]
    async fn test_batch_processor_concurrent_limit() {
        use std::sync::atomic::{AtomicUsize, Ordering};
        use tokio::time::{sleep, Duration};

        let temp = TempDir::new().unwrap();

        // Create multiple test files
        let mut files = Vec::new();
        for i in 0..10 {
            let file = temp.path().join(format!("TEST-{:03}.mp4", i + 1));
            fs::write(&file, "content").unwrap();
            files.push(file);
        }

        let config = ProcessorConfig {
            mode: ProcessingMode::Organizing,
            link_mode: LinkMode::Move,
            success_folder: temp.path().join("output"),
            create_nfo: false,
            ..Default::default()
        };

        // Set max_concurrent to 3
        let processor = BatchProcessor::new(config, 3);

        // Track concurrent executions
        let concurrent_count = Arc::new(AtomicUsize::new(0));
        let max_concurrent = Arc::new(AtomicUsize::new(0));
        let concurrent_clone = concurrent_count.clone();
        let max_clone = max_concurrent.clone();

        let metadata_provider = Arc::new(move |dual_id: DualId| {
            let concurrent_clone = concurrent_clone.clone();
            let max_clone = max_clone.clone();
            async move {
                // Increment concurrent count
                let current = concurrent_clone.fetch_add(1, Ordering::SeqCst) + 1;

                // Update max if needed
                let mut max = max_clone.load(Ordering::SeqCst);
                while current > max {
                    match max_clone.compare_exchange(max, current, Ordering::SeqCst, Ordering::SeqCst) {
                        Ok(_) => break,
                        Err(x) => max = x,
                    }
                }

                // Simulate some work
                sleep(Duration::from_millis(50)).await;

                // Decrement concurrent count
                concurrent_clone.fetch_sub(1, Ordering::SeqCst);

                Ok(json!({
                    "number": dual_id.display,
                    "title": "Test"
                }))
            }
        });

        let (results, stats) = processor
            .process_batch(files, metadata_provider, None)
            .await
            .unwrap();

        assert_eq!(results.len(), 10);
        assert_eq!(stats.succeeded, 10);

        // Verify concurrent limit was respected (should be <= 3)
        let max = max_concurrent.load(Ordering::SeqCst);
        assert!(max <= 3, "Max concurrent was {}, expected <= 3", max);
        assert!(max > 0, "Should have had some concurrent execution");
    }

    #[tokio::test]
    async fn test_batch_processor_mixed_results() {
        let temp = TempDir::new().unwrap();

        // Create test files - some valid, some invalid
        let file1 = temp.path().join("TEST-001.mp4");
        let file2 = temp.path().join("invalid_name.mp4");
        let file3 = temp.path().join("TEST-003.mp4");
        fs::write(&file1, "content1").unwrap();
        fs::write(&file2, "content2").unwrap();
        fs::write(&file3, "content3").unwrap();

        let config = ProcessorConfig {
            mode: ProcessingMode::Organizing,
            link_mode: LinkMode::Move,
            success_folder: temp.path().join("output"),
            create_nfo: false,
            ..Default::default()
        };

        let processor = BatchProcessor::new(config, 2);

        // Provider that fails for specific numbers
        let metadata_provider = Arc::new(|dual_id: DualId| async move {
            if dual_id.display == "TEST-003" {
                Err(anyhow::anyhow!("Simulated fetch error"))
            } else {
                Ok(json!({
                    "number": dual_id.display,
                    "title": "Test"
                }))
            }
        });

        let files = vec![file1.clone(), file2.clone(), file3.clone()];
        let (results, stats) = processor
            .process_batch(files, metadata_provider, None)
            .await
            .unwrap();

        assert_eq!(results.len(), 3);

        // One should succeed (TEST-001), two should fail (invalid_name and TEST-003)
        assert_eq!(stats.succeeded, 1);
        assert_eq!(stats.failed, 2);

        // Check individual results
        let success_count = results.iter().filter(|r| r.success).count();
        let fail_count = results.iter().filter(|r| !r.success).count();
        assert_eq!(success_count, 1);
        assert_eq!(fail_count, 2);
    }

    #[tokio::test]
    async fn test_batch_processor_empty_batch() {
        let temp = TempDir::new().unwrap();

        let config = ProcessorConfig {
            mode: ProcessingMode::Organizing,
            success_folder: temp.path().join("output"),
            ..Default::default()
        };

        let processor = BatchProcessor::new(config, 2);

        let metadata_provider = Arc::new(|dual_id: DualId| async move {
            Ok(json!({
                "number": dual_id.display,
                "title": "Test"
            }))
        });

        let files = Vec::new();
        let (results, stats) = processor
            .process_batch(files, metadata_provider, None)
            .await
            .unwrap();

        assert_eq!(results.len(), 0);
        assert_eq!(stats.succeeded, 0);
        assert_eq!(stats.failed, 0);
        assert_eq!(stats.total_processed, 0);
    }

    #[tokio::test]
    async fn test_batch_processor_dual_id_format() {
        let temp = TempDir::new().unwrap();

        // Create files with various ID formats
        let file1 = temp.path().join("SSIS-123.mp4");
        let file2 = temp.path().join("FC2-PPV-1234567.mp4");
        fs::write(&file1, "content1").unwrap();
        fs::write(&file2, "content2").unwrap();

        let config = ProcessorConfig {
            mode: ProcessingMode::Organizing,
            link_mode: LinkMode::Move,
            success_folder: temp.path().join("output"),
            create_nfo: false,
            ..Default::default()
        };

        let processor = BatchProcessor::new(config, 2);

        // Track which dual IDs were passed
        let captured_ids = Arc::new(std::sync::Mutex::new(Vec::new()));
        let captured_clone = captured_ids.clone();

        let metadata_provider = Arc::new(move |dual_id: DualId| {
            captured_clone.lock().unwrap().push((dual_id.display.clone(), dual_id.content.clone()));
            async move {
                Ok(json!({
                    "number": dual_id.display,
                    "title": "Test"
                }))
            }
        });

        let files = vec![file1, file2];
        let (results, stats) = processor
            .process_batch(files, metadata_provider, None)
            .await
            .unwrap();

        assert_eq!(results.len(), 2);
        assert_eq!(stats.succeeded, 2);

        // Verify dual IDs were captured correctly
        let ids = captured_ids.lock().unwrap();
        assert_eq!(ids.len(), 2);

        // Check that we have both display and content formats
        assert!(ids.iter().any(|(display, _)| display == "SSIS-123"));
        assert!(ids.iter().any(|(display, content)| display == "FC2-PPV-1234567" && content.contains("fc2")));
    }

    #[tokio::test]
    async fn test_batch_processor_number_parsing_errors() {
        let temp = TempDir::new().unwrap();

        // Create files with various problematic names that won't parse
        let file1 = temp.path().join("no_number.mp4");
        let file2 = temp.path().join("invalid.mp4");
        fs::write(&file1, "content1").unwrap();
        fs::write(&file2, "content2").unwrap();

        let config = ProcessorConfig {
            mode: ProcessingMode::Organizing,
            success_folder: temp.path().join("output"),
            ..Default::default()
        };

        let processor = BatchProcessor::new(config, 2);

        let metadata_provider = Arc::new(|dual_id: DualId| async move {
            Ok(json!({
                "number": dual_id.display,
                "title": "Test"
            }))
        });

        let files = vec![file1, file2];
        let (results, stats) = processor
            .process_batch(files, metadata_provider, None)
            .await
            .unwrap();

        assert_eq!(results.len(), 2);

        // All should fail with number parsing errors
        assert_eq!(stats.failed, 2);
        assert_eq!(stats.succeeded, 0);

        // Verify error messages mention parsing
        for result in &results {
            assert!(!result.success);
            assert!(result.error.is_some());
            let error = result.error.as_ref().unwrap();
            assert!(error.contains("parsing") || error.contains("extract"),
                    "Error should mention parsing: {}", error);
        }
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_batch_processor_large_batch() {
        let temp = TempDir::new().unwrap();

        // Create a large batch (50 files)
        let mut files = Vec::new();
        for i in 0..50 {
            let file = temp.path().join(format!("TEST-{:03}.mp4", i + 1));
            fs::write(&file, "content").unwrap();
            files.push(file);
        }

        let config = ProcessorConfig {
            mode: ProcessingMode::Organizing,
            link_mode: LinkMode::Move,
            success_folder: temp.path().join("output"),
            create_nfo: false,
            ..Default::default()
        };

        let processor = BatchProcessor::new(config, 10);

        let metadata_provider = Arc::new(|dual_id: DualId| async move {
            Ok(json!({
                "number": dual_id.display,
                "title": "Test"
            }))
        });

        let (results, stats) = processor
            .process_batch(files, metadata_provider, None)
            .await
            .unwrap();

        assert_eq!(results.len(), 50);
        assert_eq!(stats.total_processed, 50);
        assert_eq!(stats.succeeded, 50);
        assert_eq!(stats.failed, 0);
    }

    #[tokio::test]
    async fn test_batch_processor_progress_accuracy() {
        let temp = TempDir::new().unwrap();

        // Create 5 test files
        let mut files = Vec::new();
        for i in 0..5 {
            let file = temp.path().join(format!("TEST-{:03}.mp4", i + 1));
            fs::write(&file, "content").unwrap();
            files.push(file);
        }

        let config = ProcessorConfig {
            mode: ProcessingMode::Organizing,
            link_mode: LinkMode::Move,
            success_folder: temp.path().join("output"),
            create_nfo: false,
            ..Default::default()
        };

        let processor = BatchProcessor::new(config, 2);

        let metadata_provider = Arc::new(|dual_id: DualId| async move {
            Ok(json!({
                "number": dual_id.display,
                "title": "Test"
            }))
        });

        // Track all progress updates
        let progress_updates = Arc::new(std::sync::Mutex::new(Vec::new()));
        let progress_clone = progress_updates.clone();

        let progress_callback = Arc::new(move |current: usize, total: usize| {
            progress_clone.lock().unwrap().push((current, total));
        });

        processor
            .process_batch(files, metadata_provider, Some(progress_callback))
            .await
            .unwrap();

        let updates = progress_updates.lock().unwrap();

        // Should have 5 progress updates (one per file)
        assert_eq!(updates.len(), 5);

        // All updates should report total=5
        assert!(updates.iter().all(|(_, total)| *total == 5));

        // Current should go from 1 to 5
        let mut currents: Vec<usize> = updates.iter().map(|(c, _)| *c).collect();
        currents.sort();
        assert_eq!(currents, vec![1, 2, 3, 4, 5]);
    }
}
