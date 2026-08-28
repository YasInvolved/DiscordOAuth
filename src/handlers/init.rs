use std::net::SocketAddr;

use axum::{
    extract::{ConnectInfo, Json, State},
    http::{StatusCode, HeaderMap}
};

use chrono::{Duration, Utc};
use sea_orm::{ActiveModelTrait, Set};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{SharedServerState, entity::oauth_states, handlers::utils::log_endpoint};

#[derive(Deserialize)]
pub struct InitRequest {
    pub minecraft_uuid: String,
    pub challenge_token: String
}

#[derive(Serialize)]
pub struct InitResponse {
    pub auth_url: String
}

pub async fn init_auth_handler(
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    State(state): State<SharedServerState>,
    Json(payload): Json<InitRequest>
) -> Result<Json<InitResponse>, StatusCode> {
    log_endpoint("POST", "/init", addr, headers);

    let state_id = Uuid::new_v4();
    let expires_at = Utc::now() + Duration::minutes(15);

    let new_state = oauth_states::ActiveModel {
        state_id: Set(state_id.clone()),
        minecraft_uuid: Set(payload.minecraft_uuid),
        expires_at: Set(expires_at),
        challenge_token: Set(payload.challenge_token)
    };

    new_state.insert(&state.db).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let auth_url = format!(
        "https://discord.com/oauth2/authorize?client_id={}&response_type=code&redirect_uri={}&scope=identify+guilds&state={}",
        urlencoding::encode(&state.config.discord.client_id),
        urlencoding::encode(&state.config.discord.redirect_uri),
        urlencoding::encode(&state_id.to_string())
    );

    Ok(Json(InitResponse { auth_url }))
}