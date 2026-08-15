use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::get,
    Json, Router
};

use serde::{Deserialize, Serialize};
use std::{eprintln, sync::Arc};

use crate::entity::verified_users::Entity as VerifiedUser;
use crate::state::ServerState;

pub fn build_router(state: Arc<ServerState>) -> Router {
    Router::new()
        .route("/login", get(handle_login))
        .route("/callback", get(handle_callback))
        .route("/status/{uuid}", get(handle_status))
        .with_state(state)
}

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


#[derive(Serialize)]
struct StatusResponse {
    verified: bool
}

async fn handle_login(
    State(state): State<Arc<ServerState>>,
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

async fn handle_callback(
    State(state): State<Arc<ServerState>>,
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

    if let Err(e) = VerifiedUser::register_player(&state.db, &mc_uuid, &user_data.id).await {
        eprintln!("Failed to save user to database: {}", e);
        return (StatusCode::INTERNAL_SERVER_ERROR, "Failed to save user to the database").into_response();
    }

    state.notify_minecraft(&mc_uuid).await;
    "Authentication successful! You can return to Minecraft.".into_response()
}

async fn handle_status(
    State(state): State<Arc<ServerState>>,
    Path(uuid): Path<String>,
) -> Json<StatusResponse> {
    let is_verified = VerifiedUser::is_player_verified(&state.db, &uuid)
        .await
        .unwrap_or_else(|e| {
            eprintln!("Database error: {}", e);
            false
        });

    Json(StatusResponse { verified: is_verified })
}