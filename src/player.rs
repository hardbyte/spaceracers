use bevy::prelude::Component;
use serde::{Deserialize, Serialize};

// Player structure
#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct PlayerRegistration {
    pub name: String,
    pub team: Option<String>,
    pub password: String,
}
