//! Movie Data Capture Core Library
//!
//! This crate contains the core types and business logic for Movie Data Capture.

pub mod batch;
pub mod file_ops;
pub mod logging;
pub mod nfo;
pub mod number_parser;
pub mod processor;
pub mod scanner;
pub mod types;
pub mod workflow;

// Re-export commonly used types
pub use batch::{BatchProcessor, ProcessingResult};
pub use file_ops::{execute_file_operation, move_file, move_subtitles, sanitize_filename};
pub use nfo::{generate_nfo, write_nfo};
pub use processor::{FileAttributes, LinkMode, ProcessingMode, ProcessingStats, Template};
pub use types::{ImageCutMode, JobStatus, MovieMetadata, ProcessingJob};
pub use workflow::{ProcessingContext, ProcessorConfig};
