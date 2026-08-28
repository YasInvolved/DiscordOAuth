use axum::{
    extract::{Path, State, Json},
    http::HeaderMap, 
    response::IntoResponse
};
use reqwest::StatusCode;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use serde::Serialize;

use crate::{
    config::SharedServerState, discord::{guild::DiscordGuild, oauth2::revoke_token}, entity::{oauth_tokens, users}
};

#[derive(Serialize)]
struct PlayerResponse {
    pub user_id: String,
    pub minecraft_id: String,
    pub discord_id: String
}

#[derive(Serialize)]
struct MemberResponse {
    pub is_member: bool,
    pub is_pending: bool,
    pub nickname: Option<String>
}

pub async fn player_handler(
    State(state): State<SharedServerState>,
    headers: HeaderMap,
    Path(minecraft_uuid): Path<String>
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let auth_header = headers.get("Authorization").and_then(|h| h.to_str().ok());
    let expected_auth = format!("Bearer {}", state.config.minecraft.webhook_secret);

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

pub async fn player_unlink_handler(
    State(state): State<SharedServerState>,
    headers: HeaderMap,
    Path(minecraft_uuid): Path<String>
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let auth_header = headers.get("Authorization").and_then(|h| h.to_str().ok());
    let expected_auth = format!("Bearer {}", state.config.minecraft.webhook_secret);

    if auth_header != Some(&expected_auth) {
        return Err((StatusCode::UNAUTHORIZED, "Invalid authentication string".into()));
    }

    let linked_user = match users::Entity::find()
        .filter(users::Column::MinecraftUuid.eq(minecraft_uuid))
        .find_both_related(oauth_tokens::Entity)
        .require_one(&state.db)
        .await {
            Ok(u) => u,
            Err(_) => {
                eprintln!("Lookup failed. User doesn't exist or has a duplicate");
                return Err((StatusCode::NOT_FOUND, "User doesn't exist".into()))
            }
        };

    let http_client = &state.http_client;
    let _ = revoke_token(
        http_client, 
        &state.config.discord.client_id, 
        &state.config.discord.client_secret, 
        &linked_user.1.access_token
    ).await.inspect_err(|err| println!("Failed to revoke Discord token (ignoring): {}", err));

    let delete_result = users::Entity::delete_by_id(linked_user.0.id)
        .exec(&state.db)
        .await;

    match delete_result {
        Ok(_) => Ok(StatusCode::OK.into_response()),
        Err(err) => {
            eprintln!("Failed to delete token/user from database: {err}");
            Err((StatusCode::INTERNAL_SERVER_ERROR, "Failed to delete user from database".into()))
        }
    }
}

pub async fn player_guild_handler(
    State(state): State<SharedServerState>,
    headers: HeaderMap,
    Path((minecraft_uuid, guild_id)): Path<(String, String)>
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let auth_header = headers.get("Authorization").and_then(|h| h.to_str().ok());
    let expected_auth = format!("Bearer {}", state.config.minecraft.webhook_secret);

    if auth_header != Some(&expected_auth) {
        return Err((StatusCode::UNAUTHORIZED, "Invalid authentication string".into()));
    }

    let linked_user = match users::Entity::find()
        .filter(users::Column::MinecraftUuid.eq(minecraft_uuid))
        .find_both_related(oauth_tokens::Entity)
        .require_one(&state.db)
        .await {
            Ok(u) => u,
            Err(_) => {
                eprintln!("Lookup failed. User doesn't exist or has a duplicate");
                return Err((StatusCode::NOT_FOUND, "User doesn't exist".into()))
            }
        };

    let member_result = DiscordGuild::get_membership_of_id(&state.discord, &linked_user.1.access_token, &guild_id).await;

    match member_result {
        Ok(member) => {
            let res = MemberResponse{
                is_member: true,
                is_pending: member.pending.unwrap_or(false),
                nickname: member.nick
            };
            
            return Ok((StatusCode::OK, Json(res)));
        },
        Err(_) => {
            return Err((StatusCode::NOT_FOUND, "User is not a member of this guild".into()));
        }
    }
}