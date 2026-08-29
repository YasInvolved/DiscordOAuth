use axum::{
    extract::{Query, State},
    http::{StatusCode},
    response::IntoResponse
};
use chrono::{Duration, Utc};
use sea_orm::{ActiveModelTrait, ActiveValue::Set, ColumnTrait, EntityTrait, QueryFilter};

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    config::SharedServerState, 
    crypto::{decrypt_token, encrypt_token}, 
    discord::{oauth2::{exchange_token, refresh_token}, user::DiscordUser}, 
    entity::{oauth_states, oauth_tokens, users}, 
};

#[derive(Deserialize)]
pub struct CallbackQuery {
    pub code: String,
    pub state: String
}

#[derive(Serialize)]
struct WebhookNotification {
    minecraft_uuid: String,
    user_id: String,
    event: String
}

pub async fn callback_handler(
    State(state): State<SharedServerState>,
    Query(query): Query<CallbackQuery>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let config = &state.config;

    let state_uuid = Uuid::parse_str(&query.state)
        .map_err(|_| (StatusCode::BAD_REQUEST, "Invalid state UUID".into()))?;

    let state_record = oauth_states::Entity::find_by_id(state_uuid)
    .one(&state.db)
    .await
    .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Database error".into()))?
    .ok_or((StatusCode::BAD_REQUEST, "Invalid or expired state".into()))?;

    if state_record.expires_at < Utc::now() {
        let _ = oauth_states::Entity::delete_by_id(state_uuid).exec(&state.db).await;
        return Err((StatusCode::BAD_REQUEST, "State has expired".into()));
    }

    let discord_config = &state.config.discord;
    let http_client = &state.http_client;
    let token_res = match exchange_token(
        http_client, 
        &discord_config.client_id, 
        &discord_config.client_secret, 
        &query.code, 
        &discord_config.redirect_uri
    ).await {
        Ok(t) => t,
        Err(e) => {
            eprintln!("Discord token exchange failed: {}", e);
            Err((StatusCode::INTERNAL_SERVER_ERROR, "Failed to exchange Discord token".into()))?
        }
    };


    let user_res = match DiscordUser::fetch(&state.discord, &token_res.access_token).await {
        Ok(u) => u,
        Err(e) => {
            eprintln!("Failed to fetch user: {}", e.to_string());
            return Err((StatusCode::INTERNAL_SERVER_ERROR, "Failed to fetch user. Try again.".into()));
        }
    };

    let key_bytes: [u8; 32] = config.discord.aes_key.as_bytes().try_into()
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Invalid AES key".into()))?;

    let encrypted_refresh = encrypt_token(&token_res.refresh_token, &key_bytes)
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Encryption failed".into()))?;

    let token_expiry = Utc::now() + Duration::seconds(token_res.expires_in);

    let existing_user = users::Entity::find()
            .filter(users::Column::DiscordId.eq(&user_res.id))
            .one(&state.db)
            .await
            .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Database error".into()))?;

    let user_id = if let Some(user) = existing_user {
        let mut user_active: users::ActiveModel = user.into();
        user_active.minecraft_uuid = Set(Some(state_record.minecraft_uuid.clone()));
        let updated = user_active.update(&state.db).await
            .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Failed updating user".into()))?;
        updated.id
    } else {
        let new_user = users::ActiveModel {
            id: Set(Uuid::new_v4()),
            discord_id: Set(user_res.id),
            minecraft_uuid: Set(Some(state_record.minecraft_uuid.clone()))
        };

        let created = new_user.insert(&state.db).await
            .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Failed creating user".into()))?;
        created.id
    };

    let _ = oauth_tokens::Entity::delete_many()
        .filter(oauth_tokens::Column::UserId.eq(user_id))
        .exec(&state.db)
        .await;

    let new_token = oauth_tokens::ActiveModel {
        id: Set(Uuid::new_v4()),
        user_id: Set(user_id),
        access_token: Set(token_res.access_token),
        refresh_token: Set(encrypted_refresh),
        expires_at: Set(token_expiry),
    };

    new_token.insert(&state.db).await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Failed saving tokens".into()))?;

    let _ = oauth_states::Entity::delete_by_id(state_uuid).exec(&state.db).await;
    notify_server(&state_record.minecraft_uuid, &user_id.to_string(), &state).await;

    Ok("Successfully linked your Minecraft account with Discord! You can close this tab now.")
}

pub async fn start_token_refresh_task(state: SharedServerState) {
    let mut interval = tokio::time::interval(std::time::Duration::from_secs(60));
    let client = reqwest::Client::new();
    let config = &state.config.discord;

    let key_bytes: [u8; 32] = match config.aes_key.as_bytes().try_into() {
        Ok(k) => k,
        Err(_) => {
            eprintln!("Invalid AES key for background refresher. Task exiting");
            return;
        }
    };

    loop {
        interval.tick().await;

        let treshold = Utc::now() + Duration::minutes(5);

        let expiring_tokens = match oauth_tokens::Entity::find()
            .filter(oauth_tokens::Column::ExpiresAt.lte(treshold))
            .all(&state.db)
            .await {
                Ok(tokens) => tokens,
                Err(err) => {
                    eprintln!("Failed querying expiring tokens: {err}");
                    continue;
                }
        };

        for token_record in expiring_tokens {
            let token = match decrypt_token(&token_record.refresh_token, &key_bytes) {
                Ok(t) => t,
                Err(err) => {
                    eprintln!("Failed to decrypt token for record {}: {err}", token_record.id);
                    continue;
                }
            };

            let res= refresh_token(
                &client, 
                &config.client_id, 
                &config.client_secret, 
                &token
            ).await;

            let response = match res {
                Ok(r) => r,
                _ => {
                    eprintln!("Failed refreshing token for record {}", token_record.id);
                    continue;
                }
            };

            if let Ok(new_encrypted_refresh) = encrypt_token(&response.refresh_token, &key_bytes) {
                let mut active_token: oauth_tokens::ActiveModel = token_record.into();
                active_token.access_token = Set(response.access_token);
                active_token.refresh_token = Set(new_encrypted_refresh);
                active_token.expires_at = Set(Utc::now() + Duration::seconds(response.expires_in));

                if let Err(e) = active_token.update(&state.db).await {
                    eprintln!("Failed updating refreshed token in DB: {e}");
                }
            }
        }
    }
}

async fn notify_server(minecraft_uuid: &str, user_id: &str, state: &SharedServerState) {
    let client = &state.http_client;
    let payload = WebhookNotification{
        minecraft_uuid: minecraft_uuid.to_string(),
        user_id: user_id.to_string(),
        event: "link_complete".to_string()
    };

    let res = client.post(&state.config.minecraft.webhook_url)
        .bearer_auth(&state.config.minecraft.webhook_secret)
        .json(&payload)
        .send()
        .await;

    if let Err(e) = res {
        eprintln!("Failed to notify Minecraft server for UUID {}: {}", minecraft_uuid, e);
    }
}