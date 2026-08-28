use axum::{
    extract::{Path, State, Json},
    http::HeaderMap, 
    response::IntoResponse
};
use reqwest::StatusCode;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use serde::Serialize;

use crate::{
    config::SharedServerState, 
    entity::users
};

#[derive(Serialize)]
struct PlayerResponse {
    pub user_id: String,
    pub minecraft_id: String,
    pub discord_id: String
}

pub async fn player_handler(
    State(state): State<SharedServerState>,
    headers: HeaderMap,
    Path(minecraft_uuid): Path<String>
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let auth_header = headers.get("Authorization").and_then(|h| h.to_str().ok());
    let expected_auth = format!("Bearer {}", state.config.minecraft_webhook_secret);

    if auth_header != Some(&expected_auth) {
        return Err((StatusCode::UNAUTHORIZED, "Invalid authentication string".into()));
    }

    let linked_user = match users::Entity::find()
        .filter(users::Column::MinecraftUuid.eq(minecraft_uuid))
        .require_one(&state.db)
        .await {
            Ok(user) => user,
            Err(e) => {
                eprintln!("Lookup failed. User doesn't exist or there's a duplicate. {}", e);
                return Err((StatusCode::NOT_FOUND, "User doesn't exist.".into()));
            }
        };
    
    let response = PlayerResponse{
        user_id: linked_user.id.to_string(),
        minecraft_id: linked_user.minecraft_uuid.expect("Minecraft player UUID should be set"),
        discord_id: linked_user.discord_id
    };

    Ok(Json(response))
}