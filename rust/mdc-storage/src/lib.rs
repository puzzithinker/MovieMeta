//! Movie Data Capture Storage Library
//!
//! This crate handles configuration and persistence for Movie Data Capture.

pub mod config;
pub mod models;
pub mod repository;

// Re-export commonly used types
pub use config::Config;
pub use models::{FailedFile, JobStats, JobStatus, ProcessingJob};
pub use repository::JobRepository;
