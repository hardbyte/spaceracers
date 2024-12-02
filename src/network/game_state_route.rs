use serde::{Deserialize, Serialize};
use axum::extract::State;
use axum::Json;
use std::ops::Deref;
use crate::app_state::AppState;
use crate::game_state::GameState;

#[derive(Debug, Serialize, Deserialize)]
pub enum StateResponse {
    Inactive,
    Active(GameState),
}

// TODO update to take a game-id query parameter
#[axum::debug_handler]
pub async fn state_handler(State(state): State<AppState>) -> Json<StateResponse> {
    if let Some(game) = state.active_game.lock().unwrap().deref() {
        Json(StateResponse::Active(game.clone()))
    } else {
        Json(StateResponse::Inactive)
    }
}