use serde::{Deserialize, Serialize};

// Player structure
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Player {
    name: String,
    team: String,
    password: String,
    game_id: Option<String>,
}