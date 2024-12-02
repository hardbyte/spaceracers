
use bevy::prelude::*;
use bevy_rapier2d::prelude::*;
use crate::app_state::AppState;
use crate::game_state::{GameState, GameStatus};

const SPRITE_SIZE: f32 = 25.0;

// Enum that will be used as a global state for the game
#[derive(Clone, Copy, Default, Eq, PartialEq, Debug, Hash, States)]
pub enum ServerState {
    #[default]
    Inactive,

    Active,
}

#[derive(Debug, Event)]
pub enum GameEvent {
    GameStarted { game_id: uuid::Uuid },
    GameFinished { game_id: uuid::Uuid },
}

pub fn game_scheduler_system(app_state: Res<AppState>, mut writer: EventWriter<GameEvent>) {
    let mut lobby = app_state.lobby.lock().unwrap();
    let mut active_game = app_state.active_game.lock().unwrap();

    // If there's no active game, check if we can start one
    if active_game.is_none() {
        if let Some(index) = lobby.iter().position(|game| !game.players.is_empty()) {
            // Remove the pending game from the lobby
            let pending_game = lobby.remove(index);

            // Create a new GameState from the pending game
            let game_state = GameState::from(pending_game.clone());

            // Update the active game
            *active_game = Some(game_state.clone());

            // Send a GameStarted event so other Bevy systems can react
            writer.send(GameEvent::GameStarted {
                game_id: game_state.game_id,
            });

            tracing::info!(game_id=?game_state.game_id, "Game started");
        }
    }
}


fn get_starting_positions(num_players: usize) -> Vec<Vec2> {
    // Return a list of starting positions based on the number of players and the current map
    // For simplicity, we'll hardcode some positions
    vec![
        Vec2::new(-200.0, 150.0),
        Vec2::new(-200.0, 100.0),
        Vec2::new(-200.0, 50.0),
        Vec2::new(-200.0, 0.0),
    ]
}