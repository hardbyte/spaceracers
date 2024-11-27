use crate::app_state::AppState;
use crate::player::Player;
use axum::extract::State;
use axum::Json;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use uuid::Uuid;

// Root handler
pub async fn root_handler() -> &'static str {
    "Welcome to Space Race!"
}

const MIN_PLAYERS: usize = 2; // Minimum players to start a game
const LOBBY_WAIT_TIME: Duration = Duration::from_secs(30);

// Lobby handler
#[axum::debug_handler]
pub async fn lobby_handler(
    State(state): State<AppState>,
    Json(payload): Json<Player>,
) -> Json<LobbyResponse> {
    let mut lobby = state.lobby_players.lock().await;

    // Add player to lobby

    // Generate a unique game ID (for simplicity, we'll assign a new game to each player)
    let game_id = Uuid::new_v4().to_string();

    // Clone the player and assign the game ID
    let mut player = payload.clone();
    player.game_id = Some(game_id.clone());

    // Add player to the lobby
    lobby.insert(player.password.clone(), player.clone());

    // Respond with the lobby response
    Json(LobbyResponse {
        name: player.name,
        game: game_id,
        map: "default_map".to_string(),
    })
}

// Lobby response structure
#[derive(Debug, Serialize, Deserialize)]
pub struct LobbyResponse {
    pub name: String,
    pub game: String,
    pub map: String,
}
