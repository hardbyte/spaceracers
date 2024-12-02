use crate::network::routes::PendingGame;
use crate::player::PlayerRegistration;
use crate::ship::Ship;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum GameStatus {
    Queued,
    Running,
    Finished,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GameState {
    pub game_id: Uuid,
    pub players: Vec<PlayerRegistration>,
    pub ships: HashMap<Uuid, Ship>,
    pub map_name: String,
    pub state: GameStatus,
}

impl GameState {
    pub fn new_from_pending_game(pending_game: PendingGame) -> GameState {
        GameState {
            game_id: pending_game.game_id,
            players: pending_game.players,
            ships: HashMap::new(),
            map_name: pending_game.map_name,
            state: GameStatus::Queued,
        }
    }
}
