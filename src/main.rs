mod entity;
mod config;
mod crypto;
mod handlers;

use std::net::SocketAddr;

use tokio::net::TcpListener;
use sea_orm::Database;

use crate::{
    config::{ServerConfig, SharedServerState, create_state}, 
    handlers::{build_router, callback::start_token_refresh_task}
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();
    let config = ServerConfig::from_env();

    let db_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "sqlite://data/auth.db?mode=rwc".to_string());

    println!("Connecting to database...");
    let db = Database::connect(&db_url).await.expect("Failed to connect to the database.");

    println!("Running pending migrations...");
    // Migrator::up(db, steps)

    let shared_state: SharedServerState = create_state(db, config);

    tokio::spawn(start_token_refresh_task(shared_state.clone()));

    let app = build_router(&shared_state);

    let port = "3000";
    let listener = TcpListener::bind(format!("127.0.0.1:{}", port)).await
        .expect(&format!("Failed to start TCP listener on port {}", port).to_string());

    println!("Server running on port {}", port);
    axum::serve(
        listener, 
        app.into_make_service_with_connect_info::<SocketAddr>()
    ).await?;

    Ok(())
}