use rusqlite::{Connection, Result as SqlResult};
use std::path::Path;

pub const DATABASE_FILE: &str = "pipeline.db";

pub fn init_database() -> SqlResult<()> {
    let conn = Connection::open(DATABASE_FILE)?;
    
    // Create pipeline_runs table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS pipeline_runs (
            id TEXT PRIMARY KEY,
            pipeline_name TEXT NOT NULL,
            repository TEXT NOT NULL,
            branch TEXT NOT NULL,
            status TEXT NOT NULL,
            start_time DATETIME NOT NULL,
            end_time DATETIME,
            duration_seconds INTEGER,
            current_job_index INTEGER DEFAULT 0,
            total_jobs INTEGER NOT NULL
        )",
        [],
    )?;

    // Create job_runs table with job_index for ordering
    conn.execute(
        "CREATE TABLE IF NOT EXISTS job_runs (
            id TEXT PRIMARY KEY,
            pipeline_run_id TEXT NOT NULL,
            job_name TEXT NOT NULL,
            job_index INTEGER NOT NULL,
            status TEXT NOT NULL,
            start_time DATETIME NOT NULL,
            end_time DATETIME,
            duration_seconds INTEGER,
            output TEXT,
            FOREIGN KEY(pipeline_run_id) REFERENCES pipeline_runs(id)
        )",
        [],
    )?;

    // Create artifacts table for job outputs
    conn.execute(
        "CREATE TABLE IF NOT EXISTS job_artifacts (
            id TEXT PRIMARY KEY,
            job_run_id TEXT NOT NULL,
            name TEXT NOT NULL,
            content TEXT NOT NULL,
            created_at DATETIME NOT NULL,
            FOREIGN KEY(job_run_id) REFERENCES job_runs(id)
        )",
        [],
    )?;

    // Indexes for better performance
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_job_runs_pipeline_run_id 
         ON job_runs(pipeline_run_id)",
        [],
    )?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_job_runs_status 
         ON job_runs(status)",
        [],
    )?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_pipeline_runs_name 
         ON pipeline_runs(pipeline_name)",
        [],
    )?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_job_artifacts_job_run 
         ON job_artifacts(job_run_id)",
        [],
    )?;

    Ok(())
}