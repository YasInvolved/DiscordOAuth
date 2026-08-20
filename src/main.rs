mod entity;
mod state;
mod routes;
mod db;
mod crypto;

use state::{ServerConfig, ServerState};

use std::fs;
use std::path::Path;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    let db_path = std::env::var("DATABASE_PATH").unwrap_or_else(|_| "data/auth.db".to_string());
    if let Some(parent) = Path::new(&db_path).parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent).expect("Failed to create database directory");
        }
    }

    let config = ServerConfig::initialize();
    let server_full_addr = format!("{}:3000", config.server_address);

    let server_state = ServerState::new(config).await.expect("Failed to initialize shared server state");
    let app = routes::build_router(server_state.clone());
    
    let listener = tokio::net::TcpListener::bind(server_full_addr).await.unwrap();
    println!("Server listening on http://{}:3000", server_state.config.server_address);
    axum::serve(listener, app).await.unwrap();
}