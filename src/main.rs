mod app;
mod app_state;
mod game_state;
mod player;
mod routes;
mod ship;
mod telemetry;

mod tests;

use app_state::AppState;
use axum::extract::State;
use axum::Json;
use opentelemetry::global;
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use tracing::info;
use uuid::Uuid;

#[tokio::main]
async fn main() {
    telemetry::init();

    let app = app::create_app();

    // Run our app with hyper on localhost:5000
    let addr = SocketAddr::from(([0, 0, 0, 0], 5000));
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();

    info!("Listening on {}", addr);
    axum::serve(listener, app) //.into_make_service_with_connect_info::<SocketAddr>())
        .await
        .unwrap();
    info!("Shutting down...");

    global::shutdown_tracer_provider();
}
