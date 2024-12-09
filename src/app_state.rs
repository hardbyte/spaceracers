use crate::game_state::{GameState};
use crate::game_state::PendingGame;
use bevy::prelude::Resource;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;
use uuid::Uuid;
use crate::control::ShipInput;
use crate::player::Player;

// Application state will be shared between tokio and bevy so needs to be thread-safe
#[derive(Clone, Debug, Resource)]
pub struct AppState {
    // Stores players waiting in the lobby
    pub lobby: Arc<Mutex<Vec<PendingGame>>>,
    pub active_game: Arc<Mutex<Option<GameState>>>,

    // Stores current inputs from players
    pub control_inputs: Arc<Mutex<HashMap<Uuid, ShipInput>>>,

}

impl AppState {
    pub fn new() -> Self {
        Self {
            lobby: Arc::new(Mutex::new(Vec::new())),
            active_game: Arc::new(Mutex::new(None)),
            control_inputs: Arc::new(Mutex::new(Default::default())),
        }
    }

    pub fn get_active_player_by_password(&self, password: &str) -> Option<(GameState, Player)> {
        let active_game = self.active_game.lock().unwrap();
        if let Some(game) = active_game.as_ref() {
            for player in game.players.iter() {
                if player.password == password {
                    return Some((game.clone(), player.clone()));
                }
            }
        }
        tracing::debug!("player not found");
        None
    }
}
