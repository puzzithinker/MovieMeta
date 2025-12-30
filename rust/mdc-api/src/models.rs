//! API request and response models

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Request to create a new processing job
#[derive(Debug, Deserialize, Serialize)]
pub struct CreateJobRequest {
    /// Files to process
    pub files: Vec<PathBuf>,

    /// Processing mode (1=Scraping, 2=Organizing, 3=Analysis)
    #[serde(default = "default_mode")]
    pub mode: u8,

    /// Link mode (0=move, 1=soft link, 2=hard link)
    #[serde(default)]
    pub link_mode: u8,

    /// Output folder
    pub output_folder: Option<PathBuf>,

    /// Location rule template
    #[serde(default = "default_location_rule")]
    pub location_rule: String,

    /// Naming rule template
    #[serde(default = "default_naming_rule")]
    pub naming_rule: String,

    /// Maximum concurrent tasks
    #[serde(default = "default_concurrent")]
    pub concurrent: usize,
}

fn default_mode() -> u8 {
    1
}

fn default_location_rule() -> String {
    "number".to_string()
}

fn default_naming_rule() -> String {
    "number".to_string()
}

fn default_concurrent() -> usize {
    4
}

/// Request to scan a folder for movie files
#[derive(Debug, Deserialize, Serialize)]
pub struct ScanRequest {
    /// Folder path to scan
    pub path: PathBuf,

    /// Media file extensions to include
    #[serde(default = "default_media_types")]
    pub media_types: Vec<String>,
}

fn default_media_types() -> Vec<String> {
    vec![
        "mp4".to_string(),
        "avi".to_string(),
        "mkv".to_string(),
        "wmv".to_string(),
        "mov".to_string(),
        "flv".to_string(),
        "rmvb".to_string(),
        "ts".to_string(),
        "webm".to_string(),
        "iso".to_string(),
        "mpg".to_string(),
        "m4v".to_string(),
    ]
}

/// Response with scan results
#[derive(Debug, Serialize)]
pub struct ScanResponse {
    /// Total files found
    pub total: usize,

    /// List of found files
    pub files: Vec<PathBuf>,
}

/// Job status in the system
#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum JobStatus {
    Pending,
    Processing,
    Completed,
    Failed,
    Cancelled,
}

/// Job information response
#[derive(Debug, Serialize)]
pub struct JobResponse {
    /// Job ID
    pub id: String,

    /// Job status
    pub status: JobStatus,

    /// Total files in job
    pub total_files: usize,

    /// Files processed so far
    pub processed_files: usize,

    /// Files succeeded
    pub succeeded: usize,

    /// Files failed
    pub failed: usize,

    /// Error message (if failed)
    pub error: Option<String>,

    /// Creation timestamp
    pub created_at: String,

    /// Completion timestamp
    pub completed_at: Option<String>,
}

/// List of jobs response
#[derive(Debug, Serialize)]
pub struct JobListResponse {
    /// List of jobs
    pub jobs: Vec<JobResponse>,

    /// Total count
    pub total: usize,
}

/// Statistics response
#[derive(Debug, Serialize)]
pub struct StatsResponse {
    /// Total jobs processed
    pub total_jobs: usize,

    /// Jobs succeeded
    pub succeeded_jobs: usize,

    /// Jobs failed
    pub failed_jobs: usize,

    /// Total files processed
    pub total_files: usize,

    /// Files succeeded
    pub succeeded_files: usize,

    /// Files failed
    pub failed_files: usize,
}

/// Configuration response
#[derive(Debug, Serialize, Deserialize)]
pub struct ConfigResponse {
    /// Default processing mode
    pub default_mode: u8,

    /// Default link mode
    pub default_link_mode: u8,

    /// Default output folder
    pub default_output_folder: String,

    /// Default location rule
    pub default_location_rule: String,

    /// Default naming rule
    pub default_naming_rule: String,

    /// Default concurrency
    pub default_concurrent: usize,
}

/// WebSocket message for progress updates
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum ProgressMessage {
    /// Job started
    JobStarted { job_id: String, total_files: usize },

    /// File processing started
    FileStarted {
        job_id: String,
        file_path: String,
        current: usize,
        total: usize,
    },

    /// File processing completed
    FileCompleted {
        job_id: String,
        file_path: String,
        success: bool,
        error: Option<String>,
        current: usize,
        total: usize,
    },

    /// Job completed
    JobCompleted {
        job_id: String,
        total: usize,
        succeeded: usize,
        failed: usize,
    },

    /// Job failed
    JobFailed { job_id: String, error: String },
}
