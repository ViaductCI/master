use rusqlite::{Connection, params, Result as SqlResult};
use chrono::{DateTime, Utc};
use crate::models::pipeline::{PipelineRun, PipelineStatus};
use crate::models::job::{JobRun, JobStatus, JobArtifact};
use super::init::DATABASE_FILE;
use uuid::Uuid;

pub fn create_pipeline_run(
    name: &str,
    repository: &str,
    branch: &str,
    total_jobs: i32
) -> SqlResult<String> {
    let conn = Connection::open(DATABASE_FILE)?;
    let id = Uuid::new_v4().to_string();
    
    conn.execute(
        "INSERT INTO pipeline_runs (
            id, pipeline_name, repository, branch, status, 
            start_time, total_jobs, current_job_index
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        params![
            id,
            name,
            repository,
            branch,
            PipelineStatus::Pending.to_string(),
            Utc::now(),
            total_jobs,
            0
        ],
    )?;
    
    Ok(id)
}

pub fn create_job_run(
    pipeline_run_id: &str,
    job_name: &str,
    job_index: i32
) -> SqlResult<String> {
    let conn = Connection::open(DATABASE_FILE)?;
    let id = Uuid::new_v4().to_string();
    
    conn.execute(
        "INSERT INTO job_runs (
            id, pipeline_run_id, job_name, job_index, 
            status, start_time
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![
            id,
            pipeline_run_id,
            job_name,
            job_index,
            JobStatus::Pending.to_string(),
            Utc::now()
        ],
    )?;
    
    Ok(id)
}

pub fn update_job_status(id: &str, status: JobStatus, output: Option<&str>) -> SqlResult<()> {
    let conn = Connection::open(DATABASE_FILE)?;
    let now = Utc::now();
    
    let start_time: DateTime<Utc> = conn.query_row(
        "SELECT start_time FROM job_runs WHERE id = ?1",
        params![id],
        |row| row.get(0),
    )?;

    let duration = (now - start_time).num_seconds();

    conn.execute(
        "UPDATE job_runs 
         SET status = ?1, end_time = ?2, duration_seconds = ?3, output = ?4 
         WHERE id = ?5",
        params![
            status.to_string(),
            now,
            duration,
            output,
            id
        ],
    )?;

    // Update pipeline progress if job is complete
    if status != JobStatus::Running && status != JobStatus::Pending {
        let pipeline_run_id: String = conn.query_row(
            "SELECT pipeline_run_id FROM job_runs WHERE id = ?1",
            params![id],
            |row| row.get(0),
        )?;

        update_pipeline_progress(&pipeline_run_id)?;
    }

    Ok(())
}

pub fn update_pipeline_progress(pipeline_run_id: &str) -> SqlResult<()> {
    let conn = Connection::open(DATABASE_FILE)?;
    
    // Get total and completed jobs
    let (total_jobs, completed_jobs, failed_jobs): (i32, i32, i32) = conn.query_row(
        "SELECT 
            total_jobs,
            COUNT(CASE WHEN status = 'succeeded' THEN 1 END),
            COUNT(CASE WHEN status = 'failed' THEN 1 END)
         FROM pipeline_runs pr
         LEFT JOIN job_runs jr ON pr.id = jr.pipeline_run_id
         WHERE pr.id = ?1
         GROUP BY pr.id",
        params![pipeline_run_id],
        |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
    )?;

    // Update pipeline status based on job statuses
    let new_status = if failed_jobs > 0 {
        PipelineStatus::Failed
    } else if completed_jobs == total_jobs {
        PipelineStatus::Completed
    } else {
        PipelineStatus::Running
    };

    // Update pipeline run
    if new_status == PipelineStatus::Completed || new_status == PipelineStatus::Failed {
        let now = Utc::now();
        let start_time: DateTime<Utc> = conn.query_row(
            "SELECT start_time FROM pipeline_runs WHERE id = ?1",
            params![pipeline_run_id],
            |row| row.get(0),
        )?;

        let duration = (now - start_time).num_seconds();

        conn.execute(
            "UPDATE pipeline_runs 
             SET status = ?1, end_time = ?2, duration_seconds = ?3, current_job_index = ?4
             WHERE id = ?5",
            params![
                new_status.to_string(),
                now,
                duration,
                completed_jobs,
                pipeline_run_id
            ],
        )?;
    } else {
        conn.execute(
            "UPDATE pipeline_runs 
             SET status = ?1, current_job_index = ?2
             WHERE id = ?3",
            params![
                new_status.to_string(),
                completed_jobs,
                pipeline_run_id
            ],
        )?;
    }

    Ok(())
}

pub fn save_job_artifact(
    job_run_id: &str,
    name: &str,
    content: &str
) -> SqlResult<String> {
    let conn = Connection::open(DATABASE_FILE)?;
    let id = Uuid::new_v4().to_string();
    
    conn.execute(
        "INSERT INTO job_artifacts (id, job_run_id, name, content, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        params![
            id,
            job_run_id,
            name,
            content,
            Utc::now()
        ],
    )?;
    
    Ok(id)
}

pub fn get_pipeline_status(pipeline_name: &str) -> SqlResult<Option<(PipelineRun, Vec<JobRun>)>> {
    let conn = Connection::open(DATABASE_FILE)?;
    
    // Get the latest pipeline run
    let pipeline_run = match conn.query_row(
        "SELECT id, pipeline_name, repository, branch, status, 
                start_time, end_time, duration_seconds, 
                current_job_index, total_jobs
         FROM pipeline_runs 
         WHERE pipeline_name = ?1 
         ORDER BY start_time DESC 
         LIMIT 1",
        params![pipeline_name],
        |row| {
            Ok(PipelineRun {
                id: row.get(0)?,
                pipeline_name: row.get(1)?,
                repository: row.get(2)?,
                branch: row.get(3)?,
                status: serde_json::from_str(&row.get::<_, String>(4)?).unwrap(),
                start_time: row.get(5)?,
                end_time: row.get(6)?,
                duration_seconds: row.get(7)?,
            })
        },
    ) {
        Ok(run) => run,
        Err(rusqlite::Error::QueryReturnedNoRows) => return Ok(None),
        Err(e) => return Err(e),
    };

    // Get all jobs for this pipeline run
    let mut stmt = conn.prepare(
        "SELECT id, job_name, job_index, status, start_time, 
                end_time, duration_seconds, output
         FROM job_runs 
         WHERE pipeline_run_id = ?1 
         ORDER BY job_index"
    )?;

    let jobs = stmt.query_map(params![pipeline_run.id], |row| {
        Ok(JobRun {
            id: row.get(0)?,
            pipeline_run_id: pipeline_run.id.clone(),
            job_name: row.get(1)?,
            job_index: row.get(2)?,
            status: serde_json::from_str(&row.get::<_, String>(3)?).unwrap(),
            start_time: row.get(4)?,
            end_time: row.get(5)?,
            duration_seconds: row.get(6)?,
        })
    })?
    .collect::<SqlResult<Vec<_>>>()?;

    Ok(Some((pipeline_run, jobs)))
}