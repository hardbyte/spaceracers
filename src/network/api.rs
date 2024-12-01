use crate::app_state::AppState;
use crate::network::routes;

use axum::routing::{get, post};
use axum::Router;

pub async fn root_handler() -> &'static str {
    "Welcome to Space Race!"
}

pub fn create_app(app_state: AppState) -> Router {
    tracing::debug!("Building our axum application routes");

    Router::new()
        .route("/", get(root_handler))
        .route("/lobby", post(routes::lobby_handler))
        .route("/state", get(routes::state_handler))
        .with_state(app_state)
}
