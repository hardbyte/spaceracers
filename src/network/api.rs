use crate::app_state::AppState;
use crate::network::{game_state_route, lobby_route, ship_control_route};

use axum::routing::{get, post};
use axum::Router;

pub async fn root_handler() -> &'static str {
    "Welcome to Space Race!"
}

pub fn create_app(app_state: AppState) -> Router {
    tracing::debug!("Building our axum application routes");

    Router::new()
        .route("/", get(root_handler))
        .route("/lobby", post(lobby_route::lobby_handler))
        .route("/state", get(game_state_route::state_handler))
        .route("/control", post(ship_control_route::ship_control_handler))
        .with_state(app_state)
}
