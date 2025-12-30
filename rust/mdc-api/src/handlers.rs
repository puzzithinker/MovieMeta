//! API endpoint handlers

use axum::{
    extract::{Path, Query},
    Json,
};
use serde::Deserialize;

use crate::{
    error::{ApiError, ApiResult},
    models::*,
};

/// Health check endpoint
pub async fn health() -> &'static str {
    "OK"
}

/// Create a new processing job
pub async fn create_job(Json(request): Json<CreateJobRequest>) -> ApiResult<Json<JobResponse>> {
    // TODO: Implement job creation with mdc-core
    // For now, return a mock response

    let job_id = uuid::Uuid::new_v4().to_string();

    Ok(Json(JobResponse {
        id: job_id,
        status: JobStatus::Pending,
        total_files: request.files.len(),
        processed_files: 0,
        succeeded: 0,
        failed: 0,
        error: None,
        created_at: chrono::Utc::now().to_rfc3339(),
        completed_at: None,
    }))
}

/// List all jobs
#[derive(Debug, Deserialize)]
pub struct ListJobsQuery {
    /// Filter by status
    status: Option<String>,

    /// Limit results
    limit: Option<usize>,

    /// Offset for pagination
    offset: Option<usize>,
}

pub async fn list_jobs(Query(_query): Query<ListJobsQuery>) -> ApiResult<Json<JobListResponse>> {
    // TODO: Implement job listing from mdc-storage
    // For now, return empty list

    Ok(Json(JobListResponse {
        jobs: vec![],
        total: 0,
    }))
}

/// Get job by ID
pub async fn get_job(Path(job_id): Path<String>) -> ApiResult<Json<JobResponse>> {
    // TODO: Implement job retrieval from mdc-storage
    // For now, return not found

    Err(ApiError::NotFound(format!("Job not found: {}", job_id)))
}

/// Cancel a job
pub async fn cancel_job(Path(job_id): Path<String>) -> ApiResult<Json<JobResponse>> {
    // TODO: Implement job cancellation
    // For now, return not found

    Err(ApiError::NotFound(format!("Job not found: {}", job_id)))
}

/// Retry a failed job
pub async fn retry_job(Path(job_id): Path<String>) -> ApiResult<Json<JobResponse>> {
    // TODO: Implement job retry
    // For now, return not found

    Err(ApiError::NotFound(format!("Job not found: {}", job_id)))
}

/// Scan folder for movie files
pub async fn scan_folder(Json(request): Json<ScanRequest>) -> ApiResult<Json<ScanResponse>> {
    // TODO: Implement folder scanning with mdc-core scanner
    // For now, return empty results

    use mdc_core::scanner;

    let media_types: Vec<&str> = request.media_types.iter().map(|s| s.as_str()).collect();

    let files = scanner::scan_directory(&request.path, &media_types)
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?;

    Ok(Json(ScanResponse {
        total: files.len(),
        files,
    }))
}

/// Get configuration
pub async fn get_config() -> ApiResult<Json<ConfigResponse>> {
    // TODO: Load from mdc-storage config
    // For now, return defaults

    Ok(Json(ConfigResponse {
        default_mode: 1,
        default_link_mode: 0,
        default_output_folder: "./output".to_string(),
        default_location_rule: "number".to_string(),
        default_naming_rule: "number".to_string(),
        default_concurrent: 4,
    }))
}

/// Update configuration
pub async fn update_config(Json(config): Json<ConfigResponse>) -> ApiResult<Json<ConfigResponse>> {
    // TODO: Save to mdc-storage config
    // For now, just return the config back

    Ok(Json(config))
}

/// Get statistics
pub async fn get_stats() -> ApiResult<Json<StatsResponse>> {
    // TODO: Get stats from mdc-storage
    // For now, return mock stats

    Ok(Json(StatsResponse {
        total_jobs: 0,
        succeeded_jobs: 0,
        failed_jobs: 0,
        total_files: 0,
        succeeded_files: 0,
        failed_files: 0,
    }))
}
