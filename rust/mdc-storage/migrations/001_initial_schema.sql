-- Initial schema for Movie Data Capture storage
-- Processing jobs table for tracking scraping/processing status

CREATE TABLE IF NOT EXISTS processing_jobs (
    id TEXT PRIMARY KEY NOT NULL,
    file_path TEXT NOT NULL UNIQUE,
    number TEXT,
    status TEXT NOT NULL CHECK(status IN ('pending', 'processing', 'completed', 'failed')),
    metadata_json TEXT,
    error_message TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    completed_at TEXT
);

-- Index for querying by status
CREATE INDEX IF NOT EXISTS idx_jobs_status ON processing_jobs(status);

-- Index for querying by file path
CREATE INDEX IF NOT EXISTS idx_jobs_file_path ON processing_jobs(file_path);

-- Failed files tracking
CREATE TABLE IF NOT EXISTS failed_files (
    file_path TEXT PRIMARY KEY NOT NULL,
    reason TEXT,
    failed_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Index for querying by failed timestamp
CREATE INDEX IF NOT EXISTS idx_failed_at ON failed_files(failed_at);

-- Trigger to auto-update updated_at timestamp
CREATE TRIGGER IF NOT EXISTS update_job_timestamp
AFTER UPDATE ON processing_jobs
FOR EACH ROW
BEGIN
    UPDATE processing_jobs SET updated_at = datetime('now')
    WHERE id = NEW.id;
END;
