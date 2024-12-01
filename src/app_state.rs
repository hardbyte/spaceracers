use crate::game_state::GameState;
use crate::player::Player;
use std::collections::HashMap;
use std::sync::Arc;
use bevy::prelude::Resource;
use tokio::sync::Mutex;
use uuid::Uuid;

// Application state will be shared between tokio and bevy so needs to be thread-safe
#[derive(Clone, Debug, Resource)]
pub struct AppState {
    // Stores players waiting in the lobby
    pub lobby_players: Arc<Mutex<HashMap<String, Player>>>,
    pub games: Arc<Mutex<HashMap<Uuid, GameState>>>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            lobby_players: Arc::new(Mutex::new(HashMap::new())),
            games: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}
