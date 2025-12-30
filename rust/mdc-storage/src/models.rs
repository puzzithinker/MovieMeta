//! Database models for storage

use chrono::Utc;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// Job status enum
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "TEXT")]
pub enum JobStatus {
    #[sqlx(rename = "pending")]
    Pending,
    #[sqlx(rename = "processing")]
    Processing,
    #[sqlx(rename = "completed")]
    Completed,
    #[sqlx(rename = "failed")]
    Failed,
}

impl std::fmt::Display for JobStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            JobStatus::Pending => write!(f, "pending"),
            JobStatus::Processing => write!(f, "processing"),
            JobStatus::Completed => write!(f, "completed"),
            JobStatus::Failed => write!(f, "failed"),
        }
    }
}

impl std::str::FromStr for JobStatus {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "pending" => Ok(JobStatus::Pending),
            "processing" => Ok(JobStatus::Processing),
            "completed" => Ok(JobStatus::Completed),
            "failed" => Ok(JobStatus::Failed),
            _ => Err(format!("Invalid job status: {}", s)),
        }
    }
}

/// Processing job record
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct ProcessingJob {
    /// Unique job ID
    pub id: String,

    /// Path to the movie file
    pub file_path: String,

    /// Extracted movie number (if known)
    pub number: Option<String>,

    /// Current job status
    pub status: String,

    /// Scraped metadata as JSON
    pub metadata_json: Option<String>,

    /// Error message if failed
    pub error_message: Option<String>,

    /// When the job was created
    pub created_at: String,

    /// When the job was last updated
    pub updated_at: String,

    /// When the job completed
    pub completed_at: Option<String>,
}

impl ProcessingJob {
    /// Create a new pending job
    pub fn new(file_path: String, number: Option<String>) -> Self {
        let now = Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
        Self {
            id: Uuid::new_v4().to_string(),
            file_path,
            number,
            status: "pending".to_string(),
            metadata_json: None,
            error_message: None,
            created_at: now.clone(),
            updated_at: now,
            completed_at: None,
        }
    }

    /// Get status as enum
    pub fn status_enum(&self) -> Result<JobStatus, String> {
        self.status.parse()
    }

    /// Check if job is complete
    pub fn is_complete(&self) -> bool {
        matches!(
            self.status_enum(),
            Ok(JobStatus::Completed) | Ok(JobStatus::Failed)
        )
    }

    /// Check if job is in progress
    pub fn is_in_progress(&self) -> bool {
        matches!(self.status_enum(), Ok(JobStatus::Processing))
    }

    /// Check if job is pending
    pub fn is_pending(&self) -> bool {
        matches!(self.status_enum(), Ok(JobStatus::Pending))
    }
}

/// Failed file record
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct FailedFile {
    /// Path to the failed file
    pub file_path: String,

    /// Reason for failure
    pub reason: Option<String>,

    /// When it failed
    pub failed_at: String,
}

impl FailedFile {
    /// Create a new failed file record
    pub fn new(file_path: String, reason: Option<String>) -> Self {
        Self {
            file_path,
            reason,
            failed_at: Utc::now().format("%Y-%m-%d %H:%M:%S").to_string(),
        }
    }
}

/// Job statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobStats {
    /// Total number of jobs
    pub total: i64,

    /// Number of pending jobs
    pub pending: i64,

    /// Number of processing jobs
    pub processing: i64,

    /// Number of completed jobs
    pub completed: i64,

    /// Number of failed jobs
    pub failed: i64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_job_status_display() {
        assert_eq!(JobStatus::Pending.to_string(), "pending");
        assert_eq!(JobStatus::Processing.to_string(), "processing");
        assert_eq!(JobStatus::Completed.to_string(), "completed");
        assert_eq!(JobStatus::Failed.to_string(), "failed");
    }

    #[test]
    fn test_job_status_from_str() {
        assert_eq!("pending".parse::<JobStatus>().unwrap(), JobStatus::Pending);
        assert_eq!(
            "processing".parse::<JobStatus>().unwrap(),
            JobStatus::Processing
        );
        assert_eq!(
            "completed".parse::<JobStatus>().unwrap(),
            JobStatus::Completed
        );
        assert_eq!("failed".parse::<JobStatus>().unwrap(), JobStatus::Failed);
        assert!("invalid".parse::<JobStatus>().is_err());
    }

    #[test]
    fn test_processing_job_new() {
        let job = ProcessingJob::new(
            "/path/to/movie.mp4".to_string(),
            Some("TEST-001".to_string()),
        );

        assert!(!job.id.is_empty());
        assert_eq!(job.file_path, "/path/to/movie.mp4");
        assert_eq!(job.number, Some("TEST-001".to_string()));
        assert_eq!(job.status, "pending");
        assert!(job.metadata_json.is_none());
        assert!(job.error_message.is_none());
        assert!(job.completed_at.is_none());
        assert!(job.is_pending());
        assert!(!job.is_complete());
    }

    #[test]
    fn test_failed_file_new() {
        let failed = FailedFile::new(
            "/path/to/failed.mp4".to_string(),
            Some("No metadata found".to_string()),
        );

        assert_eq!(failed.file_path, "/path/to/failed.mp4");
        assert_eq!(failed.reason, Some("No metadata found".to_string()));
        assert!(!failed.failed_at.is_empty());
    }
}
