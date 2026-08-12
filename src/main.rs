mod entity;

use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::{Html, IntoResponse},
    routing::get,
    Router
};

use std::fs;
use std::path::Path;

use entity::Entity as VerifiedUsers;
use entity::{ActiveModel};
use sea_orm::{ConnectionTrait, Database, DatabaseConnection, EntityTrait, Schema, Set};
use serde::Deserialize;
use std::sync::Arc;

#[derive(Deserialize)]
struct OAuthQuery {
    code: Option<String>,
    state: Option<String>
}

#[derive(Deserialize)]
struct DiscordTokenResponse {
    access_token: String
}

#[derive(Deserialize)]
struct DiscordUser {
    id: String
}

#[derive(Clone)]
pub struct AppConfig {
    pub client_id: String,
    pub client_secret: String,
    pub server_address: String,
    pub redirect_url: String
}

pub struct AppState {
    pub db: DatabaseConnection,
    pub config: AppConfig
}

async fn login_handler(
    State(state): State<Arc<AppState>>,
    Query(query): Query<OAuthQuery>,
) -> impl IntoResponse {
    let mc_uuid = match query.state {
        Some(uuid) => uuid,
        None => return (StatusCode::BAD_REQUEST, "Missing 'state' (Minecraft UUID)").into_response()
    };

    let discord_url = format!(
        "https://discord.com/oauth2/authorize?client_id={}&response_type=code&redirect_uri={}&scope=identify&state={}",
        state.config.client_id,
        urlencoding::encode(&state.config.redirect_url),
        mc_uuid
    );

    axum::response::Redirect::temporary(&discord_url).into_response()
}

async fn callback_handler(
    State(state): State<Arc<AppState>>,
    Query(query): Query<OAuthQuery>,
) -> impl IntoResponse {
    let code = match query.code {
        Some(c) => c,
        None => return (StatusCode::BAD_REQUEST, "Missing code").into_response()
    };

    let mc_uuid = match query.state {
        Some(u) => u,
        None => return (StatusCode::BAD_REQUEST, "Missing state (MC UUID)").into_response()
    };

    let client = reqwest::Client::new();

    let token_params = [
        ("client_id", state.config.client_id.as_str()),
        ("client_secret", state.config.client_secret.as_str()),
        ("grant_type", "authorization_code"),
        ("code", &code),
        ("redirect_uri", state.config.redirect_url.as_str())
    ];

    let token_res = client
        .post("https://discord.com/api/v10/oauth2/token")
        .form(&token_params)
        .send()
        .await;

    let token_data: DiscordTokenResponse = match token_res {
        Ok(res) => match res.json().await {
            Ok(json) => json,
            Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, "Failed to parse token from Discord").into_response(),
        },
        Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, "Failed to fetch token from Discord").into_response()
    };

    let user_res = client
        .get("https://discord.com/api/v10/users/@me")
        .bearer_auth(token_data.access_token)
        .send()
        .await;

    let user_data: DiscordUser = match user_res {
        Ok(res) => match res.json().await {
            Ok(json) => json,
            Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, "Failed to parse user data").into_response()
        },
        Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, "Failed to fetch user from Discord").into_response(),
    };

    let user_entry = ActiveModel {
        uuid: Set(mc_uuid),
        discord_id: Set(user_data.id)
    };

    let save_result = VerifiedUsers::insert(user_entry)
        .on_conflict(sea_orm::sea_query::OnConflict::column(entity::Column::Uuid)
            .update_column(entity::Column::DiscordId)
            .to_owned(),
        )
        .exec(&state.db)
        .await;

    match save_result {
        Ok(_) => Html("<h1>Verification Successful</h1>").into_response(),
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Failed to save to database").into_response(),
    }
}

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    let db_path = std::env::var("DATABASE_PATH").unwrap_or_else(|_| "data/auth.db".to_string());
    if let Some(parent) = Path::new(&db_path).parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent).expect("Failed to create database directory");
        }
    }

    let server_address = std::env::var("SERVER_ADDRESS").expect("SERVER_ADDRESS must be set");
    let server_full_addr = format!("{}:3000", server_address);
    let config = AppConfig {
        client_id: std::env::var("DISCORD_CLIENT_ID").expect("DISCORD_CLIENT_ID must be set"),
        client_secret: std::env::var("DISCORD_CLIENT_SECRET").expect("DISCORD_CLIENT_SECRET must be set"),
        server_address: server_address,
        redirect_url: std::env::var("REDIRECT_URL").expect("REDIRECT_URL must be set")
    };

    let db_url = format!("sqlite://{}?mode=rwc&journal_mode=wal", db_path);
    let db: DatabaseConnection = Database::connect(db_url)
        .await.expect("Failed to connect to SQLite database");

    let builder = db.get_database_backend();
    let schema = Schema::new(builder);
    let create_table_stmt = schema.create_table_from_entity(VerifiedUsers);

    db.execute(builder.build(&create_table_stmt)).await.ok();

    let shared_state = Arc::new(AppState { db, config });
    let app = Router::new()
        .route("/login", get(login_handler))
        .route("/callback", get(callback_handler))
        .with_state(shared_state.clone());
    
    let listener = tokio::net::TcpListener::bind(server_full_addr).await.unwrap();
    println!("Server listening on http://{}:3000", shared_state.config.server_address);
    axum::serve(listener, app).await.unwrap();
}