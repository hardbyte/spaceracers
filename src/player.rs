use bevy::prelude::Component;
use serde::{Deserialize, Serialize};

// Player structure
#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct PlayerRegistration {
    pub name: String,
    pub team: Option<String>,
    pub password: String,
}


#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct Player {
    pub id: uuid::Uuid,

    pub name: String,
    pub team: Option<String>,
    pub password: String,
}

impl Player {
    pub fn new(name: String, team: Option<String>, password: String) -> Self {
        Self {
            id: uuid::Uuid::new_v4(),
            name,
            team,
            password,
        }
    }
}

impl From<PlayerRegistration> for Player {
    fn from(registration: PlayerRegistration) -> Self {
        Self::new(registration.name, registration.team, registration.password)
    }
}