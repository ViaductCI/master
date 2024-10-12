use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use serde::{Deserialize, Serialize};
use serde_yaml;
use uuid::Uuid;
use std::collections::HashMap;
use reqwest;
use std::fs;

#[derive(Debug, Serialize, Deserialize)]
struct Pipeline {
    name: String,
    stages: Vec<Stage>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Stage {
    name: String,
    jobs: Vec<Job>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Job {
    name: String,
    repository: String,
    branch: String,
    commands: Vec<String>,
    #[serde(default)]
    inputs: Vec<JobInput>,
    #[serde(default)]
    outputs: Vec<JobOutput>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct JobInput {
    name: String,
    value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct JobOutput {
    name: String,
    path: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct JobResult {
    id: String,
    status: String,
    output: String,
    artifacts: Vec<JobArtifact>,
}

#[derive(Debug, Serialize, Deserialize)]
struct JobArtifact {
    name: String,
    content: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct BuildRequest {
    repository: String,
    branch: String,
}

async fn trigger_build(build_request: web::Json<BuildRequest>) -> impl Responder {
    let pipeline_config = fetch_pipeline_config(&build_request).await;
    
    match pipeline_config {
        Ok(config) => {
            match serde_yaml::from_str::<Pipeline>(&config) {
                Ok(pipeline) => {
                    let result = execute_pipeline(&pipeline).await;
                    HttpResponse::Ok().json(result)
                }
                Err(e) => HttpResponse::BadRequest().body(format!("Invalid pipeline configuration: {}", e)),
            }
        }
        Err(e) => HttpResponse::BadRequest().body(format!("Failed to fetch pipeline configuration: {}", e)),
    }
}

async fn fetch_pipeline_config(build_request: &BuildRequest) -> Result<String, Box<dyn std::error::Error>> {
    let temp_dir = format!("temp_{}", Uuid::new_v4());
    fs::create_dir(&temp_dir)?;

    let clone_result = std::process::Command::new("git")
        .args(&["clone", "-b", &build_request.branch, &build_request.repository, &temp_dir])
        .output()?;

    if !clone_result.status.success() {
        return Err(format!("Failed to clone repository: {}", String::from_utf8_lossy(&clone_result.stderr)).into());
    }

    let config_path = format!("{}/.pipeline.yml", temp_dir);
    let config_content = fs::read_to_string(config_path)?;

    fs::remove_dir_all(temp_dir)?;

    Ok(config_content)
}

async fn execute_pipeline(pipeline: &Pipeline) -> HashMap<String, Vec<JobResult>> {
    let mut results = HashMap::new();

    for stage in &pipeline.stages {
        let mut stage_results = Vec::new();

        for job in &stage.jobs {
            let job_result = execute_job(job).await;
            stage_results.push(job_result);
        }

        results.insert(stage.name.clone(), stage_results);
    }

    results
}

async fn execute_job(job: &Job) -> JobResult {
    let worker_url = "http://localhost:8080/job";  // Assumes worker is accessible at this URL
    let client = reqwest::Client::new();

    let job_with_id = Job {
        name: job.name.clone(),
        repository: job.repository.clone(),
        branch: job.branch.clone(),
        commands: job.commands.clone(),
        inputs: job.inputs.clone(),
        outputs: job.outputs.clone(),
    };

    match client.post(worker_url)
        .json(&job_with_id)
        .send()
        .await {
        Ok(response) => {
            match response.json::<JobResult>().await {
                Ok(result) => result,
                Err(e) => JobResult {
                    id: Uuid::new_v4().to_string(),
                    status: "failed".to_string(),
                    output: format!("Failed to parse worker response: {}", e),
                    artifacts: vec![],
                },
            }
        }
        Err(e) => JobResult {
            id: Uuid::new_v4().to_string(),
            status: "failed".to_string(),
            output: format!("Failed to communicate with worker: {}", e),
            artifacts: vec![],
        },
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .route("/trigger", web::post().to(trigger_build))
    })
    .bind("0.0.0.0:8000")?
    .run()
    .await
}