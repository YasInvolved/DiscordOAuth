use crate::{
    config::SharedServerState, handlers::{
        callback::callback_handler, init::init_auth_handler, player::{player_handler, player_unlink_handler}
    }
};

use axum::{
    Router, 
    routing::{get, post}
};

mod utils;
pub mod init;
pub mod callback;
mod player;

pub fn build_router(state: &SharedServerState) -> Router {
    Router::new()
        .route("/health", get(|| async { "API is running" }))
        .route("/init", post(init_auth_handler))
        .route("/callback", get(callback_handler))
        .route("/player/{minecraft_uuid}", get(player_handler))
        .route("/player/{minecraft_uuid}/unlink", post(player_unlink_handler))
        // .route("/player/{uuid}/{guild_id}", method_router)
        .with_state(state.clone())
}