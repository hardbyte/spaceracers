use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::Player;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GameState {
    pub game_id: Uuid,
    pub players: Vec<Player>,
    pub ships: HashMap<Uuid, Ship>,
    pub map_name: String,
    pub state: String, // e.g., "lobby", "running", "finished"
}