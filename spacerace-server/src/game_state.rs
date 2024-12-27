use crate::player::Player;
use crate::ship::Ship;
use bevy::prelude::Entity;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub enum GameStatus {
    Queued,
    Running,
    Finished,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GameState {
    pub game_id: Uuid,
    pub players: Vec<Player>,

    pub ships: Vec<Ship>,
    pub map_name: String,
    pub state: GameStatus,
}

impl From<PendingGame> for GameState {
    fn from(pending_game: PendingGame) -> Self {
        GameState {
            game_id: pending_game.game_id,
            players: pending_game.players,
            ships: vec![],
            map_name: pending_game.map_name,
            state: GameStatus::Queued,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PendingGame {
    pub game_id: Uuid,
    pub players: Vec<Player>,
    pub map_name: String,
}

impl PendingGame {
    pub(crate) fn new() -> PendingGame {
        PendingGame {
            game_id: Uuid::new_v4(),
            players: vec![],
            map_name: "tiled".to_string(),
        }
    }
}
