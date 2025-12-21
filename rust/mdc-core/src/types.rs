use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use uuid::Uuid;

/// Represents movie metadata scraped from various sources
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MovieMetadata {
    /// Movie number/ID (e.g., "ABC-123", "FC2-PPV-1234567")
    pub number: String,

    /// Movie title
    pub title: String,

    /// Production studio
    #[serde(skip_serializing_if = "Option::is_none")]
    pub studio: Option<String>,

    /// Release date (YYYY-MM-DD)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub release: Option<NaiveDate>,

    /// Release year (parsed from release date)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub year: Option<String>,

    /// Plot summary/outline
    #[serde(skip_serializing_if = "Option::is_none")]
    pub outline: Option<String>,

    /// Runtime in minutes
    #[serde(skip_serializing_if = "Option::is_none")]
    pub runtime: Option<u32>,

    /// Director name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub director: Option<String>,

    /// List of actor names
    #[serde(default)]
    pub actor: Vec<String>,

    /// Actor photos: actor_name -> photo_url
    #[serde(default)]
    pub actor_photo: HashMap<String, String>,

    /// Main cover image URL
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cover: Option<String>,

    /// Small cover/thumbnail URL
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cover_small: Option<String>,

    /// Extra fanart image URLs
    #[serde(default)]
    pub extrafanart: Vec<String>,

    /// Trailer video URL
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trailer: Option<String>,

    /// Genre/category tags
    #[serde(default)]
    pub tag: Vec<String>,

    /// Publisher/label
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,

    /// Series name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub series: Option<String>,

    /// User rating (0-10 scale)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub userrating: Option<f32>,

    /// Number of user votes
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uservotes: Option<u32>,

    /// Whether the movie is uncensored
    #[serde(default)]
    pub uncensored: bool,

    /// Source website detail page URL
    #[serde(skip_serializing_if = "Option::is_none")]
    pub website: Option<String>,

    /// Source name (e.g., "tmdb", "imdb")
    pub source: String,

    /// Image cropping mode
    #[serde(default)]
    pub imagecut: ImageCutMode,
}

impl MovieMetadata {
    /// Get the release year from the release date
    pub fn get_year(&self) -> Option<String> {
        self.year
            .clone()
            .or_else(|| self.release.map(|d| d.format("%Y").to_string()))
    }
}

/// Image cropping mode
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum ImageCutMode {
    /// Copy cover as-is without cropping
    #[default]
    Copy = 0,
    /// Smart crop using face detection
    Smart = 1,
    /// Download small cover variant
    Small = 3,
}

impl From<i32> for ImageCutMode {
    fn from(value: i32) -> Self {
        match value {
            0 => ImageCutMode::Copy,
            1 => ImageCutMode::Smart,
            3 => ImageCutMode::Small,
            _ => ImageCutMode::Smart, // Default to smart
        }
    }
}

impl From<ImageCutMode> for i32 {
    fn from(mode: ImageCutMode) -> Self {
        mode as i32
    }
}

/// Represents a processing job for a movie file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessingJob {
    /// Unique job ID
    pub id: Uuid,

    /// Path to the movie file
    pub file_path: PathBuf,

    /// Extracted movie number (if available)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub number: Option<String>,

    /// Current job status
    pub status: JobStatus,

    /// Scraped metadata (if available)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<MovieMetadata>,

    /// Error message (if failed)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,

    /// When the job was created
    pub created_at: DateTime<Utc>,

    /// When the job was last updated
    pub updated_at: DateTime<Utc>,

    /// When the job was completed (if applicable)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completed_at: Option<DateTime<Utc>>,
}

impl ProcessingJob {
    /// Create a new pending job for a file path
    pub fn new(file_path: PathBuf) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            file_path,
            number: None,
            status: JobStatus::Pending,
            metadata: None,
            error: None,
            created_at: now,
            updated_at: now,
            completed_at: None,
        }
    }

    /// Update job status
    pub fn update_status(&mut self, status: JobStatus) {
        self.status = status;
        self.updated_at = Utc::now();

        if matches!(status, JobStatus::Completed | JobStatus::Failed) {
            self.completed_at = Some(Utc::now());
        }
    }

    /// Set metadata
    pub fn set_metadata(&mut self, metadata: MovieMetadata) {
        self.metadata = Some(metadata);
        self.updated_at = Utc::now();
    }

    /// Set error
    pub fn set_error(&mut self, error: String) {
        self.error = Some(error);
        self.updated_at = Utc::now();
    }
}

