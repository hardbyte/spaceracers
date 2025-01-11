use crate::app_state::AppState;
use crate::game_state::PendingGame;
use crate::game_state::{GameState, GameStatus};
use crate::components::{Player, PlayerRegistration};
use crate::components::ship::Ship;
use axum::extract::State;
use axum::Json;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::future::pending;
use std::ops::Deref;
use std::time::Duration;
use tracing::info;
use uuid::Uuid;

const MIN_PLAYERS: usize = 1; // Minimum players to start a game
const MAX_PLAYERS: usize = 5; // Maximum players for a game
const LOBBY_WAIT_TIME: Duration = Duration::from_secs(30);

// Lobby response structure
// TODO improve to deal with responding to players queued waiting for game to start
// TODO consider adding a countdown to the game starting...

#[derive(Debug, Serialize, Deserialize)]
pub struct LobbyResponse {
    pub player_id: String,
    pub game_id: String,
    pub map: String,
}

#[axum::debug_handler]
pub async fn lobby_handler(
    State(state): State<AppState>,
    Json(payload): Json<PlayerRegistration>,
) -> Json<LobbyResponse> {
    info!(?payload, "Request to add player to lobby");
    let player = Player::from(payload.clone());

    let mut pending_games = state.lobby.lock().unwrap();

    // If no pending game exists or if they are all full, create a new one
    if pending_games.len() == 0 || pending_games.iter().all(|g| g.players.len() >= MAX_PLAYERS) {
        pending_games.push(PendingGame::new());
    }

    let pending_game = pending_games
        .iter_mut()
        .filter(|g| g.players.len() < MAX_PLAYERS)
        .next()
        .unwrap();

    tracing::info!(player_id=?player.id, game_id=?pending_game.game_id, "Player will be added to pending game");

    // Add the player to the pending game
    pending_game.players.push(player);

    // Check if the pending game is now full
    if pending_game.players.len() >= MAX_PLAYERS {
        info!(game_id=?pending_game.game_id, "Pending game is now full");

        // // Remove the game from the lobby
        // let full_game = lobby.iter().position(|g| g.game_id == pending_game.game_id)
        //     .map(|index| lobby.swap_remove(index))
        //     .unwrap();
        //
        // // Move the game to the active games
        // let game_state = GameState::new_from_pending_game(full_game.clone());
        // state.active_game.lock().unwrap().insert(full_game.game_id, game_state);
        //
        // tracing::info!(game_id=full_game.game_id, "Sending a Game Started event to Bevy");
        //
        // // TODO work out how we want to handle this
        // // let _ = state.game_events_tx.send(GameEvent::GameStarted {
        // //     game_id: full_game.game_id,
        // //     players: full_game.players.clone(),
        // // });
    }

    // Respond with the lobby response
    Json(LobbyResponse {
        player_id: payload.name,
        game_id: pending_game.game_id.to_string(),
        map: pending_game.map_name.clone(),
    })
}
