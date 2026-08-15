use std::{env, eprintln, println};
use std::sync::Arc;
use reqwest::Client;
use sea_orm::{Database, DatabaseConnection, DbErr};
use migration::{Migrator, MigratorTrait};

#[derive(Clone)]
pub struct ServerConfig {
    pub client_id: String, 
    pub client_secret: String,
    pub server_address: String,
    pub db_path: String,
    pub redirect_url: String,
    pub minecraft_webhook_url: String
}

impl ServerConfig {
    pub fn initialize() -> Self {
        Self {
            client_id: env::var("DISCORD_CLIENT_ID").expect("DISCORD_CLIENT_ID must be set"),
            client_secret: env::var("DISCORD_CLIENT_SECRET").expect("DISCORD_CLIENT_SECRET must be set"),
            server_address: env::var("SERVER_ADDRESS").expect("SERVER_ADDRESS must be set"),
            db_path: env::var("DATABASE_PATH").unwrap_or_else(|_| "data/auth.db".to_string()),
            redirect_url: env::var("REDIRECT_URL").expect("REDIRECT_URL must be set"),
            minecraft_webhook_url: env::var("MINECRAFT_WEBHOOK_URL").expect("MINECRAFT_WEBHOOK_URL MUST BE SET")
        }
    }
}

pub struct ServerState {
    pub config: ServerConfig,
    pub client: Client,
    pub db: DatabaseConnection,
}

impl ServerState {
    pub async fn new(config: ServerConfig) -> Result<Arc<Self>, DbErr> {
        let db_url = format!("sqlite://{}?mode=rwc", config.db_path);
        let db = Database::connect(&db_url).await?;

        Migrator::up(&db, None).await.map_err(|e| DbErr::Custom(e.to_string()))?;

        let state = Self {
            config,
            client: Client::new(),
            db,
        };

        Ok(Arc::new(state))
    }

    pub async fn notify_minecraft(&self, player_uuid: &str) {
        let url = &self.config.minecraft_webhook_url;

        let res = self.client
            .post(url)
            .body(player_uuid.to_string())
            .send()
            .await;

        match res {
            Ok(resp) if resp.status().is_success() => {
                println!("Successfully notified Minecraft server for player {}", player_uuid);
            },
            Ok(resp) => {
                eprintln!("Minecraft server returned error status: {}", resp.status());
            },
            Err(err) => {
                eprintln!("Failed to reach Minecraft webhook at {}: {}", url, err);
            }
        }
    }
}