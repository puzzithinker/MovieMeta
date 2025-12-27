//! Simple API tests
//!
//! These tests verify the API module compiles and basic types work.

/// Test that the API module compiles and can create a router
#[test]
fn test_router_creation() {
    let _app = mdc_api::create_router();
    // If this compiles and runs, the router is correctly configured
}

/// Test error types work correctly
#[test]
fn test_error_types() {
    use mdc_api::ApiError;

    let error = ApiError::NotFound("test".to_string());
    assert!(error.to_string().contains("Not found"));

    let error = ApiError::BadRequest("bad input".to_string());
    assert!(error.to_string().contains("Bad request"));

    let error = ApiError::Internal("server error".to_string());
    assert!(error.to_string().contains("Internal error"));
}

/// Test models serialize/deserialize correctly
#[test]
fn test_model_serialization() {
    use mdc_api::models::*;

    // Test CreateJobRequest
    let request = CreateJobRequest {
        files: vec!["/test/file.mp4".into()],
        mode: 1,
        link_mode: 0,
        output_folder: Some("/output".into()),
        location_rule: "number".to_string(),
        naming_rule: "number".to_string(),
        concurrent: 4,
    };

    let json = serde_json::to_string(&request).unwrap();
    assert!(json.contains("file.mp4"));
    assert!(json.contains("\"mode\":1"));

    // Test ScanRequest
    let scan = ScanRequest {
        path: "/movies".into(),
        media_types: vec!["mp4".to_string(), "mkv".to_string()],
    };

    let json = serde_json::to_string(&scan).unwrap();
    assert!(json.contains("movies"));
    assert!(json.contains("mp4"));
}

/// Test ProgressMessage serialization
#[test]
fn test_progress_message() {
    use mdc_api::models::ProgressMessage;

    let msg = ProgressMessage::JobStarted {
        job_id: "test-123".to_string(),
        total_files: 10,
    };

    let json = serde_json::to_string(&msg).unwrap();
    assert!(json.contains("test-123"));
    assert!(json.contains("10"));

    let msg = ProgressMessage::FileCompleted {
        job_id: "test-123".to_string(),
        file_path: "/test/movie.mp4".to_string(),
        success: true,
        error: None,
        current: 5,
        total: 10,
    };

    let json = serde_json::to_string(&msg).unwrap();
    assert!(json.contains("movie.mp4"));
    assert!(json.contains("true"));
}

/// Test JobStatus enum
#[test]
fn test_job_status() {
    use mdc_api::models::JobStatus;

    let status = JobStatus::Pending;
    let json = serde_json::to_string(&status).unwrap();
    assert_eq!(json, "\"pending\"");

    let status = JobStatus::Processing;
    let json = serde_json::to_string(&status).unwrap();
    assert_eq!(json, "\"processing\"");

    let status = JobStatus::Completed;
    let json = serde_json::to_string(&status).unwrap();
    assert_eq!(json, "\"completed\"");
}
