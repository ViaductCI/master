use actix_web::{web, HttpResponse, Responder};
use serde_json::json;
use chrono::{DateTime, Utc};
use rusqlite::{Connection, params};

use crate::models::job::{JobStatus, JobResult, JobArtifact};
use crate::db::operations::{
    update_job_status,
    save_job_artifact,
};

// Helper functions for datetime conversion (matching db/operations.rs)
fn datetime_to_sqlite(dt: DateTime<Utc>) -> String {
    dt.format("%Y-%m-%d %H:%M:%S").to_string()
}

fn sqlite_to_datetime(s: String) -> Result<DateTime<Utc>, chrono::ParseError> {
    DateTime::parse_from_str(&format!("{} +0000", s), "%Y-%m-%d %H:%M:%S %z")
        .map(|dt| dt.with_timezone(&Utc))
}

#[derive(serde::Deserialize)]
pub struct JobUpdate {
    pub status: JobStatus,
    pub output: Option<String>,
    pub artifacts: Option<Vec<JobArtifact>>,
}

#[derive(serde::Serialize)]
pub struct JobDetails {
    pub id: String,
    pub pipeline_run_id: String,
    pub name: String,
    pub index: i32,
    pub status: JobStatus,
    pub start_time: DateTime<Utc>,
    pub end_time: Option<DateTime<Utc>>,
    pub duration_seconds: Option<i64>,
    pub output: String,
    pub artifacts: Vec<JobArtifact>,
}

// Update job status and optionally add artifacts
pub async fn update_job(
    job_id: web::Path<String>,
    update: web::Json<JobUpdate>,
) -> impl Responder {
    // Update job status
    if let Err(e) = update_job_status(
        &job_id,
        update.status.clone(),
        update.output.as_deref(),
    ) {
        return HttpResponse::InternalServerError()
            .json(json!({
                "error": format!("Failed to update job status: {}", e)
            }));
    }

    // If artifacts are provided, save them
    if let Some(artifacts) = &update.artifacts {
        for artifact in artifacts {
            if let Err(e) = save_job_artifact(
                &job_id,
                &artifact.name,
                &artifact.content,
            ) {
                eprintln!("Failed to save artifact {}: {}", artifact.name, e);
            }
        }
    }

    HttpResponse::Ok()
        .json(json!({
            "status": "updated",
            "job_id": job_id.to_string()
        }))
}

// Get job details including artifacts
pub async fn get_job_details(job_id: web::Path<String>) -> impl Responder {
    match get_job_with_artifacts(&job_id) {
        Ok(job_details) => HttpResponse::Ok().json(job_details),
        Err(e) => HttpResponse::InternalServerError()
            .json(json!({
                "error": format!("Failed to get job details: {}", e)
            }))
    }
}

// Get logs for a specific job
pub async fn get_job_logs(job_id: web::Path<String>) -> impl Responder {
    match get_job_output(&job_id) {
        Ok(Some(output)) => HttpResponse::Ok().body(output),
        Ok(None) => HttpResponse::NotFound()
            .json(json!({
                "error": "No logs found for this job"
            })),
        Err(e) => HttpResponse::InternalServerError()
            .json(json!({
                "error": format!("Failed to get job logs: {}", e)
            }))
    }
}

// Internal helper function to get job with its artifacts
fn get_job_with_artifacts(job_id: &str) -> Result<JobDetails, rusqlite::Error> {
    use crate::db::init::DATABASE_FILE;
    let conn = Connection::open(DATABASE_FILE)?;

    // Get job information
    let job = conn.query_row(
        "SELECT id, pipeline_run_id, job_name, job_index, status,
                start_time, end_time, duration_seconds, output
         FROM job_runs 
         WHERE id = ?1",
        params![job_id],
        |row| {
            let start_time: String = row.get(5)?;
            let end_time: Option<String> = row.get(6)?;
            let status: String = row.get(4)?;

            Ok(JobDetails {
                id: row.get(0)?,
                pipeline_run_id: row.get(1)?,
                name: row.get(2)?,
                index: row.get(3)?,
                status: serde_json::from_str(&status).unwrap(),
                start_time: sqlite_to_datetime(start_time).unwrap(),
                end_time: end_time.map(|t| sqlite_to_datetime(t).unwrap()),
                duration_seconds: row.get(7)?,
                output: row.get(8)?,
                artifacts: Vec::new(), // Will be populated below
            })
        }
    )?;

    // Get artifacts for this job
    let mut stmt = conn.prepare(
        "SELECT name, content
         FROM job_artifacts 
         WHERE job_run_id = ?1 
         ORDER BY created_at"
    )?;

    let artifacts = stmt.query_map(params![job_id], |row| {
        Ok(JobArtifact {
            name: row.get(0)?,
            content: row.get(1)?,
        })
    })?
    .collect::<Result<Vec<_>, _>>()?;

    Ok(JobDetails { artifacts, ..job })
}

// Internal helper function to get job output
fn get_job_output(job_id: &str) -> Result<Option<String>, rusqlite::Error> {
    use crate::db::init::DATABASE_FILE;
    let conn = Connection::open(DATABASE_FILE)?;
    
    conn.query_row(
        "SELECT output FROM job_runs WHERE id = ?1",
        params![job_id],
        |row| row.get(0)
    )
}