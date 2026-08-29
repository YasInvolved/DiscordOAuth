use crate::{
    config::SharedServerState, handlers::{
        callback::callback_handler, init::init_auth_handler, player::{player_guild_handler, player_handler, player_unlink_handler}
    }
};

use axum::{
    Router, 
    routing::{get, post}
};

use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

pub mod init;
pub mod callback;
mod player;

pub fn build_router(state: &SharedServerState) -> Router {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,tower_http=debug,sea_orm=warn,sqlx=warn".into())   
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    Router::new()
        .route("/health", get(|| async { "API is running" }))
        .route("/init", post(init_auth_handler))
        .route("/callback", get(callback_handler))
        .route("/player/{minecraft_uuid}", get(player_handler))
        .route("/player/{minecraft_uuid}/unlink", post(player_unlink_handler))
        .route("/player/{uuid}/{guild_id}", get(player_guild_handler))
        .layer(TraceLayer::new_for_http())
        .with_state(state.clone())
}