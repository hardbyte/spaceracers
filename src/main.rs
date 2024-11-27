mod telemetry;

use axum::{
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, net::SocketAddr, sync::Arc};
use axum::extract::State;
use opentelemetry::global;
use tokio::sync::Mutex;
use tracing::info;
use uuid::Uuid;
use crate::telemetry::init_telemetry;

fn app() -> Router {
    // Initialize shared state
    let app_state = AppState::new();

    // Build our application with routes
    Router::new()
        .route("/", get(root_handler))
        .route("/lobby", post(lobby_handler))
        .with_state(app_state)
}

#[tokio::main]
async fn main() {
    init_telemetry();

    let app = app();

    // Run our app with hyper on localhost:5000
    let addr = SocketAddr::from(([0, 0, 0, 0], 5000));
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();

    info!("Listening on {}", addr);
    axum::serve(listener, app)//.into_make_service_with_connect_info::<SocketAddr>())
        .await
        .unwrap();
    info!("Shutting down...");

    global::shutdown_tracer_provider();
}

// Application state
#[derive(Clone, Debug)]
struct AppState {
    // Stores players waiting in the lobby
    lobby_players: Arc<Mutex<HashMap<String, Player>>>,
}

impl AppState {
    fn new() -> Self {
        Self {
            lobby_players: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

// Player structure
#[derive(Debug, Serialize, Deserialize, Clone)]
struct Player {
    name: String,
    team: String,
    password: String,
    game_id: Option<String>,
}

// Root handler
async fn root_handler() -> &'static str {
    "Welcome to Space Race!"
}

// Lobby handler
#[axum::debug_handler]
async fn lobby_handler(
    State(state): State<AppState>,
    Json(payload): Json<Player>,
) -> Json<LobbyResponse> {

    // Generate a unique game ID (for simplicity, we'll assign a new game to each player)
    let game_id = Uuid::new_v4().to_string();

    // Clone the player and assign the game ID
    let mut player = payload.clone();
    player.game_id = Some(game_id.clone());

    // Add player to the lobby
    state.lobby_players
        .lock()
        .await
        .insert(player.password.clone(), player.clone());

    // Respond with the lobby response
    Json(LobbyResponse {
        name: player.name,
        game: game_id,
        map: "default_map".to_string(),
    })
}

// Lobby response structure
#[derive(Debug, Serialize, Deserialize)]
struct LobbyResponse {
    name: String,
    game: String,
    map: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::Body,
        extract::connect_info::MockConnectInfo,
        http::{Request, StatusCode}
    };
    use http_body_util::BodyExt; // for `collect`
    use serde_json::{json, Value};
    use tokio::net::TcpListener;
    use tower::{Service, ServiceExt}; // for `call`, `oneshot` and `ready`

    #[tokio::test]
    async fn test_root_handler() {
        let app = Router::new().route("/", get(root_handler));

        let response = app
            .oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = response.into_body().collect().await.unwrap().to_bytes();
        assert_eq!(&body[..], b"Welcome to Space Race!");
    }

    #[tokio::test]
    async fn test_lobby_handler() {
        let app_state = AppState::new();
        let app = Router::new()
            .route("/lobby", post(lobby_handler))
            .with_state(app_state.clone());

        let player = Player {
            name: "TestPlayer".to_string(),
            team: "TestTeam".to_string(),
            password: "secret".to_string(),
            game_id: None,
        };

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/lobby")
                    .header(http::header::CONTENT_TYPE, mime::APPLICATION_JSON.as_ref())
                    .body(Body::from(
                        serde_json::to_string(&player).unwrap())
                    )
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body_bytes = response.into_body().collect().await.unwrap().to_bytes();
        let lobby_response: LobbyResponse = serde_json::from_slice(&body_bytes).unwrap();

        assert_eq!(lobby_response.name, "TestPlayer");
        assert_eq!(lobby_response.map, "default_map");
        assert!(Uuid::parse_str(&lobby_response.game).is_ok());
    }
}
