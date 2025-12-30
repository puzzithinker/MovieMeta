//! Database repository for jobs and failed files

use anyhow::Result;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePool, SqlitePoolOptions};
use sqlx::{Executor, Row};
use std::path::Path;
use std::str::FromStr;

use crate::models::{FailedFile, JobStats, JobStatus, ProcessingJob};

/// Job repository for database operations
#[derive(Debug, Clone)]
pub struct JobRepository {
    pool: SqlitePool,
}

impl JobRepository {
    /// Create a new repository with the given database path
    pub async fn new<P: AsRef<Path>>(db_path: P) -> Result<Self> {
        // Ensure parent directory exists
        if let Some(parent) = db_path.as_ref().parent() {
            std::fs::create_dir_all(parent)?;
        }

        let db_url = format!("sqlite:{}", db_path.as_ref().display());

        let options = SqliteConnectOptions::from_str(&db_url)?
            .create_if_missing(true)
            .foreign_keys(true);

        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect_with(options)
            .await?;

        let repo = Self { pool };

        // Run migrations
        repo.run_migrations().await?;

        Ok(repo)
    }

    /// Run database migrations
    async fn run_migrations(&self) -> Result<()> {
        // Read and execute migration
        let migration_sql = include_str!("../migrations/001_initial_schema.sql");

        self.pool.execute(migration_sql).await?;

        tracing::info!("Database migrations completed");
        Ok(())
    }

    /// Create a new job
    pub async fn create_job(&self, job: &ProcessingJob) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO processing_jobs (id, file_path, number, status, metadata_json, error_message, created_at, updated_at, completed_at)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(&job.id)
        .bind(&job.file_path)
        .bind(&job.number)
        .bind(&job.status)
        .bind(&job.metadata_json)
        .bind(&job.error_message)
        .bind(&job.created_at)
        .bind(&job.updated_at)
        .bind(&job.completed_at)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Get job by ID
    pub async fn get_job(&self, id: &str) -> Result<Option<ProcessingJob>> {
        let job = sqlx::query_as::<_, ProcessingJob>("SELECT * FROM processing_jobs WHERE id = ?")
            .bind(id)
            .fetch_optional(&self.pool)
            .await?;

        Ok(job)
    }

    /// Get job by file path
    pub async fn get_job_by_path(&self, file_path: &str) -> Result<Option<ProcessingJob>> {
        let job =
            sqlx::query_as::<_, ProcessingJob>("SELECT * FROM processing_jobs WHERE file_path = ?")
                .bind(file_path)
                .fetch_optional(&self.pool)
                .await?;

        Ok(job)
    }

