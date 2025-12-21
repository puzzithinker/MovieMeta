//! Movie Data Capture Core Library
//!
//! This crate contains the core types and business logic for Movie Data Capture.

pub mod logging;
pub mod number_parser;
pub mod scanner;
pub mod types;

// Re-export commonly used types
pub use types::{ImageCutMode, JobStatus, MovieMetadata, ProcessingJob};
