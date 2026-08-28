use sea_orm::DatabaseConnection;
use std::sync::Arc;

#[derive(Clone)]
pub struct ServerConfig {
    pub discord_client_id: String,
    pub discord_client_secret: String,
    pub discord_redirect_url: String,
    pub aes_key: String,
    pub minecraft_webhook_url: String,
    pub minecraft_webhook_secret: String
}

impl ServerConfig {
    pub fn from_env() -> Self {
        Self {
            discord_client_id: std::env::var("DISCORD_CLIENT_ID").expect("DISCORD_CLIENT_ID must be set"),
            discord_client_secret: std::env::var("DISCORD_CLIENT_SECRET").expect("DISCORD_CLIENT_SECRET must be set"),
            discord_redirect_url: std::env::var("DISCORD_REDIRECT_URL").expect("DISCORD_REDIRECT_URL must be set"),
            aes_key: std::env::var("AES_KEY").expect("AES_KEY must be set"),
            minecraft_webhook_url: std::env::var("MINECRAFT_WEBHOOK_URL").expect("MINECRAFT_WEBHOOK_URL must be set"),
            minecraft_webhook_secret: std::env::var("MINECRAFT_WEBHOOK_SECRET").expect("MINECRAFT_WEBHOOK_SECRET must be set")
        }
    }
}

pub struct ServerState {
    pub db: DatabaseConnection,
    pub http_client: reqwest::Client,
    pub config: ServerConfig
}

pub type SharedServerState = Arc<ServerState>;

pub fn create_state(db: DatabaseConnection, config: ServerConfig) -> SharedServerState {
    let http_client = reqwest::Client::new();
    Arc::new(ServerState { db, http_client, config })
}