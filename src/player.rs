use bevy::prelude::Component;
use serde::{Deserialize, Serialize};

// Player structure
#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct Player {
    pub name: String,
    pub team: String,
    pub password: String,

    // Game ID assigned to the player
    pub game_id: Option<String>,
}
