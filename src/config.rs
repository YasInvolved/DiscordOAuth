use sea_orm::DatabaseConnection;
use std::sync::Arc;
use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct ServerConfig {
    pub database: DatabaseConfig,
    pub discord: DiscordConfig,
    pub minecraft: MinecraftConfig
}

#[derive(Debug, Deserialize, Clone)]
pub struct DatabaseConfig {
    pub url: String
}

#[derive(Debug, Deserialize, Clone)]
pub struct DiscordConfig {
    pub client_id: String,
    pub client_secret: String,
    pub redirect_uri: String,
    pub aes_key: String,
    pub required_guild_id: Option<String>
}

#[derive(Debug, Deserialize, Clone)]
pub struct MinecraftConfig {
    pub webhook_url: String,
    pub webhook_secret: String
}

impl ServerConfig {
    pub fn load() -> Result<Self, config::ConfigError> {
        let config = config::Config::builder()
            .add_source(config::File::with_name("config.toml").required(false))
            .add_source(config::Environment::with_prefix("OAUTH").separator("__"))
            .build()?;

        config.try_deserialize()
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