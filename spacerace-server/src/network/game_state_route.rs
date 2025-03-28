use crate::app_state::AppState;
use crate::components::ship::Ship;
use crate::game_state::{GameState, GameStatus};
use axum::extract::{Query, State};
use axum::Json;
use serde::{Deserialize, Serialize};
use std::ops::Deref;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub enum StateResponse {
    Inactive,
    Active(PublicGameState),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PublicGameState {
    pub game_id: Uuid,
    pub ships: Vec<Ship>,
    pub map_name: String,
    pub state: GameStatus,
}

impl From<&GameState> for PublicGameState {
    fn from(game: &GameState) -> Self {
        PublicGameState {
            game_id: game.game_id,
            ships: game.ships.clone(),
            map_name: game.map.name.clone(),
            state: game.state.clone(),
        }
    }
}

#[derive(Deserialize, Debug)]
pub struct StateQuery {
    pub game_id: String,
}

#[axum::debug_handler]
#[tracing::instrument(skip(state), fields(game.id))]
pub async fn state_handler(
    State(state): State<AppState>,
    query: Option<Query<StateQuery>>,
) -> Json<StateResponse> {
    match state.active_game.lock() {
        Ok(guard) => {
            let active_game = match guard.as_ref() {
                Some(game) => game,
                None => {
                    tracing::debug!("state requested but no game running");
                    return Json(StateResponse::Inactive);
                }
            };

            let query_game_id = query
                .map(|q| q.game_id.clone())
                .unwrap_or_else(|| active_game.game_id.to_string());

            tracing::Span::current().record("game.id", &active_game.game_id.to_string().deref());

            if query_game_id == active_game.game_id.to_string() {
                let public_game_state = PublicGameState::from(active_game);
                tracing::debug!(state = ?public_game_state, "Returning game state");
                Json(StateResponse::Active(public_game_state))
            } else {
                tracing::debug!("Game state requested for inactive game");
                Json(StateResponse::Inactive)
            }
        }
        Err(poisoned) => {
            tracing::error!("Failed to acquire lock on active game: {:?}", poisoned);
            return Json(StateResponse::Inactive);
        }
    }
}