    /// Update job status
    pub async fn update_job_status(
        &self,
        id: &str,
        status: JobStatus,
        error_message: Option<String>,
    ) -> Result<()> {
        let status_str = status.to_string();
        let completed_at = if matches!(status, JobStatus::Completed | JobStatus::Failed) {
            Some(chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string())
        } else {
            None
        };

        sqlx::query(
            r#"
            UPDATE processing_jobs
            SET status = ?, error_message = ?, completed_at = ?
            WHERE id = ?
            "#,
        )
        .bind(status_str)
        .bind(error_message)
        .bind(completed_at)
        .bind(id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Update job metadata
    pub async fn update_job_metadata(&self, id: &str, metadata_json: String) -> Result<()> {
        sqlx::query("UPDATE processing_jobs SET metadata_json = ? WHERE id = ?")
            .bind(metadata_json)
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    /// List jobs with optional status filter
    pub async fn list_jobs(&self, status: Option<JobStatus>) -> Result<Vec<ProcessingJob>> {
        let jobs = if let Some(status) = status {
            sqlx::query_as::<_, ProcessingJob>(
                "SELECT * FROM processing_jobs WHERE status = ? ORDER BY created_at DESC",
            )
            .bind(status.to_string())
            .fetch_all(&self.pool)
            .await?
        } else {
            sqlx::query_as::<_, ProcessingJob>(
                "SELECT * FROM processing_jobs ORDER BY created_at DESC",
            )
            .fetch_all(&self.pool)
            .await?
        };

        Ok(jobs)
    }

    /// Get all incomplete jobs (pending or processing)
    pub async fn get_incomplete_jobs(&self) -> Result<Vec<ProcessingJob>> {
        let jobs = sqlx::query_as::<_, ProcessingJob>(
            "SELECT * FROM processing_jobs WHERE status IN ('pending', 'processing') ORDER BY created_at ASC"
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(jobs)
    }

    /// Delete a job
    pub async fn delete_job(&self, id: &str) -> Result<()> {
        sqlx::query("DELETE FROM processing_jobs WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    /// Get job statistics
    pub async fn get_stats(&self) -> Result<JobStats> {
        let row = sqlx::query(
            r#"
            SELECT
                COUNT(*) as total,
                SUM(CASE WHEN status = 'pending' THEN 1 ELSE 0 END) as pending,
                SUM(CASE WHEN status = 'processing' THEN 1 ELSE 0 END) as processing,
                SUM(CASE WHEN status = 'completed' THEN 1 ELSE 0 END) as completed,
                SUM(CASE WHEN status = 'failed' THEN 1 ELSE 0 END) as failed
            FROM processing_jobs
            "#,
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(JobStats {
            total: row.get("total"),
            pending: row.get("pending"),
            processing: row.get("processing"),
            completed: row.get("completed"),
            failed: row.get("failed"),
        })
    }

    /// Add a failed file
    pub async fn add_failed_file(&self, failed: &FailedFile) -> Result<()> {
        sqlx::query(
            r#"
            INSERT OR REPLACE INTO failed_files (file_path, reason, failed_at)
            VALUES (?, ?, ?)
            "#,
        )
        .bind(&failed.file_path)
        .bind(&failed.reason)
        .bind(&failed.failed_at)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Check if a file is in the failed list
    pub async fn is_failed(&self, file_path: &str) -> Result<bool> {
        let row = sqlx::query("SELECT COUNT(*) as count FROM failed_files WHERE file_path = ?")
            .bind(file_path)
            .fetch_one(&self.pool)
            .await?;

        let count: i64 = row.get("count");
        Ok(count > 0)
    }

    /// Get all failed files
    pub async fn list_failed_files(&self) -> Result<Vec<FailedFile>> {
        let files =
            sqlx::query_as::<_, FailedFile>("SELECT * FROM failed_files ORDER BY failed_at DESC")
                .fetch_all(&self.pool)
                .await?;

        Ok(files)
    }

    /// Remove a file from failed list
    pub async fn remove_failed_file(&self, file_path: &str) -> Result<()> {
        sqlx::query("DELETE FROM failed_files WHERE file_path = ?")
            .bind(file_path)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    /// Clear all failed files
    pub async fn clear_failed_files(&self) -> Result<()> {
        sqlx::query("DELETE FROM failed_files")
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    /// Reset stuck jobs (processing jobs older than timeout)
    pub async fn reset_stuck_jobs(&self, timeout_minutes: i64) -> Result<usize> {
        let result = sqlx::query(
            r#"
            UPDATE processing_jobs
            SET status = 'pending'
            WHERE status = 'processing'
            AND datetime(updated_at) < datetime('now', '-' || ? || ' minutes')
            "#,
        )
        .bind(timeout_minutes)
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() as usize)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    async fn setup_test_repo() -> (JobRepository, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let repo = JobRepository::new(&db_path).await.unwrap();
        (repo, temp_dir)
    }

    #[tokio::test]
    async fn test_create_and_get_job() {
        let (repo, _temp) = setup_test_repo().await;

        let job = ProcessingJob::new("/test/movie.mp4".to_string(), Some("TEST-001".to_string()));
        let job_id = job.id.clone();

        repo.create_job(&job).await.unwrap();

        let retrieved = repo.get_job(&job_id).await.unwrap();
        assert!(retrieved.is_some());

        let retrieved_job = retrieved.unwrap();
        assert_eq!(retrieved_job.file_path, "/test/movie.mp4");
        assert_eq!(retrieved_job.number, Some("TEST-001".to_string()));
        assert_eq!(retrieved_job.status, "pending");
    }

    #[tokio::test]
    async fn test_update_job_status() {
        let (repo, _temp) = setup_test_repo().await;

        let job = ProcessingJob::new("/test/movie.mp4".to_string(), None);
        let job_id = job.id.clone();

        repo.create_job(&job).await.unwrap();

        repo.update_job_status(&job_id, JobStatus::Processing, None)
            .await
            .unwrap();

        let updated = repo.get_job(&job_id).await.unwrap().unwrap();
        assert_eq!(updated.status, "processing");

        repo.update_job_status(&job_id, JobStatus::Completed, None)
            .await
            .unwrap();

        let completed = repo.get_job(&job_id).await.unwrap().unwrap();
        assert_eq!(completed.status, "completed");
        assert!(completed.completed_at.is_some());
    }

    #[tokio::test]
    async fn test_list_jobs_by_status() {
        let (repo, _temp) = setup_test_repo().await;

        let job1 = ProcessingJob::new("/test/movie1.mp4".to_string(), None);
        let job2 = ProcessingJob::new("/test/movie2.mp4".to_string(), None);

        repo.create_job(&job1).await.unwrap();
        repo.create_job(&job2).await.unwrap();

        repo.update_job_status(&job1.id, JobStatus::Completed, None)
            .await
            .unwrap();

        let pending = repo.list_jobs(Some(JobStatus::Pending)).await.unwrap();
        assert_eq!(pending.len(), 1);

        let completed = repo.list_jobs(Some(JobStatus::Completed)).await.unwrap();
        assert_eq!(completed.len(), 1);

        let all = repo.list_jobs(None).await.unwrap();
        assert_eq!(all.len(), 2);
    }

    #[tokio::test]
    async fn test_get_stats() {
        let (repo, _temp) = setup_test_repo().await;

        let job1 = ProcessingJob::new("/test/movie1.mp4".to_string(), None);
        let job2 = ProcessingJob::new("/test/movie2.mp4".to_string(), None);
        let job3 = ProcessingJob::new("/test/movie3.mp4".to_string(), None);

        repo.create_job(&job1).await.unwrap();
        repo.create_job(&job2).await.unwrap();
        repo.create_job(&job3).await.unwrap();

        repo.update_job_status(&job1.id, JobStatus::Completed, None)
            .await
            .unwrap();
        repo.update_job_status(&job2.id, JobStatus::Failed, Some("Test error".to_string()))
            .await
            .unwrap();

        let stats = repo.get_stats().await.unwrap();
        assert_eq!(stats.total, 3);
        assert_eq!(stats.pending, 1);
        assert_eq!(stats.completed, 1);
        assert_eq!(stats.failed, 1);
    }

    #[tokio::test]
    async fn test_failed_files() {
        let (repo, _temp) = setup_test_repo().await;

        let failed = FailedFile::new(
            "/test/failed.mp4".to_string(),
            Some("No metadata".to_string()),
        );

        repo.add_failed_file(&failed).await.unwrap();

        let is_failed = repo.is_failed("/test/failed.mp4").await.unwrap();
        assert!(is_failed);

        let list = repo.list_failed_files().await.unwrap();
        assert_eq!(list.len(), 1);

        repo.remove_failed_file("/test/failed.mp4").await.unwrap();

        let is_failed_after = repo.is_failed("/test/failed.mp4").await.unwrap();
        assert!(!is_failed_after);
    }

    #[tokio::test]
    async fn test_get_incomplete_jobs() {
        let (repo, _temp) = setup_test_repo().await;

        let job1 = ProcessingJob::new("/test/movie1.mp4".to_string(), None);
        let job2 = ProcessingJob::new("/test/movie2.mp4".to_string(), None);
        let job3 = ProcessingJob::new("/test/movie3.mp4".to_string(), None);

        repo.create_job(&job1).await.unwrap();
        repo.create_job(&job2).await.unwrap();
        repo.create_job(&job3).await.unwrap();

        repo.update_job_status(&job1.id, JobStatus::Processing, None)
            .await
            .unwrap();
        repo.update_job_status(&job3.id, JobStatus::Completed, None)
            .await
            .unwrap();

        let incomplete = repo.get_incomplete_jobs().await.unwrap();
        assert_eq!(incomplete.len(), 2); // 1 pending + 1 processing
    }
}
