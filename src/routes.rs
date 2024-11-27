use serde::{Deserialize, Serialize};
use axum::extract::State;
use axum::Json;
use uuid::Uuid;
use crate::app_state::AppState;
use crate::Player;

// Root handler
pub async fn root_handler() -> &'static str {
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