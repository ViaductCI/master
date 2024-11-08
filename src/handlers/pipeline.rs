use actix_web::{web, HttpResponse, Responder};
use serde_yaml;
use serde_json::json;
use uuid::Uuid;

use crate::models::pipeline::{Job, Pipeline, PipelineStatus};
use crate::models::target::BuildRequest;
use crate::models::job::{JobStatus, JobResult};
use crate::db::operations::{
    create_pipeline_run, create_job_run, 
    update_job_status, get_pipeline_status
};
use crate::utils::file;

pub async fn trigger_build(build_request: web::Json<BuildRequest>) -> impl Responder {
    // Clone repository and get pipeline configuration
    let (temp_dir, config_content) = match file::clone_repository(
        &build_request.repository,
        &build_request.branch
    ).await {
        Ok(result) => result,
        Err(e) => return HttpResponse::BadRequest()
            .body(format!("Failed to fetch pipeline configuration: {}", e)),
    };

    // Parse pipeline configuration
    let pipeline: Pipeline = match serde_yaml::from_str(&config_content) {
        Ok(p) => p,
        Err(e) => return HttpResponse::BadRequest()
            .body(format!("Invalid pipeline configuration: {}", e)),
    };

    // Calculate total number of jobs
    let total_jobs = pipeline.stages.iter()
        .map(|stage| stage.jobs.len() as i32)
        .sum();

    // Create pipeline run
    let pipeline_run_id = match create_pipeline_run(
        &pipeline.name,
        &build_request.repository,
        &build_request.branch,
        total_jobs
    ) {
        Ok(id) => id,
        Err(e) => return HttpResponse::InternalServerError()
            .body(format!("Failed to create pipeline run: {}", e)),
    };

    // Create job runs
    let mut job_index = 0;
    for stage in &pipeline.stages {
        for job in &stage.jobs {
            if let Err(e) = create_job_run(&pipeline_run_id, &job.name, job_index) {
                eprintln!("Failed to create job run: {}", e);
            }
            job_index += 1;
        }
    }

    // Start pipeline execution
    tokio::spawn(execute_pipeline(
        pipeline_run_id.clone(),
        pipeline,
        build_request.0,
    ));

    HttpResponse::Ok().json(json!({
        "pipeline_run_id": pipeline_run_id,
        "status": "started"
    }))
}

pub async fn get_status(pipeline_name: web::Path<String>) -> impl Responder {
    match get_pipeline_status(&pipeline_name) {
        Ok(Some((pipeline_run, jobs))) => HttpResponse::Ok().json(json!({
            "pipeline": pipeline_run,
            "jobs": jobs
        })),
        Ok(None) => HttpResponse::NotFound()
            .body(format!("No pipeline runs found for '{}'", pipeline_name)),
        Err(e) => HttpResponse::InternalServerError()
            .body(format!("Database error: {}", e)),
    }
}

async fn execute_pipeline(
    pipeline_run_id: String,
    pipeline: Pipeline,
    build_request: BuildRequest,
) {
    let worker_url = std::env::var("WORKER_URL")
        .unwrap_or_else(|_| "http://localhost:8080".to_string());
    let client = reqwest::Client::new();

    for stage in pipeline.stages {
        for job in stage.jobs {
            let job_result = execute_job(&client, &worker_url, &job).await;
            
            // Update job status in database
            if let Err(e) = update_job_status(
                &job_result.id,
                job_result.status.clone(),
                Some(&job_result.output)
            ) {
                eprintln!("Failed to update job status: {}", e);
            }

            // Break pipeline execution if job failed
            if job_result.status == JobStatus::Failed {
                break;
            }
        }
    }
}

async fn execute_job(
    client: &reqwest::Client,
    worker_url: &str,
    job: &Job,
) -> JobResult {
    let job_id = Uuid::new_v4().to_string();

    match client.post(&format!("{}/job", worker_url))
        .json(&job)
        .send()
        .await {
        Ok(response) => {
            match response.json::<JobResult>().await {
                Ok(mut result) => {
                    result.id = job_id;
                    result
                },
                Err(e) => JobResult {
                    id: job_id,
                    status: JobStatus::Failed,
                    output: format!("Failed to parse worker response: {}", e),
                    artifacts: vec![],
                },
            }
        }
        Err(e) => JobResult {
            id: job_id,
            status: JobStatus::Failed,
            output: format!("Failed to communicate with worker: {}", e),
            artifacts: vec![],
        },
    }
}