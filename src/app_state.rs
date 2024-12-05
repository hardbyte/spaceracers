use crate::game_state::{GameState};
use crate::network::lobby_route::PendingGame;
use bevy::prelude::Resource;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;
use uuid::Uuid;

// Application state will be shared between tokio and bevy so needs to be thread-safe
#[derive(Clone, Debug, Resource)]
pub struct AppState {
    // Stores players waiting in the lobby
    pub lobby: Arc<Mutex<Vec<PendingGame>>>,
    pub active_game: Arc<Mutex<Option<GameState>>>,

    // A channel for sending events from Axum to Bevy
    //pub game_events_tx: mpsc::UnboundedSender<GameEvent>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            lobby: Arc::new(Mutex::new(Vec::new())),
            active_game: Arc::new(Mutex::new(None)),
        }
    }
}
