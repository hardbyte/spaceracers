use crate::player::Player;
use crate::ship::Ship;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GameState {
    pub game_id: Uuid,
    pub players: Vec<Player>,
    pub ships: HashMap<Uuid, Ship>,
    pub map_name: String,
    pub state: String, // e.g., "lobby", "running", "finished"
}


