use actix_web::{web, App, HttpServer};
use actix_cors::Cors;
use std::sync::Mutex;

mod models;
mod db;
mod handlers;
mod utils;

use crate::handlers::{
    pipeline::{trigger_build, get_status},
    target::{add_target, list_targets, get_target_pipeline},
};
use crate::utils::file;
use crate::db::init::init_database;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Initialize the database
    if let Err(e) = init_database() {
        eprintln!("Failed to initialize database: {}", e);
        return Ok(());
    }

    // Ensure targets directory exists
    if let Err(e) = file::ensure_directory("targets") {
        eprintln!("Failed to create targets directory: {}", e);
        return Ok(());
    }

    // Create empty targets.json if it doesn't exist
    if let Err(e) = file::read_file("targets.json") {
        if let Err(e) = file::save_file(
            "targets.json",
            &serde_json::to_string_pretty(&models::target::Targets { targets: vec![] }).unwrap()
        ) {
            eprintln!("Failed to create targets.json: {}", e);
            return Ok(());
        }
    }
    
    // Load configuration from environment
    let host = std::env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
    let port = std::env::var("PORT")
        .unwrap_or_else(|_| "8000".to_string())
        .parse::<u16>()
        .expect("Invalid PORT");
    
    let data = web::Data::new(Mutex::new(()));
    
    println!("Starting server at {}:{}", host, port);

    HttpServer::new(move || {
        // Configure CORS
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header()
            .max_age(3600);

        App::new()
            .wrap(cors)
            .app_data(data.clone())
            // Pipeline routes
            .service(
                web::scope("/api")
                    // Pipeline execution endpoints
                    .route("/trigger", web::post().to(trigger_build))
                    .route("/pipelines/{name}/status", web::get().to(get_status))
                    // Target management endpoints
                    .route("/targets", web::post().to(add_target))
                    .route("/targets", web::get().to(list_targets))
                    .route("/targets/{name}/pipeline", web::get().to(get_target_pipeline))
            )
    })
    .bind((host, port))?
    .run()
    .await
}