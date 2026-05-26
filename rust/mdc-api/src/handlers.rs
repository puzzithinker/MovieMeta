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
pub async fn create_job(Json(_request): Json<CreateJobRequest>) -> ApiResult<Json<JobResponse>> {
    Err(ApiError::NotImplemented(
        "Job creation is not yet implemented".to_string(),
    ))
}

/// List all jobs
#[derive(Debug, Deserialize)]
pub struct ListJobsQuery {
    /// Filter by status
    pub status: Option<String>,

    /// Limit results
    pub limit: Option<usize>,

    /// Offset for pagination
    pub offset: Option<usize>,
}

pub async fn list_jobs(Query(_query): Query<ListJobsQuery>) -> ApiResult<Json<JobListResponse>> {
    Err(ApiError::NotImplemented(
        "Job listing is not yet implemented".to_string(),
    ))
}

/// Get job by ID
pub async fn get_job(Path(job_id): Path<String>) -> ApiResult<Json<JobResponse>> {
    Err(ApiError::NotImplemented(format!(
        "Job retrieval is not yet implemented (id: {})",
        job_id
    )))
}

/// Cancel a job
pub async fn cancel_job(Path(job_id): Path<String>) -> ApiResult<Json<JobResponse>> {
    Err(ApiError::NotImplemented(format!(
        "Job cancellation is not yet implemented (id: {})",
        job_id
    )))
}

/// Retry a failed job
pub async fn retry_job(Path(job_id): Path<String>) -> ApiResult<Json<JobResponse>> {
    Err(ApiError::NotImplemented(format!(
        "Job retry is not yet implemented (id: {})",
        job_id
    )))
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
    Err(ApiError::NotImplemented(
        "Configuration retrieval is not yet implemented".to_string(),
    ))
}

/// Update configuration
pub async fn update_config(Json(_config): Json<ConfigResponse>) -> ApiResult<Json<ConfigResponse>> {
    Err(ApiError::NotImplemented(
        "Configuration update is not yet implemented".to_string(),
    ))
}

/// Get statistics
pub async fn get_stats() -> ApiResult<Json<StatsResponse>> {
    Err(ApiError::NotImplemented(
        "Statistics retrieval is not yet implemented".to_string(),
    ))
}
