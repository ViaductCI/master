use actix_web::{web, HttpResponse, Responder};
use std::sync::Mutex;
use serde_json;

use crate::models::target::{Target, Targets, AddTargetRequest};
use crate::utils::file;

pub async fn add_target(
    target_request: web::Json<AddTargetRequest>,
    data: web::Data<Mutex<()>>,
) -> impl Responder {
    let _lock = data.lock().unwrap();
    
    // Load existing targets
    let mut targets = match file::read_file("targets.json") {
        Ok(content) => match serde_json::from_str::<Targets>(&content) {
            Ok(t) => t,
            Err(e) => return HttpResponse::InternalServerError()
                .body(format!("Failed to parse targets file: {}", e)),
        },
        Err(e) => return HttpResponse::InternalServerError()
            .body(format!("Failed to read targets file: {}", e)),
    };
    
    // Generate target name if not provided
    let target_name = target_request.name.clone()
        .unwrap_or_else(|| file::extract_repo_name(&target_request.repository));
    
    // Check if target name already exists
    if targets.targets.iter().any(|t| t.name == target_name) {
        return HttpResponse::BadRequest()
            .body(format!("Target name '{}' already exists", target_name));
    }
    
    // Clone repository and get pipeline configuration
    let (temp_dir, config_content) = match file::clone_repository(
        &target_request.repository,
        &target_request.branch
    ).await {
        Ok(result) => result,
        Err(e) => return HttpResponse::BadRequest()
            .body(format!("Failed to fetch pipeline config: {}", e)),
    };
    
    // Save pipeline configuration to targets directory
    let filename = file::repo_to_filename(&target_request.repository, &target_request.branch);
    let path = format!("targets/{}", filename);
    if let Err(e) = file::save_file(&path, &config_content) {
        return HttpResponse::InternalServerError()
            .body(format!("Failed to save pipeline config: {}", e));
    }
    
    // Add new target
    targets.targets.push(Target {
        name: target_name,
        repository: target_request.repository.clone(),
        branch: target_request.branch.clone(),
    });
    
    // Save updated targets
    if let Err(e) = file::save_file(
        "targets.json",
        &serde_json::to_string_pretty(&targets).unwrap()
    ) {
        return HttpResponse::InternalServerError()
            .body(format!("Failed to save targets: {}", e));
    }
    
    HttpResponse::Ok().json(targets)
}

pub async fn list_targets() -> impl Responder {
    match file::read_file("targets.json") {
        Ok(content) => match serde_json::from_str::<Targets>(&content) {
            Ok(targets) => HttpResponse::Ok().json(targets),
            Err(e) => HttpResponse::InternalServerError()
                .body(format!("Failed to parse targets file: {}", e)),
        },
        Err(e) => HttpResponse::InternalServerError()
            .body(format!("Failed to read targets file: {}", e)),
    }
}

pub async fn get_target_pipeline(target_name: web::Path<String>) -> impl Responder {
    let targets = match file::read_file("targets.json") {
        Ok(content) => match serde_json::from_str::<Targets>(&content) {
            Ok(t) => t,
            Err(e) => return HttpResponse::InternalServerError()
                .body(format!("Failed to parse targets file: {}", e)),
        },
        Err(e) => return HttpResponse::InternalServerError()
            .body(format!("Failed to read targets file: {}", e)),
    };
    
    let target = match targets.targets.iter()
        .find(|t| t.name == *target_name)
    {
        Some(t) => t,
        None => return HttpResponse::NotFound()
            .body(format!("Target '{}' not found", target_name)),
    };
    
    let filename = file::repo_to_filename(&target.repository, &target.branch);
    let path = format!("targets/{}", filename);
    
    match file::read_file(&path) {
        Ok(config) => HttpResponse::Ok().body(config),
        Err(e) => HttpResponse::InternalServerError()
            .body(format!("Failed to read pipeline config: {}", e)),
    }
}