use crate::app_state::AppState;
use crate::game_state::GameState;
use crate::player::Player;
use axum::extract::State;
use axum::Json;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;
use uuid::Uuid;

const MIN_PLAYERS: usize = 2; // Minimum players to start a game
const LOBBY_WAIT_TIME: Duration = Duration::from_secs(30);

// Lobby response structure
// TODO improve to deal with responding to players queued waiting for game to start
// TODO consider adding a countdown to the game starting...

#[derive(Debug, Serialize, Deserialize)]
pub struct LobbyResponse {
    pub name: String,
    pub game: String,
    pub map: String,
}

#[axum::debug_handler]
pub async fn lobby_handler(
    State(state): State<AppState>,
    Json(payload): Json<Player>,
) -> Json<LobbyResponse> {
    let mut lobby = state.lobby_players.lock().await;

    tracing::info!(?payload, "Request to add player to lobby");
    let game_id = if lobby.len() >= MIN_PLAYERS {
        // Start a new game if the lobby is full
        let game_id = Uuid::new_v4();
        let game_state = GameState {
            game_id: game_id.clone(),
            players: lobby.drain().map(|(_, p)| p).collect(),
            ships: HashMap::new(),
            map_name: "default_map".to_string(),
            state: "running".to_string(),
        };
        state.games.lock().await.insert(Uuid::new_v4(), game_state);
        game_id.to_string()
    } else {
        "waiting_for_players".to_string()
    };

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

#[axum::debug_handler]
pub async fn state_handler(State(state): State<AppState>) -> Json<GameState> {
    let games = state.games.lock().await;
    if let Some((_, game)) = games.iter().next() {
        Json(game.clone())
    } else {
        Json(GameState {
            game_id: Uuid::new_v4(),
            players: vec![],
            ships: HashMap::new(),
            map_name: "default_map".to_string(),
            state: "waiting".to_string(),
        })
    }
}
