use axum::{
    extract::{Json, State},
    http::StatusCode
};

use chrono::{Duration, Utc};
use sea_orm::{ActiveModelTrait, Set};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{SharedServerState, entity::oauth_states};

#[derive(Deserialize)]
pub struct InitRequest {
    pub minecraft_uuid: String
}

#[derive(Serialize)]
pub struct InitResponse {
    pub auth_url: String
}

pub async fn init_auth_handler(
    State(state): State<SharedServerState>,
    Json(payload): Json<InitRequest>
) -> Result<Json<InitResponse>, StatusCode> {
    let state_id = Uuid::new_v4();
    let expires_at = Utc::now() + Duration::minutes(15);

    let new_state = oauth_states::ActiveModel {
        state_id: Set(state_id.clone()),
        minecraft_uuid: Set(payload.minecraft_uuid),
        expires_at: Set(expires_at)
    };

    new_state.insert(&state.db).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let auth_url = format!(
        "https://discord.com/oauth2/authorize?client_id={}&response_type=code&redirect_uri={}&scope=identify&state={}",
        urlencoding::encode(&state.config.discord_client_id),
        urlencoding::encode(&state.config.discord_redirect_url),
        urlencoding::encode(&state_id.to_string())
    );

    Ok(Json(InitResponse { auth_url }))
}