use crate::{config::SharedServerState, handlers::{callback::callback_handler, init::init_auth_handler}};
use axum::{
    Router, 
    routing::{get, post}
};

mod utils;
pub mod init;
pub mod callback;

pub fn build_router(state: &SharedServerState) -> Router {
    Router::new()
        .route("/health", get(|| async { "API is running" }))
        .route("/init", post(init_auth_handler))
        .route("/callback", get(callback_handler))
        .with_state(state.clone())
}