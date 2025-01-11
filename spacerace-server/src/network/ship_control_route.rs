use crate::app_state::AppState;
use crate::control::ShipInput;
use crate::game_state::{GameState, GameStatus};
use crate::components::ship::Ship;
use axum::extract::State;
use axum::Json;
use serde::{Deserialize, Serialize};
use std::ops::Deref;
use uuid::Uuid;

#[derive(Deserialize)]
pub struct ControlInput {
    password: String,

    // movement is either 0, or 1 for off, or full forward thrust
    thrust: i8,
    // Rotation is either -1, 0, or 1 for left, none, or right rotational thrust
    rotation: i8,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ShipControlResponse {
    pub status: String,
}

#[axum::debug_handler]
#[tracing::instrument(skip_all, fields(player.id, player.name, player.input, game.id))]
pub async fn ship_control_handler(
    State(state): State<AppState>,
    Json(input): Json<ControlInput>,
) -> Json<ShipControlResponse> {
    // 1. Validate the password and player existence in the given game retrieving the player uuid
    if let Some((game_state, player)) = state.get_active_player_by_password(&input.password) {
        tracing::Span::current()
            .record("player.id", &player.id.to_string().deref())
            .record("player.name", &player.name.deref())
            .record("game.id", game_state.game_id.to_string().deref());
        // 2. If valid, update the `AppState.control_inputs` for this player
        let mut control_inputs = state.control_inputs.lock().unwrap();
        control_inputs.insert(
            player.id,
            ShipInput {
                thrust: input.thrust.into(),
                rotation: input.rotation.into(),
            },
        );
        tracing::Span::current().record(
            "player.input",
            &format!("{:?}", control_inputs.get(&player.id)),
        );
        tracing::debug!(
            input = &format!("{:?}", control_inputs.get(&player.id)),
            "Updated control input"
        );
        Json(ShipControlResponse {
            status: "ok".to_string(),
        })
    } else {
        // 3. Respond with a `ShipControlResponse` indicating failure.
        Json(ShipControlResponse {
            status: "error".to_string(),
        })
    }
}
