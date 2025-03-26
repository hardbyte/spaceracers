use crate::components::ship::Ship;
use crate::components::Player;
use crate::map::{Map, NamedMapId};

use bevy::asset::AssetId;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub enum GameStatus {
    Queued,
    Running,
    Finished,
}

#[derive(Debug, Clone)]
pub struct GameState {
    pub game_id: Uuid,
    pub players: Vec<Player>,
    pub finish_times: HashMap<Uuid, f32>,

    pub ships: Vec<Ship>,
    pub map: Map,
    pub state: GameStatus,
}

impl GameState {
    pub(crate) fn new(
        game_id: Uuid,
        players: Vec<Player>,
        map: Map,
    ) -> Result<Self, anyhow::Error> {
        Ok(GameState {
            game_id: game_id,
            players: players.clone(),
            ships: vec![],
            map,
            state: GameStatus::Queued,
            finish_times: HashMap::new(),
        })
    }
}

#[derive(Debug, Clone)]
pub struct PendingGame {
    pub game_id: Uuid,
    pub players: Vec<Player>,
    pub map_id: NamedMapId,
}

impl PendingGame {
    pub(crate) fn new(map_id: NamedMapId) -> PendingGame {
        PendingGame {
            game_id: Uuid::new_v4(),
            players: vec![],
            map_id,
        }
    }
}
