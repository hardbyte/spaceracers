use crate::app_state::AppState;
use axum::routing::{get, post};
use axum::Router;

pub fn create_app(app_state: AppState) -> Router {
    tracing::debug!("Building our axum application routes");

    Router::new()
        .route("/", get(crate::routes::root_handler))
        .route("/lobby", post(crate::routes::lobby_handler))
        .with_state(app_state)
}
