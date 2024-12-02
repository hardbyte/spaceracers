use crate::network::lobby_route::PendingGame;
use crate::player::{Player, PlayerRegistration};
use crate::ship::Ship;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use bevy::prelude::Entity;
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
    pub ships: HashMap<Uuid, Entity>,
    pub map_name: String,
    pub state: GameStatus,
}


impl From<PendingGame> for GameState {
    fn from(pending_game: PendingGame) -> Self {
        GameState {
            game_id: pending_game.game_id,
            players: pending_game.players,
            ships: HashMap::new(),
            map_name: pending_game.map_name,
            state: GameStatus::Queued,
        }
    }
}