/// Processing job status
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum JobStatus {
    /// Job is queued and waiting to be processed
    Pending,
    /// Job is currently being processed
    Processing,
    /// Job completed successfully
    Completed,
    /// Job failed with an error
    Failed,
}

impl JobStatus {
    /// Check if a transition to another status is valid
    pub fn can_transition_to(&self, next: JobStatus) -> bool {
        matches!(
            (self, next),
            (JobStatus::Pending, JobStatus::Processing)
                | (JobStatus::Processing, JobStatus::Completed)
                | (JobStatus::Processing, JobStatus::Failed)
                | (JobStatus::Failed, JobStatus::Pending) // Allow retry
        )
    }

    /// Convert to string representation
    pub fn as_str(&self) -> &'static str {
        match self {
            JobStatus::Pending => "pending",
            JobStatus::Processing => "processing",
            JobStatus::Completed => "completed",
            JobStatus::Failed => "failed",
        }
    }
}

impl std::fmt::Display for JobStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_job_status_transitions() {
        assert!(JobStatus::Pending.can_transition_to(JobStatus::Processing));
        assert!(JobStatus::Processing.can_transition_to(JobStatus::Completed));
        assert!(JobStatus::Processing.can_transition_to(JobStatus::Failed));
        assert!(JobStatus::Failed.can_transition_to(JobStatus::Pending)); // Retry

        // Invalid transitions
        assert!(!JobStatus::Pending.can_transition_to(JobStatus::Completed));
        assert!(!JobStatus::Completed.can_transition_to(JobStatus::Processing));
    }

    #[test]
    fn test_processing_job_creation() {
        let job = ProcessingJob::new(PathBuf::from("/test/movie.mp4"));

        assert_eq!(job.status, JobStatus::Pending);
        assert!(job.metadata.is_none());
        assert!(job.error.is_none());
        assert!(job.completed_at.is_none());
    }

    #[test]
    fn test_processing_job_status_update() {
        let mut job = ProcessingJob::new(PathBuf::from("/test/movie.mp4"));

        job.update_status(JobStatus::Processing);
        assert_eq!(job.status, JobStatus::Processing);
        assert!(job.completed_at.is_none());

        job.update_status(JobStatus::Completed);
        assert_eq!(job.status, JobStatus::Completed);
        assert!(job.completed_at.is_some());
    }

    #[test]
    fn test_image_cut_mode_conversion() {
        assert_eq!(ImageCutMode::from(0), ImageCutMode::Copy);
        assert_eq!(ImageCutMode::from(1), ImageCutMode::Smart);
        assert_eq!(ImageCutMode::from(3), ImageCutMode::Small);
        assert_eq!(ImageCutMode::from(99), ImageCutMode::Smart); // Default

        assert_eq!(i32::from(ImageCutMode::Copy), 0);
        assert_eq!(i32::from(ImageCutMode::Smart), 1);
        assert_eq!(i32::from(ImageCutMode::Small), 3);
    }

    #[test]
    fn test_metadata_year_extraction() {
        let mut metadata = MovieMetadata {
            number: "TEST-001".to_string(),
            title: "Test Movie".to_string(),
            source: "test".to_string(),
            release: Some(NaiveDate::from_ymd_opt(2024, 1, 15).unwrap()),
            year: None,
            studio: None,
            outline: None,
            runtime: None,
            director: None,
            actor: vec![],
            actor_photo: HashMap::new(),
            cover: None,
            cover_small: None,
            extrafanart: vec![],
            trailer: None,
            tag: vec![],
            label: None,
            series: None,
            userrating: None,
            uservotes: None,
            uncensored: false,
            website: None,
            imagecut: ImageCutMode::Smart,
        };

        // Should extract year from release date
        assert_eq!(metadata.get_year(), Some("2024".to_string()));

        // Explicit year takes precedence
        metadata.year = Some("2023".to_string());
        assert_eq!(metadata.get_year(), Some("2023".to_string()));
    }
}
